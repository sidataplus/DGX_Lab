#![forbid(unsafe_code)]

//! Constrained virtual shell. Commands are parsed into simulator operations;
//! nothing is passed to an operating-system process.

use dgxlab_contracts::{JobId, TerminalKind, TerminalLine};
use serde::{Deserialize, Serialize};
use sim_core::SimulationWorld;
use slurm_model::{JobSpec, JobStatus, Tres};
use std::collections::BTreeMap;
use virtual_fs::{normalize_path, VfsError};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellSession {
    pub user: String,
    pub hostname: String,
    pub cwd: String,
    pub env: BTreeMap<String, String>,
    pub active_job_id: Option<JobId>,
    pub loaded_modules: Vec<String>,
    pub history: Vec<String>,
}

impl Default for ShellSession {
    fn default() -> Self {
        Self::learner()
    }
}

impl ShellSession {
    #[must_use]
    pub fn learner() -> Self {
        Self {
            user: "learner".into(),
            hostname: "dgx-login-01".into(),
            cwd: "/home/learner".into(),
            env: BTreeMap::from([
                ("HOME".into(), "/home/learner".into()),
                ("USER".into(), "learner".into()),
                ("SHELL".into(), "/bin/bash".into()),
            ]),
            active_job_id: None,
            loaded_modules: Vec::new(),
            history: Vec::new(),
        }
    }

    #[must_use]
    pub fn prompt(&self) -> String {
        let host = if self.active_job_id.is_some() { "dgx-h200-01" } else { &self.hostname };
        format!("{}@{}:{}$", self.user, host, display_cwd(&self.cwd, &self.user))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResult {
    pub lines: Vec<TerminalLine>,
    pub state_changed: bool,
}

impl CommandResult {
    fn stdout(text: impl Into<String>) -> Self {
        Self { lines: vec![TerminalLine::stdout(text)], state_changed: false }
    }

    fn error(text: impl Into<String>) -> Self {
        Self {
            lines: vec![TerminalLine { kind: TerminalKind::Stderr, text: text.into() }],
            state_changed: false,
        }
    }
}

pub fn execute_line(
    world: &mut SimulationWorld,
    shell: &mut ShellSession,
    input: &str,
) -> CommandResult {
    let input = input.trim();
    if input.is_empty() {
        return CommandResult::default();
    }
    shell.history.push(input.to_string());
    let tokens = tokenize(input);
    let Some(command) = tokens.first().map(String::as_str) else {
        return CommandResult::default();
    };
    match command {
        "help" => help(),
        "pwd" => CommandResult::stdout(shell.cwd.clone()),
        "cd" => cd(world, shell, tokens.get(1).map(String::as_str)),
        "ls" => ls(world, shell, tokens.get(1).map(String::as_str)),
        "cat" => cat(world, shell, tokens.get(1).map(String::as_str)),
        "tail" => tail(world, shell, &tokens[1..]),
        "echo" => echo(shell, &tokens[1..]),
        "env" => env(shell),
        "mkdir" => mkdir(world, shell, tokens.get(1).map(String::as_str)),
        "touch" => touch(world, shell, tokens.get(1).map(String::as_str)),
        "rm" => rm(world, shell, tokens.get(1).map(String::as_str)),
        "sinfo" => sinfo(world),
        "squeue" => squeue(world, &tokens[1..]),
        "sbatch" => sbatch(world, shell, tokens.get(1).map(String::as_str)),
        "srun" => srun(world, shell, &tokens[1..], false),
        "salloc" => srun(world, shell, &tokens[1..], true),
        "scancel" => scancel(world, tokens.get(1).map(String::as_str)),
        "scontrol" => scontrol(world, &tokens[1..]),
        "sacct" => sacct(world, &tokens[1..]),
        "nvidia-smi" => nvidia_smi(world, shell, &tokens[1..]),
        "module" => module(shell, &tokens[1..]),
        "singularity" => simulated_runtime(world, shell, "singularity", &tokens[1..]),
        "python" | "python3" | "torchrun" => simulated_runtime(world, shell, command, &tokens[1..]),
        "exit" => exit_allocation(world, shell),
        _ => CommandResult::error(format!(
            "dgxlab: unsupported command '{command}'. Type 'help' for the simulated command set."
        )),
    }
}

fn help() -> CommandResult {
    CommandResult::stdout(
        "Supported: sinfo squeue sbatch srun salloc scancel scontrol sacct \
         nvidia-smi module singularity python torchrun pwd cd ls cat tail echo env mkdir touch rm exit",
    )
}

fn cd(world: &SimulationWorld, shell: &mut ShellSession, path: Option<&str>) -> CommandResult {
    let target = path.unwrap_or_else(|| shell.env.get("HOME").map(String::as_str).unwrap_or("/"));
    match resolve_path(&shell.cwd, target) {
        Ok(path) => match world.fs.list_dir(&path) {
            Ok(_) => {
                shell.cwd = path;
                CommandResult { lines: vec![], state_changed: true }
            }
            Err(error) => CommandResult::error(format!("cd: {error}")),
        },
        Err(error) => CommandResult::error(format!("cd: {error}")),
    }
}

fn ls(world: &SimulationWorld, shell: &ShellSession, path: Option<&str>) -> CommandResult {
    let path = match resolve_path(&shell.cwd, path.unwrap_or(".")) {
        Ok(path) => path,
        Err(error) => return CommandResult::error(format!("ls: {error}")),
    };
    match world.fs.list_dir(&path) {
        Ok(entries) => CommandResult::stdout(entries.join("  ")),
        Err(VfsError::NotDirectory(_)) => CommandResult::stdout(path),
        Err(error) => CommandResult::error(format!("ls: {error}")),
    }
}

fn cat(world: &SimulationWorld, shell: &ShellSession, path: Option<&str>) -> CommandResult {
    let Some(path) = path else {
        return CommandResult::error("cat: missing virtual file operand");
    };
    let path = match resolve_path(&shell.cwd, path) {
        Ok(path) => path,
        Err(error) => return CommandResult::error(format!("cat: {error}")),
    };
    match world.fs.read_text(&path) {
        Ok(text) => CommandResult::stdout(text),
        Err(error) => CommandResult::error(format!("cat: {error}")),
    }
}

fn tail(world: &SimulationWorld, shell: &ShellSession, args: &[String]) -> CommandResult {
    let mut count = 10_usize;
    let mut path = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "-n" => {
                if let Some(value) = args.get(index + 1).and_then(|value| value.parse().ok()) {
                    count = value;
                    index += 1;
                }
            }
            token if !token.starts_with('-') => path = Some(token),
            _ => {}
        }
        index += 1;
    }
    let Some(path) = path else {
        return CommandResult::error("tail: missing virtual file operand");
    };
    let path = match resolve_path(&shell.cwd, path) {
        Ok(path) => path,
        Err(error) => return CommandResult::error(format!("tail: {error}")),
    };
    match world.fs.read_text(&path) {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().collect();
            CommandResult::stdout(lines[lines.len().saturating_sub(count)..].join("\n"))
        }
        Err(error) => CommandResult::error(format!("tail: {error}")),
    }
}

