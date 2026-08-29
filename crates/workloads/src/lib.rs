#![forbid(unsafe_code)]

//! Deterministic synthetic workload planning. No workload code is executed.

use dgxlab_contracts::SimTimeMs;
use serde::{Deserialize, Serialize};
use slurm_model::{JobSpec, JobStatus};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedLogLine {
    pub offset_ms: u64,
    pub stream: LogStream,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogStream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedArtifact {
    pub offset_ms: u64,
    pub path: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryPoint {
    pub offset_ms: u64,
    pub cpu_percent: u8,
    pub host_memory_mib: u64,
    pub gpu_percent: u8,
    pub gpu_memory_mib: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkloadPlan {
    pub workload_id: String,
    pub start_time: SimTimeMs,
    pub natural_duration_ms: u64,
    pub terminal_after_ms: u64,
    pub terminal_status: JobStatus,
    pub exit_code: (u8, u8),
    pub logs: Vec<PlannedLogLine>,
    pub artifacts: Vec<PlannedArtifact>,
    pub telemetry: Vec<TelemetryPoint>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkloadRequest {
    pub workload_id: String,
    pub batch_size: u32,
    pub epochs: u32,
    pub checkpoint_every_epochs: u32,
    pub forced_failure: Option<FailureMode>,
}

impl Default for WorkloadRequest {
    fn default() -> Self {
        Self {
            workload_id: "pytorch-training-v1".into(),
            batch_size: 64,
            epochs: 5,
            checkpoint_every_epochs: 1,
            forced_failure: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureMode {
    GpuOutOfMemory,
    HostOutOfMemory,
    ScriptFailure,
    MissingInput,
    NodeFailure,
}

pub fn request_from_command(command: &str, workload_id: &str) -> WorkloadRequest {
    let mut request = WorkloadRequest { workload_id: workload_id.into(), ..Default::default() };
    let tokens: Vec<&str> = command.split_whitespace().collect();
    for (index, token) in tokens.iter().enumerate() {
        if *token == "--batch-size"
            && let Some(value) = tokens.get(index + 1).and_then(|value| value.parse().ok())
        {
            request.batch_size = value;
        }
        if *token == "--epochs"
            && let Some(value) = tokens.get(index + 1).and_then(|value| value.parse().ok())
        {
            request.epochs = value;
        }
    }
    request
}

pub fn plan_workload(spec: &JobSpec, request: &WorkloadRequest, start_time: SimTimeMs) -> WorkloadPlan {
    let natural_duration_ms = natural_duration(request);
    let inferred_failure = request.forced_failure.or_else(|| infer_failure(spec, request));
    let (terminal_status, failure_at_ms) = match inferred_failure {
        Some(FailureMode::GpuOutOfMemory) => (JobStatus::OutOfMemory, natural_duration_ms.min(45_000)),
        Some(FailureMode::HostOutOfMemory) => (JobStatus::OutOfMemory, natural_duration_ms.min(75_000)),
        Some(FailureMode::ScriptFailure | FailureMode::MissingInput) => {
            (JobStatus::Failed, natural_duration_ms.min(2_000))
        }
        Some(FailureMode::NodeFailure) => (JobStatus::NodeFail, natural_duration_ms.min(30_000)),
        None if spec.time_limit_ms < natural_duration_ms => (JobStatus::Timeout, spec.time_limit_ms),
        None => (JobStatus::Completed, natural_duration_ms),
    };
    let mut logs = training_logs(request, terminal_status, failure_at_ms);
    if terminal_status == JobStatus::Timeout {
        logs.push(PlannedLogLine {
            offset_ms: failure_at_ms,
            stream: LogStream::Stderr,
            text: "slurmstepd: error: job cancelled due to time limit".into(),
        });
    }
    let artifacts = checkpoint_artifacts(request, terminal_status, failure_at_ms, natural_duration_ms);
    let telemetry = telemetry_curve(spec, request, terminal_status, failure_at_ms);
    WorkloadPlan {
        workload_id: request.workload_id.clone(),
        start_time,
        natural_duration_ms,
        terminal_after_ms: failure_at_ms,
        terminal_status,
        exit_code: if terminal_status == JobStatus::Completed { (0, 0) } else { (1, 0) },
        logs,
        artifacts,
        telemetry,
    }
}

fn natural_duration(request: &WorkloadRequest) -> u64 {
    let base_per_epoch: u64 = match request.workload_id.as_str() {
        "cpu-preprocess-v1" => 12_000,
        "torchrun-multigpu-v1" => 9_000,
        "checkpoint-resume-v1" => 11_000,
        "interactive-shell-v1" => 24 * 60 * 60 * 1_000,
        _ => 15_000,
    };
    base_per_epoch.saturating_mul(u64::from(request.epochs.max(1)))
}

fn infer_failure(spec: &JobSpec, request: &WorkloadRequest) -> Option<FailureMode> {
    if request.batch_size > 160 && spec.resources.gpus > 0 {
        return Some(FailureMode::GpuOutOfMemory);
    }
    if request.workload_id != "interactive-shell-v1" && spec.resources.memory_mib < 48 * 1024 {
        return Some(FailureMode::HostOutOfMemory);
    }
    if spec.command.contains("missing.py") || spec.command.contains("/missing/") {
        return Some(FailureMode::MissingInput);
    }
    if spec.command.contains("syntax-error") {
        return Some(FailureMode::ScriptFailure);
    }
    None
}

fn training_logs(request: &WorkloadRequest, status: JobStatus, terminal_after_ms: u64) -> Vec<PlannedLogLine> {
    if request.workload_id == "interactive-shell-v1" {
        return vec![PlannedLogLine {
            offset_ms: 0,
            stream: LogStream::Stdout,
            text: "Interactive allocation ready.".into(),
        }];
    }
    let epochs = request.epochs.max(1);
    let step_ms = terminal_after_ms.max(epochs as u64) / epochs as u64;
    let mut lines = Vec::new();
    for epoch in 1..=epochs {
        let offset = step_ms.saturating_mul(epoch as u64).min(terminal_after_ms.saturating_sub(1));
        let loss_milli = 2_400_u32.saturating_sub(epoch.saturating_mul(210));
        lines.push(PlannedLogLine {
            offset_ms: offset,
            stream: LogStream::Stdout,
            text: format!(
                "Epoch {epoch}/{epochs} loss={}.{:03} batch_size={}",
                loss_milli / 1000,
                loss_milli % 1000,
                request.batch_size
            ),
        });
    }
    match status {
        JobStatus::OutOfMemory if request.batch_size > 160 => lines.push(PlannedLogLine {
            offset_ms: terminal_after_ms,
            stream: LogStream::Stderr,
            text: "torch.OutOfMemoryError: simulated GPU memory allocation failed".into(),
        }),
        JobStatus::OutOfMemory => lines.push(PlannedLogLine {
            offset_ms: terminal_after_ms,
            stream: LogStream::Stderr,
            text: "slurmstepd: error: Detected simulated host-memory oom_kill event".into(),
        }),
        JobStatus::Failed => lines.push(PlannedLogLine {
            offset_ms: terminal_after_ms,
            stream: LogStream::Stderr,
            text: "python: simulated workload entry point failed".into(),
        }),
        JobStatus::Completed => lines.push(PlannedLogLine {
            offset_ms: terminal_after_ms,
            stream: LogStream::Stdout,
            text: "Training completed successfully.".into(),
        }),
        _ => {}
    }
    lines
}

fn checkpoint_artifacts(
    request: &WorkloadRequest,
    status: JobStatus,
    terminal_after_ms: u64,
    natural_duration_ms: u64,
) -> Vec<PlannedArtifact> {
    if request.workload_id == "interactive-shell-v1" || request.checkpoint_every_epochs == 0 {
        return Vec::new();
    }
    let per_epoch = natural_duration_ms / request.epochs.max(1) as u64;
    (1..=request.epochs)
        .filter(|epoch| epoch % request.checkpoint_every_epochs == 0)
        .filter_map(|epoch| {
            let at = per_epoch.saturating_mul(epoch as u64);
            (at < terminal_after_ms || status == JobStatus::Completed).then_some(PlannedArtifact {
                offset_ms: at.min(terminal_after_ms),
                path: format!("checkpoints/epoch-{epoch:03}.pt"),
                content: format!("DGX-LAB-SIMULATED-CHECKPOINT epoch={epoch}"),
            })
        })
        .collect()
}

fn telemetry_curve(
    spec: &JobSpec,
    request: &WorkloadRequest,
    status: JobStatus,
    terminal_after_ms: u64,
) -> Vec<TelemetryPoint> {
    let points = 10_u64;
    (0..=points)
        .map(|point| {
            let offset = terminal_after_ms.saturating_mul(point) / points;
            let wave = ((point * 17 + request.batch_size as u64) % 23) as u8;
            let gpu_memory = if spec.resources.gpus > 0 {
                30_000 + request.batch_size as u64 * 420
            } else {
                0
            };
            TelemetryPoint {
                offset_ms: offset,
                cpu_percent: 35 + wave.min(55),
                host_memory_mib: spec.resources.memory_mib.saturating_mul(65 + point) / 100,
                gpu_percent: if spec.resources.gpus > 0 { 70 + wave.min(28) } else { 0 },
                gpu_memory_mib: if status == JobStatus::OutOfMemory && point == points {
                    141_000
                } else {
                    gpu_memory.min(138_000)
                },
            }
        })
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum WorkloadError {
    #[error("unknown workload: {0}")]
    UnknownWorkload(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use slurm_model::Tres;

    fn spec() -> JobSpec {
        JobSpec {
            resources: Tres {
                cpus: 8,
                memory_mib: 64 * 1024,
                gpu_type: Some("h200".into()),
                gpus: 1,
            },
            ..JobSpec::default()
        }
    }

    #[test]
    fn oversized_batch_causes_gpu_oom() {
        let request = WorkloadRequest { batch_size: 256, ..Default::default() };
        let plan = plan_workload(&spec(), &request, SimTimeMs::ZERO);
        assert_eq!(plan.terminal_status, JobStatus::OutOfMemory);
    }

    #[test]
    fn short_walltime_causes_timeout() {
        let mut spec = spec();
        spec.time_limit_ms = 1_000;
        let plan = plan_workload(&spec, &WorkloadRequest::default(), SimTimeMs::ZERO);
        assert_eq!(plan.terminal_status, JobStatus::Timeout);
        assert_eq!(plan.terminal_after_ms, 1_000);
    }
}
