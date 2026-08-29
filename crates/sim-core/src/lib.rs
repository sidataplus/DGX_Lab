#![forbid(unsafe_code)]

//! Deterministic discrete-event simulation kernel for DGX Lab.

use actors::ActorAction;
use dgxlab_contracts::{EventId, JobId, SimTimeMs};
use scheduler::{release_allocation, schedule_pending, validate_job};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use slurm_model::{
    AccountingRecord, ClusterState, GpuHealth, JobRecord, JobSpec, JobStatus, NodeStatus,
    PendingReason,
};
use std::collections::{BTreeMap, VecDeque};
use virtual_fs::VirtualFileSystem;
use workloads::{plan_workload, request_from_command, LogStream};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationWorld {
    pub scenario_id: String,
    pub seed: u64,
    pub now: SimTimeMs,
    pub paused: bool,
    pub clock_multiplier: u32,
    pub cluster: ClusterState,
    pub jobs: BTreeMap<JobId, JobRecord>,
    pub accounting: BTreeMap<JobId, AccountingRecord>,
    pub fs: VirtualFileSystem,
    pub event_log: Vec<WorldEventRecord>,
    queue: EventQueue,
    rng: DeterministicRng,
    next_job_id: u64,
    next_event_id: u64,
}

impl SimulationWorld {
    #[must_use]
    pub fn dgx_h200_8(seed: u64) -> Self {
        Self {
            scenario_id: "dgx-h200-8".into(),
            seed,
            now: SimTimeMs::ZERO,
            paused: false,
            clock_multiplier: 1,
            cluster: ClusterState::dgx_h200_8(),
            jobs: BTreeMap::new(),
            accounting: BTreeMap::new(),
            fs: VirtualFileSystem::dgx_default(),
            event_log: Vec::new(),
            queue: EventQueue::default(),
            rng: DeterministicRng::new(seed),
            next_job_id: 10_000,
            next_event_id: 1,
        }
    }

    pub fn submit_job(&mut self, spec: JobSpec) -> Result<JobId, SimError> {
        validate_job(&self.cluster, &spec)?;
        let job_id = JobId(self.next_job_id);
        self.next_job_id = self.next_job_id.saturating_add(1);
        let stdout_path = resolve_output_path(&spec, job_id, false);
        let stderr_path = resolve_output_path(&spec, job_id, true);
        ensure_parent(&mut self.fs, &stdout_path)?;
        ensure_parent(&mut self.fs, &stderr_path)?;
        let record = JobRecord {
            id: job_id,
            spec,
            status: JobStatus::Pending,
            pending_reason: PendingReason::Priority,
            submitted_at: self.now,
            eligible_at: self.now,
            started_at: None,
            ended_at: None,
            allocation: None,
            exit_code: None,
            steps: Vec::new(),
            stdout_path,
            stderr_path,
        };
        self.jobs.insert(job_id, record);
        self.record(WorldEventKind::JobSubmitted { job_id });
        self.schedule_at(self.now, ScheduledEventKind::TrySchedule);
        self.process_due()?;
        Ok(job_id)
    }