fn echo(shell: &ShellSession, args: &[String]) -> CommandResult {
    let rendered = args
        .iter()
        .map(|token| {
            if let Some(name) = token.strip_prefix('$') {
                match name {
                    "SLURM_JOB_ID" => shell
                        .env
                        .get("SLURM_JOB_ID")
                        .cloned()
                        .or_else(|| shell.active_job_id.map(|id| id.0.to_string()))
                        .unwrap_or_default(),
                    "CUDA_VISIBLE_DEVICES" => shell
                        .env
                        .get("CUDA_VISIBLE_DEVICES")
                        .cloned()
                        .unwrap_or_default(),
                    _ => shell.env.get(name).cloned().unwrap_or_default(),
                }
            } else {
                token.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    CommandResult::stdout(rendered)
}

fn env(shell: &ShellSession) -> CommandResult {
    let entries: Vec<String> = shell
        .env
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect();
    CommandResult::stdout(entries.join("\n"))
}

fn mkdir(world: &mut SimulationWorld, shell: &ShellSession, path: Option<&str>) -> CommandResult {
    let Some(path) = path else {
        return CommandResult::error("mkdir: missing virtual path");
    };
    match resolve_path(&shell.cwd, path).and_then(|path| world.fs.mkdir_all(&path).map(|_| path)) {
        Ok(_) => CommandResult { lines: vec![], state_changed: true },
        Err(error) => CommandResult::error(format!("mkdir: {error}")),
    }
}

fn touch(world: &mut SimulationWorld, shell: &ShellSession, path: Option<&str>) -> CommandResult {
    let Some(path) = path else {
        return CommandResult::error("touch: missing virtual path");
    };
    match resolve_path(&shell.cwd, path).and_then(|path| world.fs.write_file(&path, b"").map(|_| path)) {
        Ok(_) => CommandResult { lines: vec![], state_changed: true },
        Err(error) => CommandResult::error(format!("touch: {error}")),
    }
}

fn rm(world: &mut SimulationWorld, shell: &ShellSession, path: Option<&str>) -> CommandResult {
    let Some(path) = path else {
        return CommandResult::error("rm: missing virtual path");
    };
    match resolve_path(&shell.cwd, path).and_then(|path| world.fs.remove(&path).map(|_| path)) {
        Ok(_) => CommandResult { lines: vec![], state_changed: true },
        Err(error) => CommandResult::error(format!("rm: {error}")),
    }
}

fn sinfo(world: &SimulationWorld) -> CommandResult {
    let mut output = vec![format!(
        "{:<12} {:<6} {:<11} {:<5} {:<8} {}",
        "PARTITION", "AVAIL", "TIMELIMIT", "NODES", "STATE", "NODELIST"
    )];
    for partition in world.cluster.partitions.values() {
        let node_states: Vec<String> = partition
            .node_ids
            .iter()
            .filter_map(|node_id| world.cluster.nodes.get(node_id))
            .map(|node| format!("{:?}", node.status).to_lowercase())
            .collect();
        output.push(format!(
            "{:<12} {:<6} {:<11} {:<5} {:<8} {}",
            format!("{}{}", partition.id, if partition.is_default { "*" } else { "" }),
            if partition.status == slurm_model::PartitionStatus::Up { "up" } else { "down" },
            partition.max_time_ms.map(format_duration).unwrap_or_else(|| "infinite".into()),
            partition.node_ids.len(),
            node_states.join(","),
            partition.node_ids.join(",")
        ));
    }
    CommandResult::stdout(output.join("\n"))
}

fn squeue(world: &SimulationWorld, args: &[String]) -> CommandResult {
    let user_filter = option_value(args, "-u").or_else(|| option_value(args, "--user"));
    let mut output = vec![format!(
        "{:<8} {:<10} {:<16} {:<10} {:<3} {:<10} {}",
        "JOBID", "PARTITION", "NAME", "USER", "ST", "TIME", "NODELIST(REASON)"
    )];
    for job in world.jobs.values().filter(|job| !job.status.is_terminal()) {
        if user_filter.as_deref().is_some_and(|user| user != job.spec.user.as_str()) {
            continue;
        }
        let state = match job.status {
            JobStatus::Pending => "PD",
            JobStatus::Running => "R",
            _ => "--",
        };
        let location = if job.status == JobStatus::Pending {
            format!("({})", job.pending_reason.display_name())
        } else {
            job.allocation
                .as_ref()
                .map(|allocation| allocation.node_id.clone())
                .unwrap_or_default()
        };
        output.push(format!(
            "{:<8} {:<10} {:<16} {:<10} {:<3} {:<10} {}",
            job.id.0,
            job.spec.partition,
            truncate(&job.spec.name, 16),
            truncate(&job.spec.user, 10),
            state,
            format_duration(job.elapsed_ms(world.now)),
            location
        ));
    }
    CommandResult::stdout(output.join("\n"))
}

fn sbatch(world: &mut SimulationWorld, shell: &ShellSession, path: Option<&str>) -> CommandResult {
    let Some(path) = path else {
        return CommandResult::error("sbatch: error: Batch script filename required");
    };
    let path = match resolve_path(&shell.cwd, path) {
        Ok(path) => path,
        Err(error) => return CommandResult::error(format!("sbatch: error: {error}")),
    };
    let script = match world.fs.read_text(&path) {
        Ok(script) => script,
        Err(error) => return CommandResult::error(format!("sbatch: error: {error}")),
    };
    let (spec, array_tasks) = match job_spec_from_script_with_array(&script, &shell.user) {
        Ok(value) => value,
        Err(error) => return CommandResult::error(format!("sbatch: error: {error}")),
    };
    let mut submitted = Vec::new();
    let tasks = if array_tasks.is_empty() {
        vec![None]
    } else {
        array_tasks.into_iter().map(Some).collect()
    };
    for task in tasks {
        let mut task_spec = spec.clone();
        if let Some(index) = task {
            task_spec.array_index = Some(index);
            if !task_spec.name.contains('[') {
                task_spec.name = format!("{}[{index}]", task_spec.name);
            }
        }
        match world.submit_job(task_spec) {
            Ok(job_id) => submitted.push(job_id.0),
            Err(error) => {
                return CommandResult::error(format!("sbatch: error: {error}"));
            }
        }
    }
    let text = if submitted.len() == 1 {
        format!("Submitted batch job {}", submitted[0])
    } else {
        format!(
            "Submitted batch job array {} ({} tasks)",
            submitted.first().copied().unwrap_or(0),
            submitted.len()
        )
    };
    CommandResult {
        lines: vec![TerminalLine {
            kind: TerminalKind::Success,
            text,
        }],
        state_changed: true,
    }
}

fn srun(
    world: &mut SimulationWorld,
    shell: &mut ShellSession,
    args: &[String],
    allocation_only: bool,
) -> CommandResult {
    let mut spec = match job_spec_from_args(args, &shell.user) {
        Ok(spec) => spec,
        Err(error) => return CommandResult::error(format!("srun: error: {error}")),
    };
    let interactive = allocation_only || spec.command == "bash" || args.iter().any(|arg| arg == "--pty");
    if interactive {
        spec.workload_id = "interactive-shell-v1".into();
        spec.name = if allocation_only { "allocation".into() } else { "interactive".into() };
    }
    match world.submit_job(spec) {
        Ok(job_id) => {
            let job = &world.jobs[&job_id];
            if job.status == JobStatus::Running && interactive {
                shell.active_job_id = Some(job_id);
                shell.hostname = job
                    .allocation
                    .as_ref()
                    .map(|allocation| allocation.node_id.clone())
                    .unwrap_or_else(|| "dgx-h200-01".into());
                shell.env.insert("SLURM_JOB_ID".into(), job_id.0.to_string());
                let visible = job
                    .allocation
                    .as_ref()
                    .map(|allocation| {
                        (0..allocation.gpu_indices.len())
                            .map(|index| index.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .unwrap_or_default();
                shell.env.insert("CUDA_VISIBLE_DEVICES".into(), visible);
                CommandResult {
                    lines: vec![TerminalLine {
                        kind: TerminalKind::Success,
                        text: format!("srun: Granted job allocation {}", job_id.0),
                    }],
                    state_changed: true,
                }
            } else if job.status == JobStatus::Pending {
                CommandResult {
                    lines: vec![TerminalLine {
                        kind: TerminalKind::Warning,
                        text: format!(
                            "srun: job {} queued and waiting for resources ({})",
                            job_id.0,
                            job.pending_reason.display_name()
                        ),
                    }],
                    state_changed: true,
                }
            } else {
                CommandResult::stdout(format!("srun: submitted job {}", job_id.0))
            }
        }
        Err(error) => CommandResult::error(format!("srun: error: {error}")),
    }
}

fn scancel(world: &mut SimulationWorld, job_id: Option<&str>) -> CommandResult {
    let Some(job_id) = job_id.and_then(|value| value.parse::<u64>().ok()).map(JobId) else {
        return CommandResult::error("scancel: error: valid job ID required");
    };
    match world.cancel_job(job_id) {
        Ok(()) => CommandResult { lines: vec![], state_changed: true },
        Err(error) => CommandResult::error(format!("scancel: error: {error}")),
    }
}

fn scontrol(world: &SimulationWorld, args: &[String]) -> CommandResult {
    if args.len() < 3 || args[0] != "show" {
        return CommandResult::error("scontrol: supported forms: show job <id>, show node <id>");
    }
    match args[1].as_str() {
        "job" => {
            let Some(job_id) = args[2].parse::<u64>().ok().map(JobId) else {
                return CommandResult::error("scontrol: invalid job ID");
            };
            let Some(job) = world.jobs.get(&job_id) else {
                return CommandResult::error(format!("slurm_load_jobs error: Invalid job id specified: {}", job_id.0));
            };
            CommandResult::stdout(format!(
                "JobId={} JobName={} UserId={} Account={} QOS={}\n   JobState={:?} Reason={} Partition={}\n   NumCPUs={} ReqMem={}M TresPerNode=gres/gpu:{}\n   NodeList={} RunTime={} TimeLimit={}",
                job.id.0,
                job.spec.name,
                job.spec.user,
                job.spec.account,
                job.spec.qos,
                job.status,
                job.pending_reason.display_name(),
                job.spec.partition,
                job.spec.resources.cpus,
                job.spec.resources.memory_mib,
                job.spec.resources.gpus,
                job.allocation.as_ref().map(|allocation| allocation.node_id.as_str()).unwrap_or("(null)"),
                format_duration(job.elapsed_ms(world.now)),
                format_duration(job.spec.time_limit_ms)
            ))
        }
        "node" => {
            let node_id = &args[2];
            let Some(node) = world.cluster.nodes.get(node_id) else {
                return CommandResult::error(format!("slurm_load_node error: Invalid node name specified: {node_id}"));
            };
            CommandResult::stdout(format!(
                "NodeName={} State={:?} CPUs={} AllocCPUs={} RealMemory={} AllocMem={} Gres=gpu:h200:{}",
                node.id,
                node.status,
                node.capacity.cpus,
                node.allocated.cpus,
                node.capacity.memory_mib,
                node.allocated.memory_mib,
                node.capacity.gpus
            ))
        }
        _ => CommandResult::error("scontrol: supported entities: job, node"),
    }
}

fn sacct(world: &SimulationWorld, args: &[String]) -> CommandResult {
    let requested = option_value(args, "-j")
        .or_else(|| option_value(args, "--jobs"))
        .and_then(|value| value.parse::<u64>().ok())
        .map(JobId);
    let mut output = vec![format!(
        "{:<10} {:<16} {:<10} {:<12} {:<10} {}",
        "JobID", "JobName", "Account", "State", "Elapsed", "ExitCode"
    )];
    for job in world.jobs.values() {
        if requested.is_some_and(|job_id| job_id != job.id) {
            continue;
        }
        output.push(format!(
            "{:<10} {:<16} {:<10} {:<12} {:<10} {}:{}",
            job.id.0,
            truncate(&job.spec.name, 16),
            truncate(&job.spec.account, 10),
            format!("{:?}", job.status).to_uppercase(),
            format_duration(job.elapsed_ms(world.now)),
            job.exit_code.map(|code| code.0).unwrap_or(0),
            job.exit_code.map(|code| code.1).unwrap_or(0)
        ));
    }
    CommandResult::stdout(output.join("\n"))
}

fn nvidia_smi(world: &SimulationWorld, shell: &ShellSession, args: &[String]) -> CommandResult {
    let Some(job_id) = shell.active_job_id else {
        return CommandResult::error("NVIDIA-SMI has failed because no GPU allocation is active in this simulated shell.");
    };
    let Some(job) = world.jobs.get(&job_id) else {
        return CommandResult::error("NVIDIA-SMI: simulated allocation no longer exists");
    };
    let Some(allocation) = job.allocation.as_ref() else {
        return CommandResult::error("NVIDIA-SMI: job has no GPU allocation");
    };
    if args.iter().any(|arg| arg == "-L") {
        let lines: Vec<String> = allocation
            .gpu_indices
            .iter()
            .enumerate()
            .map(|(logical, physical)| {
                format!("GPU {logical}: H200 (UUID: SIM-GPU-{physical:04})")
            })
            .collect();
        return CommandResult::stdout(lines.join("\n"));
    }
    CommandResult::stdout(format!(
        "DGX Lab simulated NVIDIA-SMI\nAllocated GPUs: {}\nCUDA_VISIBLE_DEVICES={}",
        allocation.gpu_indices.len(),
        (0..allocation.gpu_indices.len()).map(|index| index.to_string()).collect::<Vec<_>>().join(",")
    ))
}

fn module(shell: &mut ShellSession, args: &[String]) -> CommandResult {
    match args.first().map(String::as_str) {
        Some("load") => {
            for module in &args[1..] {
                if !shell.loaded_modules.contains(module) {
                    shell.loaded_modules.push(module.clone());
                }
            }
            CommandResult { lines: vec![], state_changed: true }
        }
        Some("purge") => {
            shell.loaded_modules.clear();
            CommandResult { lines: vec![], state_changed: true }
        }
        Some("list") => CommandResult::stdout(shell.loaded_modules.join("\n")),
        Some("avail") | None => CommandResult::stdout("singularity/4.5.0\ncuda/12.8\nnccl/2.25\npytorch/2.11"),
        _ => CommandResult::error("module: supported operations: avail, load, list, purge"),
    }
}

fn simulated_runtime(
    world: &mut SimulationWorld,
    shell: &mut ShellSession,
    command: &str,
    args: &[String],
) -> CommandResult {
    if shell.active_job_id.is_some() {
        return CommandResult::stdout(format!(
            "DGX Lab simulated {command} workload accepted inside allocation. No host process was executed.\nArguments: {}",
            args.join(" ")
        ));
    }
    let full = std::iter::once(command.to_string()).chain(args.iter().cloned()).collect::<Vec<_>>();
    srun(world, shell, &full, false)
}

fn exit_allocation(world: &mut SimulationWorld, shell: &mut ShellSession) -> CommandResult {
    let Some(job_id) = shell.active_job_id else {
        return CommandResult::stdout("logout");
    };
    match world.complete_interactive_job(job_id) {
        Ok(()) => {
            shell.active_job_id = None;
            shell.hostname = "dgx-login-01".into();
            shell.env.remove("SLURM_JOB_ID");
            shell.env.remove("CUDA_VISIBLE_DEVICES");
            CommandResult {
                lines: vec![TerminalLine { kind: TerminalKind::System, text: format!("exit: released allocation {}", job_id.0) }],
                state_changed: true,
            }
        }
        Err(error) => CommandResult::error(format!("exit: {error}")),
    }
}

pub fn job_spec_from_script(script: &str, user: &str) -> Result<JobSpec, ShellError> {
    Ok(job_spec_from_script_with_array(script, user)?.0)
}

pub fn job_spec_from_script_with_array(
    script: &str,
    user: &str,
) -> Result<(JobSpec, Vec<u32>), ShellError> {
    let mut args = Vec::new();
    let mut commands = Vec::new();
    for line in script.lines() {
        let trimmed = line.trim();
        if let Some(directive) = trimmed.strip_prefix("#SBATCH") {
            args.extend(tokenize(directive.trim()));
        } else if !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.starts_with("module ")
        {
            commands.push(trimmed.trim_end_matches('\\').trim().to_string());
        }
    }
    let (mut spec, array_tasks) = job_spec_from_args_with_array(&args, user)?;
    spec.command = if commands.is_empty() {
        "bash".into()
    } else {
        commands.join(" ")
    };
    spec.workload_id = infer_workload_id(&spec.command, spec.resources.gpus);
    Ok((spec, array_tasks))
}

pub fn job_spec_from_args(args: &[String], user: &str) -> Result<JobSpec, ShellError> {
    Ok(job_spec_from_args_with_array(args, user)?.0)
}

pub fn job_spec_from_args_with_array(
    args: &[String],
    user: &str,
) -> Result<(JobSpec, Vec<u32>), ShellError> {
    let mut spec = JobSpec { user: user.into(), ..JobSpec::default() };
    let mut command = Vec::new();
    let mut array_tasks = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if token == "--pty" {
            index += 1;
            continue;
        }
        if let Some(value) = token.strip_prefix("--job-name=") {
            spec.name = value.into();
        } else if token == "--job-name" {
            index += 1;
            spec.name = args.get(index).ok_or_else(|| ShellError::MissingValue(token.clone()))?.clone();
        } else if let Some(value) = token.strip_prefix("--partition=") {
            spec.partition = value.into();
        } else if token == "--partition" || token == "-p" {
            index += 1;
            spec.partition = args.get(index).ok_or_else(|| ShellError::MissingValue(token.clone()))?.clone();
        } else if let Some(value) = token.strip_prefix("--cpus-per-task=") {
            spec.resources.cpus = value.parse().map_err(|_| ShellError::InvalidNumber(value.into()))?;
        } else if token == "--cpus-per-task" || token == "-c" {
            index += 1;
            let value = args.get(index).ok_or_else(|| ShellError::MissingValue(token.clone()))?;
            spec.resources.cpus = value.parse().map_err(|_| ShellError::InvalidNumber(value.clone()))?;
        } else if let Some(value) = token.strip_prefix("--mem=") {
            spec.resources.memory_mib = parse_memory_mib(value)?;
        } else if token == "--mem" {
            index += 1;
            let value = args.get(index).ok_or_else(|| ShellError::MissingValue(token.clone()))?;
            spec.resources.memory_mib = parse_memory_mib(value)?;
        } else if let Some(value) = token.strip_prefix("--time=") {
            spec.time_limit_ms = parse_duration_ms(value)?;
        } else if token == "--time" || token == "-t" {
            index += 1;
            let value = args.get(index).ok_or_else(|| ShellError::MissingValue(token.clone()))?;
            spec.time_limit_ms = parse_duration_ms(value)?;
        } else if let Some(value) = token.strip_prefix("--gres=") {
            parse_gres(value, &mut spec.resources)?;
        } else if token == "--gres" {
            index += 1;
            let value = args.get(index).ok_or_else(|| ShellError::MissingValue(token.clone()))?;
            parse_gres(value, &mut spec.resources)?;
        } else if let Some(value) = token.strip_prefix("--gpus=") {
            spec.resources.gpus = value.parse().map_err(|_| ShellError::InvalidNumber(value.into()))?;
            spec.resources.gpu_type = Some("h200".into());
        } else if token == "--gpus" || token == "-G" {
            index += 1;
            let value = args.get(index).ok_or_else(|| ShellError::MissingValue(token.clone()))?;
            spec.resources.gpus = value.parse().map_err(|_| ShellError::InvalidNumber(value.clone()))?;
            spec.resources.gpu_type = Some("h200".into());
        } else if token == "--account" || token == "-A" {
            index += 1;
            spec.account = args.get(index).ok_or_else(|| ShellError::MissingValue(token.clone()))?.clone();
        } else if let Some(value) = token.strip_prefix("--account=") {
            spec.account = value.into();
        } else if token == "--qos" {
            index += 1;
            spec.qos = args.get(index).ok_or_else(|| ShellError::MissingValue(token.clone()))?.clone();
        } else if let Some(value) = token.strip_prefix("--qos=") {
            spec.qos = value.into();
        } else if let Some(value) = token.strip_prefix("--output=") {
            spec.output_path = Some(value.into());
        } else if token == "--output" || token == "-o" {
            index += 1;
            spec.output_path = Some(
                args.get(index).ok_or_else(|| ShellError::MissingValue(token.clone()))?.clone(),
            );
        } else if let Some(value) = token.strip_prefix("--error=") {
            spec.error_path = Some(value.into());
        } else if token == "--error" || token == "-e" {
            index += 1;
            spec.error_path = Some(
                args.get(index).ok_or_else(|| ShellError::MissingValue(token.clone()))?.clone(),
            );
        } else if token == "--dependency" {
            index += 1;
            let value = args.get(index).ok_or_else(|| ShellError::MissingValue(token.clone()))?;
            spec.dependency_after_ok = parse_dependency(value)?;
        } else if let Some(value) = token.strip_prefix("--dependency=") {
            spec.dependency_after_ok = parse_dependency(value)?;
        } else if let Some(value) = token.strip_prefix("--array=") {
            array_tasks = parse_array_spec(value)?;
        } else if token == "--array" {
            index += 1;
            let value = args
                .get(index)
                .ok_or_else(|| ShellError::MissingValue(token.clone()))?;
            array_tasks = parse_array_spec(value)?;
        } else if token.starts_with('-') {
            return Err(ShellError::UnsupportedOption(token.clone()));
        } else {
            command.extend_from_slice(&args[index..]);
            break;
        }
        index += 1;
    }
    spec.command = if command.is_empty() { "bash".into() } else { command.join(" ") };
    spec.workload_id = infer_workload_id(&spec.command, spec.resources.gpus);
    Ok((spec, array_tasks))
}

/// Parse a teaching subset of Slurm array specs: `1-3`, `1-3:2`, `1,3,5` (optional `%N` concurrency ignored).
fn parse_array_spec(value: &str) -> Result<Vec<u32>, ShellError> {
    let value = value.split('%').next().unwrap_or(value).trim();
    if value.is_empty() {
        return Err(ShellError::InvalidNumber(value.into()));
    }
    let mut tasks = Vec::new();
    for part in value.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((start, rest)) = part.split_once('-') {
            let start: u32 = start
                .parse()
                .map_err(|_| ShellError::InvalidNumber(part.into()))?;
            let (end, step) = if let Some((end, step)) = rest.split_once(':') {
                (
                    end.parse::<u32>()
                        .map_err(|_| ShellError::InvalidNumber(part.into()))?,
                    step.parse::<u32>()
                        .map_err(|_| ShellError::InvalidNumber(part.into()))?
                        .max(1),
                )
            } else {
                (
                    rest.parse::<u32>()
                        .map_err(|_| ShellError::InvalidNumber(part.into()))?,
                    1,
                )
            };
            if end < start {
                return Err(ShellError::InvalidNumber(part.into()));
            }
            let mut current = start;
            while current <= end {
                tasks.push(current);
                current = current.saturating_add(step);
                if tasks.len() > 64 {
                    return Err(ShellError::InvalidNumber(
                        "array too large for DGX Lab teaching profile".into(),
                    ));
                }
            }
        } else {
            tasks.push(
                part.parse()
                    .map_err(|_| ShellError::InvalidNumber(part.into()))?,
            );
        }
    }
    if tasks.is_empty() {
        return Err(ShellError::InvalidNumber(value.into()));
    }
    Ok(tasks)
}

fn infer_workload_id(command: &str, gpus: u16) -> String {
    if command == "bash" {
        "interactive-shell-v1".into()
    } else if command.contains("torchrun") || gpus > 1 {
        "torchrun-multigpu-v1".into()
    } else if command.contains("preprocess") {
        "cpu-preprocess-v1".into()
    } else {
        "pytorch-training-v1".into()
    }
}

fn parse_gres(value: &str, resources: &mut Tres) -> Result<(), ShellError> {
    let parts: Vec<&str> = value.split(':').collect();
    match parts.as_slice() {
        ["gpu", count] => {
            resources.gpu_type = Some("h200".into());
            resources.gpus = count.parse().map_err(|_| ShellError::InvalidNumber((*count).into()))?;
        }
        ["gpu", gpu_type, count] => {
            resources.gpu_type = Some((*gpu_type).into());
            resources.gpus = count.parse().map_err(|_| ShellError::InvalidNumber((*count).into()))?;
        }
        _ => return Err(ShellError::InvalidGres(value.into())),
    }
    Ok(())
}

fn parse_dependency(value: &str) -> Result<Option<JobId>, ShellError> {
    let value = value.strip_prefix("afterok:").ok_or_else(|| ShellError::UnsupportedDependency(value.into()))?;
    let id = value.parse().map_err(|_| ShellError::InvalidNumber(value.into()))?;
    Ok(Some(JobId(id)))
}

pub fn parse_memory_mib(value: &str) -> Result<u64, ShellError> {
    let value = value.trim();
    let split = value.find(|character: char| !character.is_ascii_digit()).unwrap_or(value.len());
    let number: u64 = value[..split].parse().map_err(|_| ShellError::InvalidNumber(value.into()))?;
    let suffix = value[split..].to_ascii_uppercase();
    match suffix.as_str() {
        "" | "M" | "MB" => Ok(number),
        "G" | "GB" => Ok(number.saturating_mul(1024)),
        "T" | "TB" => Ok(number.saturating_mul(1024 * 1024)),
        _ => Err(ShellError::InvalidMemory(value.into())),
    }
}

pub fn parse_duration_ms(value: &str) -> Result<u64, ShellError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ShellError::InvalidDuration(value.into()));
    }

    // Convenience suffix retained for course authoring. Learner-facing Slurm
    // examples otherwise follow the documented input forms:
    // minutes, minutes:seconds, hours:minutes:seconds, days-hours,
    // days-hours:minutes, or days-hours:minutes:seconds.
    if let Some(minutes) = value.strip_suffix('m') {
        return minutes
            .parse::<u64>()
            .map(|minutes| minutes.saturating_mul(60_000))
            .map_err(|_| ShellError::InvalidDuration(value.into()));
    }

    let (days, clock, has_day_prefix) = match value.split_once('-') {
        Some((days, clock)) if !days.is_empty() && !clock.is_empty() => {
            let days = days
                .parse::<u64>()
                .map_err(|_| ShellError::InvalidDuration(value.into()))?;
            (days, clock, true)
        }
        Some(_) => return Err(ShellError::InvalidDuration(value.into())),
        None => (0, value, false),
    };

    let parts: Vec<&str> = clock.split(':').collect();
    let (hours, minutes, seconds) = if has_day_prefix {
        match parts.as_slice() {
            [hours] => (parse_time_component(hours, value)?, 0, 0),
            [hours, minutes] => (
                parse_time_component(hours, value)?,
                parse_bounded_time_component(minutes, 59, value)?,
                0,
            ),
            [hours, minutes, seconds] => (
                parse_time_component(hours, value)?,
                parse_bounded_time_component(minutes, 59, value)?,
                parse_bounded_time_component(seconds, 59, value)?,
            ),
            _ => return Err(ShellError::InvalidDuration(value.into())),
        }
    } else {
        match parts.as_slice() {
            [minutes] => (0, parse_time_component(minutes, value)?, 0),
            [minutes, seconds] => (
                0,
                parse_time_component(minutes, value)?,
                parse_bounded_time_component(seconds, 59, value)?,
            ),
            [hours, minutes, seconds] => (
                parse_time_component(hours, value)?,
                parse_bounded_time_component(minutes, 59, value)?,
                parse_bounded_time_component(seconds, 59, value)?,
            ),
            _ => return Err(ShellError::InvalidDuration(value.into())),
        }
    };

    let total_seconds = days
        .saturating_mul(86_400)
        .saturating_add(hours.saturating_mul(3_600))
        .saturating_add(minutes.saturating_mul(60))
        .saturating_add(seconds);
    Ok(total_seconds.saturating_mul(1_000))
}

fn parse_time_component(component: &str, original: &str) -> Result<u64, ShellError> {
    if component.is_empty() || !component.chars().all(|character| character.is_ascii_digit()) {
        return Err(ShellError::InvalidDuration(original.into()));
    }
    component
        .parse::<u64>()
        .map_err(|_| ShellError::InvalidDuration(original.into()))
}

fn parse_bounded_time_component(
    component: &str,
    maximum: u64,
    original: &str,
) -> Result<u64, ShellError> {
    let value = parse_time_component(component, original)?;
    if value > maximum {
        return Err(ShellError::InvalidDuration(original.into()));
    }
    Ok(value)
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    for character in input.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            } else {
                current.push(character);
            }
            continue;
        }
        if character == '\'' || character == '"' {
            quote = Some(character);
        } else if character.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(character);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn resolve_path(cwd: &str, path: &str) -> Result<String, VfsError> {
    if path.starts_with('/') {
        normalize_path(path)
    } else if path == "." {
        normalize_path(cwd)
    } else {
        normalize_path(&format!("{cwd}/{path}"))
    }
}

