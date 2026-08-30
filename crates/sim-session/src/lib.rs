#![forbid(unsafe_code)]

//! Pure simulation session used by native tests, the WASM worker, and the UI bridge.

mod course;
mod lab04;
mod lab06;
mod lab07;
mod lab09;
mod pending;

pub mod cert_bank;

use course::{COURSE_LABS, generic_hints, generic_lab_checks, lab_for_scenario, lab_step_meta};
use dgxlab_contracts::{
    SIMULATOR_COMPATIBILITY_VERSION, SessionId, SimRequest, SimResponse, TerminalLine, UiGpuTile,
    UiJobSummary, UiLabStep, UiWorldView, WORKER_PROTOCOL_VERSION,
};
use grading::{EvidenceLedger, PracticalCheck, evaluate_practical};
use lab04::{LAB04_ID, lab04_checks, lab04_hints};
use lab06::{LAB06_ID, lab06_checks, lab06_hints};
use lab07::{LAB07_ID, lab07_checks, lab07_hints};
use lab09::{LAB09_ID, lab09_checks, lab09_hints};
use pending::explain_pending;
use persistence_codec::SessionBundle;
use scenarios::initialize_scenario;
use serde::{Deserialize, Serialize};
use sim_core::{SimulationWorld, WorldEventKind};
use slurm_model::JobStatus;
use std::collections::{BTreeMap, BTreeSet};
use virtual_shell::{CommandResult, ShellSession, execute_line};

pub use course::{
    COURSE_LABS as BUILTIN_LABS, LabMeta, LabStepMeta, lab_meta, lab_step_meta as learner_step_meta,
};

/// Authoritative learner session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimSession {
    world: SimulationWorld,
    shell: ShellSession,
    ledger: EvidenceLedger,
    seq: u64,
    scenario_id: String,
    seed: u64,
    hint_level: u8,
    lab_id: String,
}

impl SimSession {
    pub fn new(scenario_id: &str, seed: u64) -> Result<Self, SessionError> {
        let world = initialize_scenario(scenario_id, seed)?;
        let lab_id = lab_for_scenario(scenario_id).unwrap_or("").to_string();
        Ok(Self {
            world,
            shell: ShellSession::learner(),
            ledger: EvidenceLedger::new(),
            seq: 0,
            scenario_id: scenario_id.into(),
            seed,
            hint_level: 0,
            lab_id,
        })
    }

    /// Open a lab by course id (maps to scenario).
    pub fn open_lab(lab_id: &str, seed: u64) -> Result<Self, SessionError> {
        let meta = COURSE_LABS
            .iter()
            .find(|lab| lab.id == lab_id)
            .ok_or_else(|| SessionError::Codec(format!("unknown lab {lab_id}")))?;
        let mut session = Self::new(meta.scenario, seed)?;
        session.lab_id = meta.id.into();
        Ok(session)
    }

    #[must_use]
    pub fn seq(&self) -> u64 {
        self.seq
    }

    #[must_use]
    pub fn state_digest(&self) -> String {
        self.world.state_digest()
    }

    #[must_use]
    pub fn practical_percent(&self) -> u8 {
        self.view().practical_percent
    }

    #[must_use]
    pub fn critical_practical_passed(&self) -> bool {
        let checks = checks_for_lab(&self.lab_id);
        if checks.is_empty() {
            return true;
        }
        evaluate_practical(&checks, &self.world, &self.shell, &self.ledger, "learner")
            .all_critical_passed
    }

    #[must_use]
    pub fn view(&self) -> UiWorldView {
        build_view(self)
    }

    pub fn export_bundle(&self) -> SessionBundle {
        SessionBundle::new(
            SessionId(format!("local-{}", self.seed)),
            env!("CARGO_PKG_VERSION"),
            "1.0.0",
            self.world.clone(),
            self.shell.clone(),
            self.ledger.clone(),
        )
    }

    pub fn export_json(&self) -> Result<String, SessionError> {
        serde_json::to_string(self).map_err(|error| SessionError::Codec(error.to_string()))
    }

    pub fn import_json(json: &str) -> Result<Self, SessionError> {
        serde_json::from_str(json).map_err(|error| SessionError::Codec(error.to_string()))
    }