    pub fn complete_interactive_job(&mut self, job_id: JobId) -> Result<(), SimError> {
        let (status, allocation, workload_id) = self
            .jobs
            .get(&job_id)
            .map(|job| (job.status, job.allocation.clone(), job.spec.workload_id.clone()))
            .ok_or(SimError::JobNotFound(job_id))?;
        if workload_id != "interactive-shell-v1" || status != JobStatus::Running {
            return Err(SimError::NotRunningInteractiveJob(job_id));
        }
        if let Some(allocation) = allocation.as_ref() {
            release_allocation(&mut self.cluster, job_id, allocation);
        }
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.status = JobStatus::Completed;
            job.ended_at = Some(self.now);
            job.exit_code = Some((0, 0));
        }
        self.record(WorldEventKind::JobFinished { job_id, status: JobStatus::Completed });
        self.finalize_accounting(job_id);
        self.schedule_at(self.now, ScheduledEventKind::TrySchedule);
        self.process_due()?;
        Ok(())
    }

    pub fn cancel_job(&mut self, job_id: JobId) -> Result<(), SimError> {
        let (status, allocation) = self
            .jobs
            .get(&job_id)
            .map(|job| (job.status, job.allocation.clone()))
            .ok_or(SimError::JobNotFound(job_id))?;
        if status.is_terminal() {
            return Err(SimError::JobAlreadyTerminal(job_id));
        }
        if let Some(allocation) = allocation.as_ref() {
            release_allocation(&mut self.cluster, job_id, allocation);
        }
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.status = JobStatus::Cancelled;
            job.ended_at = Some(self.now);
            job.exit_code = Some((0, 15));
        }
        self.record(WorldEventKind::JobFinished { job_id, status: JobStatus::Cancelled });
        self.finalize_accounting(job_id);
        self.schedule_at(self.now, ScheduledEventKind::TrySchedule);
        self.process_due()?;
        Ok(())
    }

    pub fn advance_by(&mut self, delta_ms: u64) -> Result<(), SimError> {
        self.advance_to(self.now.saturating_add(delta_ms))
    }

    pub fn advance_to(&mut self, target: SimTimeMs) -> Result<(), SimError> {
        if target < self.now {
            return Err(SimError::ClockCannotReverse { now: self.now, target });
        }
        while let Some((time, event)) = self.queue.pop_next_before_or_at(target) {
            self.now = time;
            self.process_event(event)?;
        }
        self.now = target;
        Ok(())
    }

    pub fn apply_actor_action(&mut self, action: ActorAction) -> Result<(), SimError> {
        match action {
            ActorAction::SubmitJob { spec } => {
                self.submit_job(*spec)?;
            }
            ActorAction::CancelJob { job_id } => self.cancel_job(job_id)?,
            ActorAction::DrainNode { node_id, reason } => {
                let node = self
                    .cluster
                    .nodes
                    .get_mut(&node_id)
                    .ok_or_else(|| SimError::NodeNotFound(node_id.clone()))?;
                node.status = NodeStatus::Draining;
                node.drain_reason = Some(reason.clone());
                self.record(WorldEventKind::NodeDrained { node_id, reason });
            }
            ActorAction::ResumeNode { node_id } => {
                let node = self
                    .cluster
                    .nodes
                    .get_mut(&node_id)
                    .ok_or_else(|| SimError::NodeNotFound(node_id.clone()))?;
                node.status = if node.running_jobs.is_empty() {
                    NodeStatus::Idle
                } else {
                    NodeStatus::Mixed
                };
                node.drain_reason = None;
                self.record(WorldEventKind::NodeResumed { node_id });
                self.schedule_at(self.now, ScheduledEventKind::TrySchedule);
            }
            ActorAction::InjectGpuWarning { node_id, gpu_index } => {
                let node = self
                    .cluster
                    .nodes
                    .get_mut(&node_id)
                    .ok_or_else(|| SimError::NodeNotFound(node_id.clone()))?;
                let gpu = node
                    .gpus
                    .iter_mut()
                    .find(|gpu| gpu.index == gpu_index)
                    .ok_or_else(|| SimError::GpuNotFound { node_id: node_id.clone(), gpu_index })?;
                gpu.health = GpuHealth::Warning;
                self.record(WorldEventKind::GpuHealthChanged {
                    node_id,
                    gpu_index,
                    health: GpuHealth::Warning,
                });
            }
            ActorAction::RestoreGpu { node_id, gpu_index } => {
                let node = self
                    .cluster
                    .nodes
                    .get_mut(&node_id)
                    .ok_or_else(|| SimError::NodeNotFound(node_id.clone()))?;
                let gpu = node
                    .gpus
                    .iter_mut()
                    .find(|gpu| gpu.index == gpu_index)
                    .ok_or_else(|| SimError::GpuNotFound { node_id: node_id.clone(), gpu_index })?;
                gpu.health = GpuHealth::Ok;
                self.record(WorldEventKind::GpuHealthChanged {
                    node_id,
                    gpu_index,
                    health: GpuHealth::Ok,
                });
            }
        }
        self.process_due()
    }

    #[must_use]
    pub fn state_digest(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("serializing world into JSON cannot fail");
        hex::encode(Sha256::digest(bytes))
    }

    #[must_use]
    pub fn next_random_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    fn process_due(&mut self) -> Result<(), SimError> {
        while let Some((time, event)) = self.queue.pop_next_before_or_at(self.now) {
            self.now = time;
            self.process_event(event)?;
        }
        Ok(())
    }

    fn process_event(&mut self, event: ScheduledEvent) -> Result<(), SimError> {
        match event.kind {
            ScheduledEventKind::TrySchedule => self.try_schedule(),
            ScheduledEventKind::WriteLog { job_id, stream, text } => {
                let job = self.jobs.get(&job_id).ok_or(SimError::JobNotFound(job_id))?;
                if job.status.is_terminal() && job.ended_at.is_some_and(|ended| ended < self.now) {
                    return Ok(());
                }
                let path = match stream {
                    LogStream::Stdout => job.stdout_path.clone(),
                    LogStream::Stderr => job.stderr_path.clone(),
                };
                self.fs.append_file(&path, format!("{text}\n").as_bytes())?;
                self.record(WorldEventKind::JobLog { job_id, stream, text });
                Ok(())
            }
            ScheduledEventKind::WriteArtifact { job_id, relative_path, content } => {
                let owner = self
                    .jobs
                    .get(&job_id)
                    .map(|job| job.spec.user.clone())
                    .ok_or(SimError::JobNotFound(job_id))?;
                let path = if relative_path.starts_with('/') {
                    relative_path
                } else {
                    format!("/home/{owner}/{relative_path}")
                };
                ensure_parent(&mut self.fs, &path)?;
                self.fs.write_file(&path, content.as_bytes())?;
                self.record(WorldEventKind::ArtifactWritten { job_id, path });
                Ok(())
            }
            ScheduledEventKind::FinishJob { job_id, status, exit_code } => {
                self.finish_job(job_id, status, exit_code)
            }
            ScheduledEventKind::ActorAction(action) => self.apply_actor_action(action),
        }
    }

    fn try_schedule(&mut self) -> Result<(), SimError> {
        let result = schedule_pending(&mut self.cluster, &mut self.jobs, self.now);
        for (job_id, reason) in result.pending_updates {
            self.record(WorldEventKind::JobPending { job_id, reason });
        }
        for decision in result.started {
            let job = self
                .jobs
                .get(&decision.job_id)
                .cloned()
                .ok_or(SimError::JobNotFound(decision.job_id))?;
            self.record(WorldEventKind::JobStarted {
                job_id: decision.job_id,
                node_id: decision.allocation.node_id,
                gpu_indices: decision.allocation.gpu_indices,
            });
            let request = request_from_command(&job.spec.command, &job.spec.workload_id);
            let plan = plan_workload(&job.spec, &request, self.now);
            for log in plan.logs {
                self.schedule_at(
                    self.now.saturating_add(log.offset_ms),
                    ScheduledEventKind::WriteLog {
                        job_id: decision.job_id,
                        stream: log.stream,
                        text: log.text,
                    },
                );
            }
            for artifact in plan.artifacts {
                self.schedule_at(
                    self.now.saturating_add(artifact.offset_ms),
                    ScheduledEventKind::WriteArtifact {
                        job_id: decision.job_id,
                        relative_path: artifact.path,
                        content: artifact.content,
                    },
                );
            }
            if job.spec.workload_id != "interactive-shell-v1" {
                self.schedule_at(
                    self.now.saturating_add(plan.terminal_after_ms),
                    ScheduledEventKind::FinishJob {
                        job_id: decision.job_id,
                        status: plan.terminal_status,
                        exit_code: plan.exit_code,
                    },
                );
            }
        }
        Ok(())
    }

    fn finish_job(
        &mut self,
        job_id: JobId,
        status: JobStatus,
        exit_code: (u8, u8),
    ) -> Result<(), SimError> {
        let (should_finish, allocation) = self
            .jobs
            .get(&job_id)
            .map(|job| (!job.status.is_terminal(), job.allocation.clone()))
            .ok_or(SimError::JobNotFound(job_id))?;
        if !should_finish {
            // A previously cancelled/failed job may still have future synthetic
            // events in the queue. Ignore them without releasing resources twice.
            return Ok(());
        }
        if let Some(allocation) = allocation.as_ref() {
            release_allocation(&mut self.cluster, job_id, allocation);
        }
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.status = status;
            job.ended_at = Some(self.now);
            job.exit_code = Some(exit_code);
        }
        self.record(WorldEventKind::JobFinished { job_id, status });
        self.finalize_accounting(job_id);
        self.schedule_at(self.now, ScheduledEventKind::TrySchedule);
        self.process_due()
    }

    fn finalize_accounting(&mut self, job_id: JobId) {
        if let Some(job) = self.jobs.get(&job_id) {
            self.accounting.insert(
                job_id,
                AccountingRecord {
                    job_id,
                    user: job.spec.user.clone(),
                    account: job.spec.account.clone(),
                    state: job.status,
                    requested: job.spec.resources.clone(),
                    allocation: job.allocation.clone(),
                    submit_time: job.submitted_at,
                    start_time: job.started_at,
                    end_time: job.ended_at,
                    elapsed_ms: job.elapsed_ms(self.now),
                    exit_code: job.exit_code,
                },
            );
        }
    }

    fn schedule_at(&mut self, at: SimTimeMs, kind: ScheduledEventKind) {
        let event = ScheduledEvent {
            id: EventId(self.next_event_id),
            kind,
        };
        self.next_event_id = self.next_event_id.saturating_add(1);
        self.queue.push(at, event);
    }

    fn record(&mut self, kind: WorldEventKind) {
        let sequence = self.event_log.len() as u64 + 1;
        self.event_log.push(WorldEventRecord { sequence, at: self.now, kind });
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ScheduledEvent {
    id: EventId,
    kind: ScheduledEventKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ScheduledEventKind {
    TrySchedule,
    WriteLog { job_id: JobId, stream: LogStream, text: String },
    WriteArtifact { job_id: JobId, relative_path: String, content: String },
    FinishJob { job_id: JobId, status: JobStatus, exit_code: (u8, u8) },
    ActorAction(ActorAction),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct EventQueue {
    /// String keys keep JSON serialization valid (`(SimTimeMs, u64)` map keys are not).
    /// Format `{time_ms:020}:{event_id:020}` preserves chronological BTree order.
    events: BTreeMap<String, VecDeque<ScheduledEvent>>,
}

impl EventQueue {
    fn queue_key(at: SimTimeMs, event_id: u64) -> String {
        format!("{:020}:{:020}", at.0, event_id)
    }

    fn parse_key(key: &str) -> Option<SimTimeMs> {
        let (time, _) = key.split_once(':')?;
        time.parse().ok().map(SimTimeMs)
    }

    fn push(&mut self, at: SimTimeMs, event: ScheduledEvent) {
        let key = Self::queue_key(at, event.id.0);
        self.events.entry(key).or_default().push_back(event);
    }

    fn pop_next_before_or_at(&mut self, target: SimTimeMs) -> Option<(SimTimeMs, ScheduledEvent)> {
        let key = self.events.keys().next()?.clone();
        let at = Self::parse_key(&key)?;
        if at > target {
            return None;
        }
        let queue = self.events.get_mut(&key)?;
        let event = queue.pop_front()?;
        if queue.is_empty() {
            self.events.remove(&key);
        }
        Some((at, event))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D049BB133111EB);
        value ^ (value >> 31)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldEventRecord {
    pub sequence: u64,
    pub at: SimTimeMs,
    pub kind: WorldEventKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorldEventKind {
    JobSubmitted { job_id: JobId },
    JobPending { job_id: JobId, reason: PendingReason },
    JobStarted { job_id: JobId, node_id: String, gpu_indices: Vec<u16> },
    JobLog { job_id: JobId, stream: LogStream, text: String },
    ArtifactWritten { job_id: JobId, path: String },
    JobFinished { job_id: JobId, status: JobStatus },
    NodeDrained { node_id: String, reason: String },
    NodeResumed { node_id: String },
    GpuHealthChanged { node_id: String, gpu_index: u16, health: GpuHealth },
}

fn resolve_output_path(spec: &JobSpec, job_id: JobId, stderr: bool) -> String {
    let configured = if stderr {
        spec.error_path.as_deref()
    } else {
        spec.output_path.as_deref()
    };
    if let Some(template) = configured {
        let expanded = template
            .replace("%x", &spec.name)
            .replace("%j", &job_id.0.to_string())
            .replace("%A", &job_id.0.to_string());
        return if expanded.starts_with('/') {
            expanded
        } else {
            format!("/home/{}/{}", spec.user, expanded)
        };
    }
    let suffix = if stderr { "err" } else { "out" };
    format!("/home/{}/logs/{}-{}.{}", spec.user, spec.name, job_id.0, suffix)
}

fn ensure_parent(fs: &mut VirtualFileSystem, path: &str) -> Result<(), virtual_fs::VfsError> {
    if let Some((parent, _)) = path.rsplit_once('/') {
        fs.mkdir_all(if parent.is_empty() { "/" } else { parent })?;
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum SimError {
    #[error(transparent)]
    SchedulerValidation(#[from] scheduler::ValidationError),
    #[error(transparent)]
    Vfs(#[from] virtual_fs::VfsError),
    #[error("job not found: {0}")]
    JobNotFound(JobId),
    #[error("job is already terminal: {0}")]
    JobAlreadyTerminal(JobId),
    #[error("job is not a running interactive allocation: {0}")]
    NotRunningInteractiveJob(JobId),
    #[error("node not found: {0}")]
    NodeNotFound(String),
    #[error("GPU {gpu_index} not found on node {node_id}")]
    GpuNotFound { node_id: String, gpu_index: u16 },
    #[error("simulation clock cannot reverse from {now} to {target}")]
    ClockCannotReverse { now: SimTimeMs, target: SimTimeMs },
}

#[cfg(test)]
mod tests {
    use super::*;
    use slurm_model::Tres;

    fn gpu_job(name: &str, gpus: u16) -> JobSpec {
        JobSpec {
            name: name.into(),
            resources: Tres {
                cpus: 8,
                memory_mib: 64 * 1024,
                gpu_type: Some("h200".into()),
                gpus,
            },
            command: "python train.py --batch-size 64 --epochs 2".into(),
            workload_id: "pytorch-training-v1".into(),
            ..JobSpec::default()
        }
    }

    #[test]
    fn equal_seed_and_commands_produce_equal_digest() {
        let mut first = SimulationWorld::dgx_h200_8(42);
        let mut second = SimulationWorld::dgx_h200_8(42);
        first.submit_job(gpu_job("a", 1)).unwrap();
        second.submit_job(gpu_job("a", 1)).unwrap();
        first.advance_by(90_000).unwrap();
        second.advance_by(90_000).unwrap();
        assert_eq!(first.state_digest(), second.state_digest());
    }

    #[test]
    fn queued_job_starts_after_resources_release() {
        let mut world = SimulationWorld::dgx_h200_8(1);
        let first = world.submit_job(gpu_job("all", 8)).unwrap();
        let second = world.submit_job(gpu_job("waiting", 1)).unwrap();
        assert_eq!(world.jobs[&second].status, JobStatus::Pending);
        world.advance_by(31_000).unwrap();
        assert_eq!(world.jobs[&first].status, JobStatus::Completed);
        assert_eq!(world.jobs[&second].status, JobStatus::Running);
    }
}