/// Resolve a learner-relative virtual path for session/editor use.
pub fn resolve_path_for_session(shell: &ShellSession, path: &str) -> Result<String, String> {
    resolve_path(&shell.cwd, path).map_err(|error| error.to_string())
}

fn option_value(args: &[String], option: &str) -> Option<String> {
    for (index, token) in args.iter().enumerate() {
        if token == option {
            return args.get(index + 1).cloned();
        }
        if let Some(value) = token.strip_prefix(&format!("{option}=")) {
            return Some(value.into());
        }
    }
    None
}

fn format_duration(milliseconds: u64) -> String {
    let total_seconds = milliseconds / 1_000;
    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    if days > 0 {
        format!("{days}-{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }
}

fn truncate(value: &str, width: usize) -> String {
    value.chars().take(width).collect()
}

fn display_cwd(cwd: &str, user: &str) -> String {
    let home = format!("/home/{user}");
    cwd.strip_prefix(&home).map(|rest| format!("~{rest}")).unwrap_or_else(|| cwd.into())
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ShellError {
    #[error("missing value for {0}")]
    MissingValue(String),
    #[error("invalid number: {0}")]
    InvalidNumber(String),
    #[error("invalid memory specification: {0}")]
    InvalidMemory(String),
    #[error("invalid duration: {0}")]
    InvalidDuration(String),
    #[error("invalid GRES specification: {0}")]
    InvalidGres(String),
    #[error("unsupported option: {0}")]
    UnsupportedOption(String),
    #[error("unsupported dependency: {0}")]
    UnsupportedDependency(String),
    #[error(transparent)]
    Vfs(#[from] VfsError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_documented_slurm_time_forms() {
        assert_eq!(parse_duration_ms("30").unwrap(), 30 * 60_000);
        assert_eq!(parse_duration_ms("30:15").unwrap(), (30 * 60 + 15) * 1_000);
        assert_eq!(parse_duration_ms("01:30:15").unwrap(), (3600 + 30 * 60 + 15) * 1_000);
        assert_eq!(parse_duration_ms("2-03").unwrap(), (2 * 86_400 + 3 * 3_600) * 1_000);
        assert_eq!(parse_duration_ms("0-01:00").unwrap(), 3_600_000);
        assert_eq!(
            parse_duration_ms("2-03:04:05").unwrap(),
            (2 * 86_400 + 3 * 3_600 + 4 * 60 + 5) * 1_000
        );
    }

    #[test]
    fn rejects_invalid_slurm_time_forms() {
        assert!(parse_duration_ms("").is_err());
        assert!(parse_duration_ms("00:60").is_err());
        assert!(parse_duration_ms("01:02:60").is_err());
        assert!(parse_duration_ms("1-").is_err());
        assert!(parse_duration_ms("1-02:03:04:05").is_err());
    }

    #[test]
    fn parses_one_h200_gpu() {
        let args = tokenize("--gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:30:00 --pty bash");
        let spec = job_spec_from_args(&args, "learner").unwrap();
        assert_eq!(spec.resources.gpus, 1);
        assert_eq!(spec.resources.memory_mib, 65_536);
        assert_eq!(spec.time_limit_ms, 1_800_000);
    }

    #[test]
    fn batch_script_maps_to_synthetic_workload() {
        let script = r#"
#SBATCH --gres=gpu:h200:1
#SBATCH --cpus-per-task=8
#SBATCH --mem=64G
python train.py --batch-size 64 --epochs 5
"#;
        let spec = job_spec_from_script(script, "learner").unwrap();
        assert_eq!(spec.workload_id, "pytorch-training-v1");
        assert_eq!(spec.resources.gpus, 1);
    }

    #[test]
    fn default_training_script_accepts_output_directive() {
        let world = SimulationWorld::dgx_h200_8(7);
        let script = world.fs.read_text("/home/learner/train.sbatch").unwrap();
        let spec = job_spec_from_script(&script, "learner").unwrap();
        assert_eq!(spec.output_path.as_deref(), Some("logs/%x-%j.out"));
        assert_eq!(spec.resources.gpus, 1);
    }

    #[test]
    fn shell_executes_guided_allocation_path() {
        let mut world = SimulationWorld::dgx_h200_8(7);
        let mut shell = ShellSession::learner();
        let result = execute_line(
            &mut world,
            &mut shell,
            "srun --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:30:00 --pty bash",
        );
        assert!(result.lines[0].text.contains("Granted job allocation"));
        assert!(shell.active_job_id.is_some());
        let gpu = execute_line(&mut world, &mut shell, "nvidia-smi -L");
        assert!(gpu.lines[0].text.contains("GPU 0: H200"));
    }
}