    pub fn handle(&mut self, request: SimRequest) -> SimResponse {
        match request {
            SimRequest::Initialize { scenario_id, seed } => match Self::new(&scenario_id, seed) {
                Ok(session) => {
                    *self = session;
                    self.seq = self.seq.saturating_add(1);
                    SimResponse::Ready {
                        protocol_version: WORKER_PROTOCOL_VERSION.into(),
                        compatibility_version: SIMULATOR_COMPATIBILITY_VERSION.into(),
                        seq: self.seq,
                        state: self.view(),
                    }
                }
                Err(error) => self.error("init_failed", error.to_string()),
            },
            SimRequest::ExecuteCommand { command } => {
                let command_at = self.world.now;
                let active_job_id = self.shell.active_job_id;
                let existing_job_ids = self.world.jobs.keys().copied().collect::<BTreeSet<_>>();
                let result = execute_line(&mut self.world, &mut self.shell, &command);
                if result
                    .lines
                    .iter()
                    .all(|line| line.kind != dgxlab_contracts::TerminalKind::Stderr)
                {
                    let created_job_ids = self
                        .world
                        .jobs
                        .keys()
                        .filter(|job_id| !existing_job_ids.contains(job_id))
                        .copied()
                        .collect();
                    self.ledger.record_command_with_context(
                        command_at,
                        command.clone(),
                        active_job_id,
                        created_job_ids,
                    );
                }
                self.seq = self.seq.saturating_add(1);
                SimResponse::CommandResult {
                    seq: self.seq,
                    prompt: self.shell.prompt(),
                    lines: decorate_command_lines(&command, result),
                    state: self.view(),
                }
            }
            SimRequest::AdvanceClock { delta_ms } => {
                if let Err(error) = self.world.advance_by(delta_ms) {
                    return self.error("advance_failed", error.to_string());
                }
                self.bump_state()
            }
            SimRequest::SetClockSpeed { multiplier } => {
                self.world.clock_multiplier = multiplier.max(1);
                self.bump_state()
            }
            SimRequest::Pause => {
                self.world.paused = true;
                self.bump_state()
            }
            SimRequest::Resume => {
                self.world.paused = false;
                self.bump_state()
            }
            SimRequest::Reset { scenario_id, seed } => match Self::new(&scenario_id, seed) {
                Ok(mut session) => {
                    session.seq = self.seq.saturating_add(1);
                    let seq = session.seq;
                    let state = session.view();
                    *self = session;
                    SimResponse::State { seq, state }
                }
                Err(error) => self.error("reset_failed", error.to_string()),
            },
            SimRequest::Snapshot => self.bump_state(),
            SimRequest::CancelJob { job_id } => {
                if let Err(error) = self.world.cancel_job(dgxlab_contracts::JobId(job_id)) {
                    return self.error("cancel_failed", error.to_string());
                }
                self.bump_state()
            }
            SimRequest::UseHint => {
                let hints = hints_for_lab(&self.lab_id);
                if self.hint_level < hints.len() as u8 {
                    self.hint_level = self.hint_level.saturating_add(1);
                    self.ledger.events.push(grading::EvidenceEvent::HintUsed {
                        at: self.world.now,
                        hint_id: format!("{}-hint-{}", self.lab_id, self.hint_level),
                        level: self.hint_level,
                    });
                }
                self.bump_state()
            }
            SimRequest::ReadVfs { path } => {
                let resolved = match virtual_shell::resolve_path_for_session(&self.shell, &path) {
                    Ok(path) => path,
                    Err(message) => return self.error("vfs_path", message),
                };
                match self.world.fs.read_text(&resolved) {
                    Ok(content) => {
                        self.seq = self.seq.saturating_add(1);
                        SimResponse::FileContent { seq: self.seq, path: resolved, content }
                    }
                    Err(error) => self.error("vfs_read", error.to_string()),
                }
            }
            SimRequest::WriteVfs { path, content } => {
                let resolved = match virtual_shell::resolve_path_for_session(&self.shell, &path) {
                    Ok(path) => path,
                    Err(message) => return self.error("vfs_path", message),
                };
                match self.world.fs.write_file(&resolved, content.as_bytes()) {
                    Ok(()) => {
                        self.ledger.events.push(grading::EvidenceEvent::FileWritten {
                            at: self.world.now,
                            path: resolved,
                        });
                        self.bump_state()
                    }
                    Err(error) => self.error("vfs_write", error.to_string()),
                }
            }
        }
    }

    fn bump_state(&mut self) -> SimResponse {
        self.seq = self.seq.saturating_add(1);
        SimResponse::State { seq: self.seq, state: self.view() }
    }

    fn error(&mut self, code: &str, message: String) -> SimResponse {
        self.seq = self.seq.saturating_add(1);
        SimResponse::Error { code: code.into(), message, seq: self.seq }
    }
}

fn hints_for_lab(lab_id: &str) -> Vec<&'static str> {
    match lab_id {
        LAB04_ID => lab04_hints().to_vec(),
        LAB06_ID => lab06_hints().to_vec(),
        LAB07_ID => lab07_hints().to_vec(),
        LAB09_ID => lab09_hints().to_vec(),
        other => generic_hints(other),
    }
}

fn checks_for_lab(lab_id: &str) -> Vec<PracticalCheck> {
    match lab_id {
        LAB04_ID => lab04_checks(),
        LAB06_ID => lab06_checks(),
        LAB07_ID => lab07_checks(),
        LAB09_ID => lab09_checks(),
        other if !other.is_empty() => generic_lab_checks(other),
        _ => Vec::new(),
    }
}

fn decorate_command_lines(command: &str, result: CommandResult) -> Vec<TerminalLine> {
    let mut lines = vec![TerminalLine::input(command.to_string())];
    lines.extend(result.lines);
    lines
}

fn learner_checkpoint_paths(world: &SimulationWorld) -> Vec<String> {
    let checkpoint_root = "/home/learner/checkpoints/";
    let mut artifact_order = BTreeMap::new();
    for event in &world.event_log {
        if let WorldEventKind::ArtifactWritten { path, .. } = &event.kind
            && path.starts_with(checkpoint_root)
        {
            artifact_order
                .entry(path.clone())
                .and_modify(|order: &mut (dgxlab_contracts::SimTimeMs, u64)| {
                    *order = (*order).max((event.at, event.sequence));
                })
                .or_insert((event.at, event.sequence));
        }
    }

    let mut paths = world
        .fs
        .list_dir("/home/learner/checkpoints")
        .unwrap_or_default()
        .into_iter()
        .map(|name| format!("checkpoints/{name}"))
        .filter(|path| world.fs.read_file(&format!("/home/learner/{path}")).is_ok())
        .collect::<Vec<_>>();
    paths.sort_by(|left, right| {
        let left_path = format!("/home/learner/{left}");
        let right_path = format!("/home/learner/{right}");
        match (artifact_order.get(&left_path), artifact_order.get(&right_path)) {
            (Some(left_order), Some(right_order)) => {
                left_order.cmp(right_order).then_with(|| left.cmp(right))
            }
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, None) => left.cmp(right),
        }
    });
    paths
}

fn build_view(session: &SimSession) -> UiWorldView {
    let world = &session.world;
    let shell = &session.shell;
    let compute = world
        .cluster
        .nodes
        .values()
        .find(|node| node.id != "dgx-login-01")
        .or_else(|| world.cluster.nodes.values().next());

    let gpus = compute
        .map(|node| {
            node.gpus
                .iter()
                .map(|gpu| {
                    let status = if gpu.allocated_to.is_some() {
                        "Allocated"
                    } else {
                        match gpu.health {
                            slurm_model::GpuHealth::Ok => "Idle",
                            slurm_model::GpuHealth::Warning => "Warning",
                            slurm_model::GpuHealth::Failed => "Failed",
                        }
                    };
                    UiGpuTile {
                        index: gpu.index,
                        model: gpu.model.clone(),
                        status: status.into(),
                        owner_job_id: gpu.allocated_to.map(|id| id.0),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let jobs = world
        .jobs
        .values()
        .map(|job| {
            let pending = job.status == JobStatus::Pending;
            UiJobSummary {
                id: job.id.0,
                name: job.spec.name.clone(),
                user: job.spec.user.clone(),
                status: format!("{:?}", job.status).to_uppercase(),
                pending_reason: pending.then(|| job.pending_reason.display_name().into()),
                pending_explanation: pending
                    .then(|| explain_pending(job.pending_reason).to_string()),
                gpus: job.spec.resources.gpus,
                cpus: job.spec.resources.cpus,
                memory_mib: job.spec.resources.memory_mib,
            }
        })
        .collect();
    let checkpoint_paths = learner_checkpoint_paths(world);

    let (lab_steps, practical_percent, lab_complete) = lab_progress(session);
    let hints = hints_for_lab(&session.lab_id);
    let hint_text = if session.hint_level == 0 {
        None
    } else {
        hints.get(session.hint_level as usize - 1).map(|text| (*text).into())
    };

    UiWorldView {
        lab_id: session.lab_id.clone(),
        scenario_id: session.scenario_id.clone(),
        seed: session.seed,
        now_ms: world.now.0,
        paused: world.paused,
        clock_multiplier: world.clock_multiplier,
        state_digest: world.state_digest(),
        prompt: shell.prompt(),
        gpus,
        jobs,
        checkpoint_paths,
        node_status: compute
            .map(|node| format!("{:?}", node.status).to_lowercase())
            .unwrap_or_else(|| "unknown".into()),
        lab_steps,
        hint_level: session.hint_level,
        hint_text,
        lab_complete,
        practical_percent,
    }
}

fn lab_progress(session: &SimSession) -> (Vec<UiLabStep>, u8, bool) {
    let checks = checks_for_lab(&session.lab_id);
    if checks.is_empty() {
        return (Vec::new(), 0, false);
    }
    let score =
        evaluate_practical(&checks, &session.world, &session.shell, &session.ledger, "learner");
    let steps = score
        .results
        .iter()
        .map(|result| UiLabStep {
            id: result.id.clone(),
            label: lab_step_meta(&session.lab_id, &result.id)
                .map(|step| step.label.to_string())
                .unwrap_or_else(|| result.id.replace('-', " ")),
            complete: result.passed,
            critical: result.critical,
        })
        .collect::<Vec<_>>();
    let percent =
        score.earned_points.saturating_mul(100).checked_div(score.possible_points).unwrap_or(0)
            as u8;
    let complete = score.all_critical_passed && percent >= 80;
    (steps, percent, complete)
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error(transparent)]
    Scenario(#[from] scenarios::ScenarioError),
    #[error("session codec error: {0}")]
    Codec(String),
}

pub fn digest_for_transcript(
    scenario_id: &str,
    seed: u64,
    commands: &[&str],
) -> Result<String, SessionError> {
    let mut session = SimSession::new(scenario_id, seed)?;
    for command in commands {
        let _ = session.handle(SimRequest::ExecuteCommand { command: (*command).into() });
    }
    Ok(session.state_digest())
}

pub fn lab04_completes(seed: u64) -> Result<(bool, u8, String), SessionError> {
    let commands = [
        "sinfo",
        "srun --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:30:00 --pty bash",
        "echo $SLURM_JOB_ID",
        "echo $CUDA_VISIBLE_DEVICES",
        "nvidia-smi -L",
        "exit",
        "sacct",
    ];
    let mut session = SimSession::new("guided-one-gpu", seed)?;
    for command in commands {
        let _ = session.handle(SimRequest::ExecuteCommand { command: command.into() });
    }
    let view = session.view();
    Ok((view.lab_complete, view.practical_percent, session.state_digest()))
}

pub fn lab06_batch_path(seed: u64) -> Result<(bool, String), SessionError> {
    let mut session = SimSession::new("dgx-h200-8", seed)?;
    let script = session
        .world
        .fs
        .read_text("/home/learner/train.sbatch")
        .map_err(|error| SessionError::Codec(error.to_string()))?;
    let _ = session.handle(SimRequest::WriteVfs { path: "train.sbatch".into(), content: script });
    let _ = session.handle(SimRequest::ExecuteCommand { command: "sbatch train.sbatch".into() });
    let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 30 * 60 * 1_000 });
    let _ = session.handle(SimRequest::ExecuteCommand {
        command: "tail -n 20 logs/train-h200-10000.out".into(),
    });
    let _ = session.handle(SimRequest::ExecuteCommand { command: "sacct".into() });
    let view = session.view();
    Ok((view.lab_complete || view.practical_percent >= 80, session.state_digest()))
}

pub fn lab07_contention_path(seed: u64) -> Result<(bool, String, String), SessionError> {
    let mut session = SimSession::new("dgx-contended", seed)?;
    let _ = session.handle(SimRequest::ExecuteCommand { command: "sbatch train.sbatch".into() });
    let pending_reason = session
        .view()
        .jobs
        .iter()
        .find(|job| job.user == "learner")
        .and_then(|job| job.pending_reason.clone())
        .unwrap_or_default();
    let _ = session.handle(SimRequest::ExecuteCommand { command: "squeue".into() });
    if let Some(job) = session.view().jobs.iter().find(|job| job.user == "learner") {
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: format!("scontrol show job {}", job.id),
        });
    }
    let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 4 * 60 * 60 * 1_000 });
    let view = session.view();
    let started = view
        .jobs
        .iter()
        .any(|job| job.user == "learner" && (job.status == "RUNNING" || job.status == "COMPLETED"));
    Ok((started, pending_reason, session.state_digest()))
}

pub fn lab09_failure_path(seed: u64) -> Result<(bool, String), SessionError> {
    let mut session = SimSession::new("dgx-degraded", seed)?;
    let _ = session.handle(SimRequest::ExecuteCommand { command: "sacct".into() });
    let _ = session
        .handle(SimRequest::ExecuteCommand { command: "cat checkpoints/epoch-004.pt".into() });
    let _ = session.handle(SimRequest::ExecuteCommand {
        command: "tail -n 20 logs/train-llm-10000.err".into(),
    });
    if let Some(job) = session
        .view()
        .jobs
        .iter()
        .find(|job| job.user == "learner" && job.status == "OUT_OF_MEMORY")
    {
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: format!("scontrol show job {}", job.id),
        });
    }
    let _ = session.handle(SimRequest::ExecuteCommand {
        command: "srun --job-name=train-resume --partition=gpu --gres=gpu:h200:4 --cpus-per-task=16 --mem=64G --time=02:00:00 python train.py --batch-size 64 --epochs 5 --resume-from-checkpoint checkpoints/epoch-004.pt".into(),
    });
    let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 5 * 60 * 1_000 });
    let view = session.view();
    Ok((view.lab_complete || view.practical_percent >= 50, session.state_digest()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct AuthoredLabPath {
        id: String,
        title: String,
        scenario: String,
        estimated_minutes: u16,
        steps: Vec<AuthoredLabStep>,
    }

    #[derive(Deserialize)]
    struct AuthoredLabStep {
        id: String,
        instruction: String,
    }

    const AUTHORED_LABS: &[&str] = &[
        include_str!(
            "../../../course-src/slurm-fundamentals/labs/01-cluster-mental-model/lab.yaml"
        ),
        include_str!("../../../course-src/slurm-fundamentals/labs/02-interactive-cpu/lab.yaml"),
        include_str!("../../../course-src/slurm-fundamentals/labs/03-cpu-memory/lab.yaml"),
        include_str!("../../../course-src/slurm-fundamentals/labs/04-one-gpu/lab.yaml"),
        include_str!("../../../course-src/slurm-fundamentals/labs/05-containers/lab.yaml"),
        include_str!("../../../course-src/slurm-fundamentals/labs/06-batch-jobs/lab.yaml"),
        include_str!("../../../course-src/slurm-fundamentals/labs/07-pending-reasons/lab.yaml"),
        include_str!("../../../course-src/slurm-fundamentals/labs/08-arrays-dependencies/lab.yaml"),
        include_str!("../../../course-src/slurm-fundamentals/labs/09-failure-resume/lab.yaml"),
        include_str!("../../../course-src/slurm-fundamentals/labs/10-multi-gpu/lab.yaml"),
        include_str!("../../../course-src/slurm-fundamentals/labs/11-policy-efficiency/lab.yaml"),
        include_str!("../../../course-src/slurm-fundamentals/labs/12-capstone/lab.yaml"),
    ];

    #[test]
    fn sinfo_returns_authoritative_snapshot() {
        let mut session = SimSession::new("dgx-h200-8", 42).expect("session");
        let response = session.handle(SimRequest::ExecuteCommand { command: "sinfo".into() });
        assert!(matches!(response, SimResponse::CommandResult { .. }));
    }

    #[test]
    fn equal_transcripts_share_digest() {
        let commands = ["sinfo", "squeue"];
        let first = digest_for_transcript("dgx-h200-8", 7, &commands).unwrap();
        let second = digest_for_transcript("dgx-h200-8", 7, &commands).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn lab04_transcript_completes_practical() {
        let (complete, percent, digest) = lab04_completes(42).expect("lab04");
        assert!(complete, "lab should complete");
        assert!(percent >= 80, "percent={percent}");
        let again = lab04_completes(42).expect("lab04 again");
        assert_eq!(digest, again.2);
    }

    #[test]
    fn session_json_round_trip_preserves_digest() {
        let mut session = SimSession::new("guided-one-gpu", 9).unwrap();
        let _ = session.handle(SimRequest::ExecuteCommand { command: "sinfo".into() });
        let json = session.export_json().unwrap();
        let restored = SimSession::import_json(&json).unwrap();
        assert_eq!(session.state_digest(), restored.state_digest());
    }

    #[test]
    fn successful_command_evidence_captures_active_and_created_jobs() {
        let mut session = SimSession::new("dgx-h200-8", 9).unwrap();
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: "srun --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --pty bash".into(),
        });
        let active_job_id = session.shell.active_job_id.expect("interactive allocation");
        let _ = session.handle(SimRequest::ExecuteCommand { command: "nvidia-smi -L".into() });
        let _ = session
            .handle(SimRequest::ExecuteCommand { command: "scontrol show node not-a-node".into() });

        let submitted = session.ledger.events.iter().find(|event| {
            matches!(event, grading::EvidenceEvent::Command { command, .. } if command.starts_with("srun"))
        });
        assert!(matches!(
            submitted,
            Some(grading::EvidenceEvent::Command {
                active_job_id: None,
                created_job_ids,
                ..
            }) if created_job_ids == &vec![active_job_id]
        ));
        let inside = session.ledger.events.iter().find(|event| {
            matches!(event, grading::EvidenceEvent::Command { command, .. } if command == "nvidia-smi -L")
        });
        assert!(matches!(
            inside,
            Some(grading::EvidenceEvent::Command {
                active_job_id: Some(job_id),
                created_job_ids,
                ..
            }) if *job_id == active_job_id && created_job_ids.is_empty()
        ));
        assert!(!session.ledger.events.iter().any(|event| {
            matches!(event, grading::EvidenceEvent::Command { command, .. } if command.contains("not-a-node"))
        }));
    }

    #[test]
    fn array_submission_evidence_records_every_created_job() {
        let mut session = SimSession::new("dgx-h200-8", 9).unwrap();
        let _ = session.handle(SimRequest::WriteVfs {
            path: "array.sbatch".into(),
            content: "#!/bin/bash\n#SBATCH --job-name=sweep\n#SBATCH --array=0-2\npython train.py --epochs 1\n".into(),
        });
        let _ =
            session.handle(SimRequest::ExecuteCommand { command: "sbatch array.sbatch".into() });

        let created = session.ledger.events.iter().find_map(|event| match event {
            grading::EvidenceEvent::Command { command, created_job_ids, .. }
                if command == "sbatch array.sbatch" =>
            {
                Some(created_job_ids)
            }
            _ => None,
        });
        assert_eq!(created.map(Vec::len), Some(3));
        assert!(created.into_iter().flatten().all(|job_id| {
            session.world.jobs.get(job_id).is_some_and(|job| job.spec.array_index.is_some())
        }));
    }

    #[test]
    fn command_evidence_deserializes_legacy_shape_and_round_trips_context() {
        let legacy: grading::EvidenceEvent =
            serde_json::from_str(r#"{"type":"command","at":0,"command":"sinfo"}"#).unwrap();
        assert_eq!(
            legacy,
            grading::EvidenceEvent::Command {
                at: dgxlab_contracts::SimTimeMs::ZERO,
                command: "sinfo".into(),
                active_job_id: None,
                created_job_ids: Vec::new(),
            }
        );

        let contextual = grading::EvidenceEvent::Command {
            at: dgxlab_contracts::SimTimeMs(7),
            command: "nvidia-smi -L".into(),
            active_job_id: Some(dgxlab_contracts::JobId(10_000)),
            created_job_ids: vec![dgxlab_contracts::JobId(10_001), dgxlab_contracts::JobId(10_002)],
        };
        let json = serde_json::to_string(&contextual).unwrap();
        assert_eq!(serde_json::from_str::<grading::EvidenceEvent>(&json).unwrap(), contextual);
    }

    #[test]
    fn checkpoint_view_orders_event_backed_artifacts_by_recency() {
        let mut session = SimSession::new("dgx-h200-8", 9).unwrap();
        for path in [
            "/home/learner/checkpoints/orphan-b.pt",
            "/home/learner/checkpoints/orphan-a.pt",
            "/home/learner/checkpoints/epoch-999.pt",
            "/home/learner/checkpoints/epoch-001.pt",
        ] {
            session.world.fs.write_file(path, b"checkpoint").unwrap();
        }
        session.world.event_log.push(sim_core::WorldEventRecord {
            sequence: 10_000,
            at: dgxlab_contracts::SimTimeMs(10),
            kind: WorldEventKind::ArtifactWritten {
                job_id: dgxlab_contracts::JobId(10_000),
                path: "/home/learner/checkpoints/epoch-999.pt".into(),
            },
        });
        session.world.event_log.push(sim_core::WorldEventRecord {
            sequence: 10_001,
            at: dgxlab_contracts::SimTimeMs(20),
            kind: WorldEventKind::ArtifactWritten {
                job_id: dgxlab_contracts::JobId(10_000),
                path: "/home/learner/checkpoints/epoch-001.pt".into(),
            },
        });

        let paths = session.view().checkpoint_paths;
        assert_eq!(&paths[..2], &["checkpoints/orphan-a.pt", "checkpoints/orphan-b.pt"]);
        assert_eq!(paths.last().map(String::as_str), Some("checkpoints/epoch-001.pt"));
    }

    #[test]
    fn vfs_write_and_read_round_trip() {
        let mut session = SimSession::new("dgx-h200-8", 3).unwrap();
        let write = session.handle(SimRequest::WriteVfs {
            path: "train.sbatch".into(),
            content: "#!/bin/bash\n#SBATCH --gres=gpu:h200:1\npython train.py\n".into(),
        });
        assert!(matches!(write, SimResponse::State { .. }));
        let read = session.handle(SimRequest::ReadVfs { path: "train.sbatch".into() });
        assert!(matches!(read, SimResponse::FileContent { .. }));
    }

    #[test]
    fn batch_submission_path_works() {
        let (ok, digest) = lab06_batch_path(11).expect("lab06");
        assert!(ok, "batch path should score well");
        let again = lab06_batch_path(11).expect("lab06 again");
        assert_eq!(digest, again.1);
    }

    #[test]
    fn batch_completion_does_not_require_the_worked_example_job_name() {
        let mut session = SimSession::open_lab("06-batch-jobs", 11).expect("lab 06");
        let script = r"#!/bin/bash
#SBATCH --job-name=my-training-run
#SBATCH --partition=gpu
#SBATCH --gres=gpu:h200:1
#SBATCH --cpus-per-task=8
#SBATCH --mem=64G
#SBATCH --time=00:30:00
#SBATCH --output=logs/%x-%j.out

python train.py --batch-size 64 --epochs 5
";
        let _ = session
            .handle(SimRequest::WriteVfs { path: "train.sbatch".into(), content: script.into() });
        let _ = session.handle(SimRequest::ExecuteCommand { command: "cat train.sbatch".into() });
        let _ =
            session.handle(SimRequest::ExecuteCommand { command: "sbatch train.sbatch".into() });
        let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 30 * 60 * 1_000 });
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: "tail -n 20 logs/my-training-run-10000.out".into(),
        });
        let _ = session.handle(SimRequest::ExecuteCommand { command: "sacct".into() });

        assert!(
            session.view().lab_complete,
            "completion should depend on the learner job state, not a hidden name"
        );
    }

    #[test]
    fn authored_observation_steps_do_not_autocomplete_from_seeded_state() {
        for (lab_id, step_ids) in [
            ("06-batch-jobs", &["edit-script"][..]),
            ("09-failure-resume", &["checkpoint", "oom-observed"][..]),
        ] {
            let session = SimSession::open_lab(lab_id, 11).expect("course lab");
            let steps = session.view().lab_steps;
            for step_id in step_ids {
                let step = steps.iter().find(|step| step.id == *step_id).expect("authored step");
                assert!(!step.complete, "{lab_id}/{step_id} should require learner observation");
            }
        }
    }

    #[test]
    fn observation_commands_only_complete_the_step_they_evidence() {
        let mut batch = SimSession::open_lab("06-batch-jobs", 11).expect("lab 06");
        let _ = batch.handle(SimRequest::ExecuteCommand { command: "cat train.sbatch".into() });
        assert!(
            !batch
                .view()
                .lab_steps
                .iter()
                .find(|step| step.id == "edit-script")
                .expect("edit step")
                .complete,
            "reading the seeded script is not learner editing evidence"
        );
        let seeded = batch.world.fs.read_text("/home/learner/train.sbatch").unwrap();
        let _ = batch.handle(SimRequest::WriteVfs { path: "train.sbatch".into(), content: seeded });
        let batch_steps = batch.view().lab_steps;
        assert!(
            batch_steps.iter().find(|step| step.id == "edit-script").expect("edit step").complete
        );
        assert!(
            !batch_steps.iter().find(|step| step.id == "inspect-logs").expect("log step").complete
        );

        let mut recovery = SimSession::open_lab("09-failure-resume", 11).expect("lab 09");
        let _ = recovery
            .handle(SimRequest::ExecuteCommand { command: "cat checkpoints/epoch-004.pt".into() });
        let recovery_steps = recovery.view().lab_steps;
        assert!(
            recovery_steps
                .iter()
                .find(|step| step.id == "checkpoint")
                .expect("checkpoint step")
                .complete
        );
        assert!(
            !recovery_steps
                .iter()
                .find(|step| step.id == "inspect-logs")
                .expect("log step")
                .complete
        );
    }

    #[test]
    fn contended_job_pending_then_starts() {
        let (started, reason, digest) = lab07_contention_path(5).expect("lab07");
        assert_eq!(reason, "Resources");
        assert!(started);
        let again = lab07_contention_path(5).expect("lab07 again");
        assert_eq!(digest, again.2);
    }

    #[test]
    fn degraded_failure_lab_path() {
        let (ok, digest) = lab09_failure_path(2).expect("lab09");
        assert!(ok);
        let again = lab09_failure_path(2).expect("lab09 again");
        assert_eq!(digest, again.1);
    }

    #[test]
    fn array_script_submits_multiple_tasks() {
        let mut session = SimSession::new("dgx-h200-8", 4).unwrap();
        let script = "#!/bin/bash\n#SBATCH --job-name=arr\n#SBATCH --array=1-3\n#SBATCH --gres=gpu:h200:1\n#SBATCH --cpus-per-task=4\n#SBATCH --mem=16G\npython train.py\n";
        let _ = session
            .handle(SimRequest::WriteVfs { path: "array.sbatch".into(), content: script.into() });
        let response =
            session.handle(SimRequest::ExecuteCommand { command: "sbatch array.sbatch".into() });
        match response {
            SimResponse::CommandResult { lines, state, .. } => {
                assert!(
                    lines
                        .iter()
                        .any(|line| line.text.contains("array") || line.text.contains("Submitted"))
                );
                let learner_jobs = state.jobs.iter().filter(|job| job.user == "learner").count();
                assert!(learner_jobs >= 3, "expected array expansion, got {learner_jobs}");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn array_lab_requires_real_array_and_dependency_state() {
        let mut session = SimSession::open_lab("08-arrays-dependencies", 42).unwrap();
        for command in ["sbatch train.sbatch", "squeue", "scontrol show job 10002"] {
            let _ = session.handle(SimRequest::ExecuteCommand { command: command.into() });
        }
        assert!(!session.view().lab_complete, "a normal batch job is not array evidence");

        let array_script = "#!/bin/bash\n#SBATCH --job-name=sweep\n#SBATCH --array=0-3\n#SBATCH --gres=gpu:h200:1\n#SBATCH --mem=64G\npython train.py --epochs 1\n";
        let _ = session.handle(SimRequest::WriteVfs {
            path: "train.sbatch".into(),
            content: array_script.into(),
        });
        let _ =
            session.handle(SimRequest::ExecuteCommand { command: "sbatch train.sbatch".into() });

        let dependency_script = "#!/bin/bash\n#SBATCH --job-name=evaluate\n#SBATCH --dependency=afterok:10006\n#SBATCH --gres=gpu:h200:1\n#SBATCH --mem=64G\npython train.py --epochs 1\n";
        let _ = session.handle(SimRequest::WriteVfs {
            path: "train.sbatch".into(),
            content: dependency_script.into(),
        });
        let _ =
            session.handle(SimRequest::ExecuteCommand { command: "sbatch train.sbatch".into() });
        let _ = session
            .handle(SimRequest::ExecuteCommand { command: "scontrol show job 10007".into() });

        assert!(!session.view().lab_complete, "submitting a dependency is not observing release");
        let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 10 * 60_000 });
        assert!(session.view().lab_complete);
    }

    #[test]
    fn container_lab_requires_a_failed_missing_image_batch_job() {
        let mut session = SimSession::open_lab("05-containers", 42).unwrap();
        for command in [
            "salloc --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G",
            "module load singularity/4.5.0",
            "singularity exec --nv /containers/pytorch-lab.sif python train.py --epochs 1",
            "nvidia-smi -L",
            "singularity exec --nv /missing/pytorch-lab.sif python train.py",
        ] {
            let _ = session.handle(SimRequest::ExecuteCommand { command: command.into() });
        }
        assert!(
            !session.view().lab_complete,
            "an interactive error is not batch diagnosis evidence"
        );

        let script = "#!/bin/bash\n#SBATCH --job-name=container-missing\n#SBATCH --gres=gpu:h200:1\n#SBATCH --mem=64G\nsingularity exec --nv /missing/pytorch-lab.sif python train.py\n";
        let _ = session
            .handle(SimRequest::WriteVfs { path: "train.sbatch".into(), content: script.into() });
        let _ =
            session.handle(SimRequest::ExecuteCommand { command: "sbatch train.sbatch".into() });
        let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 60_000 });
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: "tail -n 20 logs/container-missing-10001.err".into(),
        });

        assert!(session.view().lab_complete);
    }

    #[test]
    fn course_has_twelve_labs() {
        assert_eq!(COURSE_LABS.len(), 12);
    }

    #[test]
    fn runtime_learning_paths_match_the_shipped_course_source() {
        let authored = AUTHORED_LABS
            .iter()
            .map(|yaml| serde_yaml::from_str::<AuthoredLabPath>(yaml).unwrap())
            .collect::<Vec<_>>();

        let authored_order = authored.iter().map(|lab| lab.id.as_str()).collect::<Vec<_>>();
        let runtime_order = COURSE_LABS.iter().map(|lab| lab.id).collect::<Vec<_>>();
        assert_eq!(authored_order, runtime_order, "course lab order drifted");

        for runtime in COURSE_LABS {
            let source = authored.iter().find(|lab| lab.id == runtime.id).unwrap();
            assert_eq!(source.title, runtime.title, "title drifted for {}", runtime.id);
            assert_eq!(source.scenario, runtime.scenario, "scenario drifted for {}", runtime.id);
            assert_eq!(
                source.estimated_minutes, runtime.estimated_minutes,
                "estimated time drifted for {}",
                runtime.id
            );
            let source_path = source
                .steps
                .iter()
                .map(|step| (step.id.as_str(), step.instruction.as_str()))
                .collect::<Vec<_>>();
            let runtime_path =
                runtime.steps.iter().map(|step| (step.check_id, step.label)).collect::<Vec<_>>();
            assert_eq!(source_path, runtime_path, "learning path drifted for {}", runtime.id);
        }
    }

    #[test]
    fn open_lab_preserves_learner_facing_identity() {
        let foundations = SimSession::open_lab("01-cluster-mental-model", 42).expect("lab 01");
        let batch = SimSession::open_lab("06-batch-jobs", 42).expect("lab 06");

        assert_eq!(foundations.view().lab_id, "01-cluster-mental-model");
        assert_eq!(batch.view().lab_id, "06-batch-jobs");
        assert_eq!(foundations.view().scenario_id, batch.view().scenario_id);
    }

    #[test]
    fn progress_uses_instructional_labels_instead_of_internal_ids() {
        let session = SimSession::open_lab("04-one-gpu", 42).expect("lab 04");
        let labels =
            session.view().lab_steps.into_iter().map(|step| step.label).collect::<Vec<_>>();

        assert_eq!(labels.first().map(String::as_str), Some("Inspect idle GPU capacity"));
        assert!(labels.iter().all(|label| !label.starts_with("step-")));
    }

    #[test]
    fn runtime_course_scenarios_match_authored_course() {
        let scenario_for =
            |id: &str| COURSE_LABS.iter().find(|lab| lab.id == id).map(|lab| lab.scenario);

        assert_eq!(scenario_for("08-arrays-dependencies"), Some("dgx-contended"));
        assert_eq!(scenario_for("11-policy-efficiency"), Some("dgx-shared"));
        assert_eq!(scenario_for("12-capstone"), Some("dgx-contended"));
    }

    #[test]
    fn shared_scenario_initializes_its_unique_course_lab() {
        let session = SimSession::new("dgx-shared", 42).unwrap();

        assert_eq!(session.view().lab_id, "11-policy-efficiency");
        assert!(!session.view().lab_steps.is_empty());
    }

    #[test]
    fn every_course_lab_opens_in_the_runtime() {
        for lab in COURSE_LABS {
            let session = SimSession::open_lab(lab.id, 42)
                .unwrap_or_else(|error| panic!("{} failed to open: {error}", lab.id));
            assert!(
                session.view().lab_steps.iter().all(|step| !step.complete),
                "{} should open before any learner-authored action is complete",
                lab.id
            );
        }
    }

    #[test]
    fn foundations_complete_with_the_authored_observation_commands() {
        let mut session = SimSession::open_lab("01-cluster-mental-model", 42).unwrap();
        for command in ["sinfo", "scontrol show node dgx-h200-01"] {
            let _ = session.handle(SimRequest::ExecuteCommand { command: command.into() });
        }

        assert!(session.view().lab_complete);
    }

    #[test]
    fn failed_commands_do_not_count_as_learning_evidence() {
        let mut session = SimSession::open_lab("01-cluster-mental-model", 42).unwrap();
        for command in ["sinfo", "scontrol show node not-a-node"] {
            let _ = session.handle(SimRequest::ExecuteCommand { command: command.into() });
        }

        assert!(
            !session.view().lab_complete,
            "an errored inspection must stay visible without completing its action"
        );
    }

    #[test]
    fn interactive_cpu_accepts_one_allocation_path_and_accounting_evidence() {
        let mut session = SimSession::open_lab("02-interactive-cpu", 42).unwrap();
        for command in [
            "salloc --cpus-per-task=4 --mem=8G --time=00:15:00",
            "echo $SLURM_JOB_ID",
            "env",
            "exit",
            "sacct",
        ] {
            let _ = session.handle(SimRequest::ExecuteCommand { command: command.into() });
        }

        assert!(session.view().lab_complete);
    }

    #[test]
    fn cpu_memory_requires_observing_the_simulated_oom() {
        let mut session = SimSession::open_lab("03-cpu-memory", 42).unwrap();
        for command in [
            "srun --job-name=prep-ok --cpus-per-task=8 --mem=64G python preprocess.py --epochs 2",
            "scontrol show job 10000",
            "srun --job-name=prep-oom --cpus-per-task=8 --mem=16G python preprocess.py --epochs 2",
        ] {
            let _ = session.handle(SimRequest::ExecuteCommand { command: command.into() });
        }
        assert!(!session.view().lab_complete);

        let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 60_000 });
        assert!(session.view().lab_complete);
    }

    #[test]
    fn failure_lab_requires_a_recovery_submission() {
        let mut session = SimSession::open_lab("09-failure-resume", 42).unwrap();
        assert!(!session.critical_practical_passed());
        for command in [
            "sacct",
            "tail -n 20 logs/train-llm-10000.err",
            "scontrol show job 10000",
            "cat checkpoints/epoch-004.pt",
        ] {
            let _ = session.handle(SimRequest::ExecuteCommand { command: command.into() });
        }
        assert!(!session.critical_practical_passed());

        let _ = session.handle(SimRequest::ExecuteCommand {
            command: "srun --job-name=train-resume --partition=gpu --gres=gpu:h200:4 --cpus-per-task=16 --mem=64G --time=02:00:00 python train.py --batch-size 64 --epochs 5 --resume-from-checkpoint checkpoints/epoch-004.pt".into(),
        });
        let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 300_000 });
        assert!(session.critical_practical_passed());
    }

    #[test]
    fn recovery_grading_rejects_name_only_or_undersized_jobs() {
        let mut missing_checkpoint = SimSession::open_lab("09-failure-resume", 42).unwrap();
        let _ = missing_checkpoint.handle(SimRequest::ExecuteCommand {
            command: "srun --job-name=train-resume --gres=gpu:h200:4 --mem=64G python train.py --batch-size 64".into(),
        });
        let _ = missing_checkpoint.handle(SimRequest::AdvanceClock { delta_ms: 300_000 });
        assert!(
            !missing_checkpoint.critical_practical_passed(),
            "a recovery job without a checkpoint must not pass"
        );

        let mut undersized = SimSession::open_lab("09-failure-resume", 42).unwrap();
        let _ = undersized.handle(SimRequest::ExecuteCommand {
            command: "srun --job-name=train-resume --gres=gpu:h200:4 --mem=48G python train.py --batch-size 64 --resume-from-checkpoint checkpoints/epoch-001.pt".into(),
        });
        let _ = undersized.handle(SimRequest::AdvanceClock { delta_ms: 300_000 });
        assert!(
            !undersized.critical_practical_passed(),
            "a recovery job below the corrected 64 GiB request must not pass"
        );

        let mut non_checkpoint = SimSession::open_lab("09-failure-resume", 42).unwrap();
        let _ = non_checkpoint.handle(SimRequest::ExecuteCommand {
            command: "srun --job-name=train-resume --gres=gpu:h200:4 --mem=64G python train.py --batch-size 64 --resume-from-checkpoint train.sbatch".into(),
        });
        let _ = non_checkpoint.handle(SimRequest::AdvanceClock { delta_ms: 300_000 });
        assert!(
            !non_checkpoint.critical_practical_passed(),
            "a readable non-checkpoint file must not satisfy recovery evidence"
        );
    }

    #[test]
    fn capstone_recovery_rejects_a_completed_job_without_checkpoint_evidence() {
        let mut session = SimSession::open_lab("12-capstone", 42).unwrap();
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: "srun --job-name=capstone-failure --gres=gpu:h200:1 --cpus-per-task=8 --mem=16G python train.py --batch-size 64 --epochs 3".into(),
        });
        let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 60_000 });
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: "srun --job-name=capstone-recovery --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G python train.py --batch-size 64 --epochs 3".into(),
        });
        let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 300_000 });
        let _ = session.handle(SimRequest::ExecuteCommand { command: "sacct".into() });

        assert!(
            !session.critical_practical_passed(),
            "capstone recovery must preserve a verified checkpoint argument"
        );
    }

    #[test]
    fn capstone_requires_failure_evidence_and_a_corrected_recovery() {
        let mut session = SimSession::open_lab("12-capstone", 42).unwrap();
        for command in ["sbatch train.sbatch", "squeue"] {
            let _ = session.handle(SimRequest::ExecuteCommand { command: command.into() });
        }
        let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 600_000 });
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: "srun --job-name=capstone-failure --gres=gpu:h200:1 --cpus-per-task=8 --mem=16G python train.py --batch-size 64 --epochs 3".into(),
        });
        assert!(!session.critical_practical_passed());

        let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 60_000 });
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: "srun --job-name=capstone-recovery --partition=gpu --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:30:00 python train.py --batch-size 64 --epochs 3 --resume-from-checkpoint checkpoints/epoch-002.pt".into(),
        });
        let _ = session.handle(SimRequest::AdvanceClock { delta_ms: 300_000 });
        let _ = session.handle(SimRequest::ExecuteCommand { command: "sacct".into() });

        assert!(session.view().lab_complete);
        assert!(session.critical_practical_passed());
    }
}
