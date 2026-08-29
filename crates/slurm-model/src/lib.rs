#![forbid(unsafe_code)]

//! Pedagogically faithful SLURM-like domain types.
//!
//! This crate models only simulator state. It contains no scheduler client,
//! command execution, sockets, or operating-system integration.

use dgxlab_contracts::{JobId, SimTimeMs, StepId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tres {
    pub cpus: u32,
    pub memory_mib: u64,
    pub gpu_type: Option<String>,
    pub gpus: u16,
}

impl Tres {
    #[must_use]
    pub fn fits_within(&self, capacity: &Self) -> bool {
        let gpu_type_matches = self.gpus == 0
            || self.gpu_type.is_none()
            || capacity.gpu_type.is_none()
            || self.gpu_type == capacity.gpu_type;
        self.cpus <= capacity.cpus
            && self.memory_mib <= capacity.memory_mib
            && self.gpus <= capacity.gpus
            && gpu_type_matches
    }

    #[must_use]
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        let gpu_type = match (&self.gpu_type, &other.gpu_type) {
            (Some(a), Some(b)) if a != b && self.gpus > 0 && other.gpus > 0 => return None,
            (Some(a), _) => Some(a.clone()),
            (_, Some(b)) => Some(b.clone()),
            _ => None,
        };
        Some(Self {
            cpus: self.cpus.checked_add(other.cpus)?,
            memory_mib: self.memory_mib.checked_add(other.memory_mib)?,
            gpu_type,
            gpus: self.gpus.checked_add(other.gpus)?,
        })
    }

    #[must_use]
    pub fn saturating_sub(&self, other: &Self) -> Self {
        Self {
            cpus: self.cpus.saturating_sub(other.cpus),
            memory_mib: self.memory_mib.saturating_sub(other.memory_mib),
            gpu_type: self.gpu_type.clone(),
            gpus: self.gpus.saturating_sub(other.gpus),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobSpec {
    pub name: String,
    pub user: String,
    pub account: String,
    pub qos: String,
    pub partition: String,
    pub resources: Tres,
    pub time_limit_ms: u64,
    pub command: String,
    pub workload_id: String,
    pub dependency_after_ok: Option<JobId>,
    pub array_index: Option<u32>,
    pub output_path: Option<String>,
    pub error_path: Option<String>,
}

impl Default for JobSpec {
    fn default() -> Self {
        Self {
            name: "interactive".into(),
            user: "learner".into(),
            account: "research".into(),
            qos: "normal".into(),
            partition: "gpu".into(),
            resources: Tres { cpus: 1, memory_mib: 1_024, gpu_type: None, gpus: 0 },
            time_limit_ms: 30 * 60 * 1_000,
            command: "bash".into(),
            workload_id: "interactive-shell-v1".into(),
            dependency_after_ok: None,
            array_index: None,
            output_path: None,
            error_path: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Submitted,
    Pending,
    Running,
    Completing,
    Completed,
    Failed,
    Cancelled,
    Timeout,
    OutOfMemory,
    NodeFail,
    Preempted,
}

impl JobStatus {
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed
                | Self::Failed
                | Self::Cancelled
                | Self::Timeout
                | Self::OutOfMemory
                | Self::NodeFail
                | Self::Preempted
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PendingReason {
    Resources,
    Priority,
    Dependency,
    InvalidAccount,
    QosMaxJobsPerUserLimit,
    QosMaxGresPerUser,
    Reservation,
    PartitionDown,
    None,
}

impl PendingReason {
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Resources => "Resources",
            Self::Priority => "Priority",
            Self::Dependency => "Dependency",
            Self::InvalidAccount => "InvalidAccount",
            Self::QosMaxJobsPerUserLimit => "QOSMaxJobsPerUserLimit",
            Self::QosMaxGresPerUser => "QOSMaxGRESPerUser",
            Self::Reservation => "Reservation",
            Self::PartitionDown => "PartitionDown",
            Self::None => "None",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Allocation {
    pub node_id: String,
    pub cpus: u32,
    pub memory_mib: u64,
    pub gpu_indices: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobStep {
    pub id: StepId,
    pub name: String,
    pub status: JobStatus,
    pub started_at: Option<SimTimeMs>,
    pub ended_at: Option<SimTimeMs>,
    pub exit_code: Option<(u8, u8)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobRecord {
    pub id: JobId,
    pub spec: JobSpec,
    pub status: JobStatus,
    pub pending_reason: PendingReason,
    pub submitted_at: SimTimeMs,
    pub eligible_at: SimTimeMs,
    pub started_at: Option<SimTimeMs>,
    pub ended_at: Option<SimTimeMs>,
    pub allocation: Option<Allocation>,
    pub exit_code: Option<(u8, u8)>,
    pub steps: Vec<JobStep>,
    pub stdout_path: String,
    pub stderr_path: String,
}

impl JobRecord {
    #[must_use]
    pub fn elapsed_ms(&self, now: SimTimeMs) -> u64 {
        self.started_at
            .map(|start| self.ended_at.unwrap_or(now).0.saturating_sub(start.0))
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuDevice {
    pub index: u16,
    pub model: String,
    pub allocated_to: Option<JobId>,
    pub health: GpuHealth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuHealth {
    Ok,
    Warning,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Idle,
    Mixed,
    Allocated,
    Draining,
    Drained,
    Down,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeState {
    pub id: String,
    pub capacity: Tres,
    pub allocated: Tres,
    pub gpus: Vec<GpuDevice>,
    pub status: NodeStatus,
    pub drain_reason: Option<String>,
    pub running_jobs: BTreeSet<JobId>,
}

impl NodeState {
    #[must_use]
    pub fn available(&self) -> Tres {
        self.capacity.saturating_sub(&self.allocated)
    }

    pub fn recompute_status(&mut self) {
        if matches!(self.status, NodeStatus::Down | NodeStatus::Draining | NodeStatus::Drained) {
            return;
        }
        self.status = if self.running_jobs.is_empty() {
            NodeStatus::Idle
        } else if self.allocated.cpus >= self.capacity.cpus
            || self.allocated.memory_mib >= self.capacity.memory_mib
            || self.allocated.gpus >= self.capacity.gpus
        {
            NodeStatus::Allocated
        } else {
            NodeStatus::Mixed
        };
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartitionStatus {
    Up,
    Down,
    Inactive,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionState {
    pub id: String,
    pub node_ids: Vec<String>,
    pub is_default: bool,
    pub status: PartitionStatus,
    pub max_time_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QosPolicy {
    pub id: String,
    pub max_running_jobs_per_user: Option<u32>,
    pub max_gpus_per_user: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterState {
    pub id: String,
    pub teaching_version: String,
    pub nodes: BTreeMap<String, NodeState>,
    pub partitions: BTreeMap<String, PartitionState>,
    pub qos: BTreeMap<String, QosPolicy>,
}

impl ClusterState {
    #[must_use]
    pub fn dgx_h200_8() -> Self {
        let node_id = "dgx-h200-01".to_string();
        let capacity = Tres {
            cpus: 224,
            memory_mib: 1_857_528,
            gpu_type: Some("h200".into()),
            gpus: 8,
        };
        let gpus = (0..8)
            .map(|index| GpuDevice {
                index,
                model: "H200".into(),
                allocated_to: None,
                health: GpuHealth::Ok,
            })
            .collect();
        let node = NodeState {
            id: node_id.clone(),
            capacity,
            allocated: Tres {
                cpus: 0,
                memory_mib: 0,
                gpu_type: Some("h200".into()),
                gpus: 0,
            },
            gpus,
            status: NodeStatus::Idle,
            drain_reason: None,
            running_jobs: BTreeSet::new(),
        };
        let partition = PartitionState {
            id: "gpu".into(),
            node_ids: vec![node_id],
            is_default: true,
            status: PartitionStatus::Up,
            max_time_ms: Some(7 * 24 * 60 * 60 * 1_000),
        };
        let qos = QosPolicy {
            id: "normal".into(),
            max_running_jobs_per_user: None,
            max_gpus_per_user: None,
        };
        Self {
            id: "dgx-h200-8".into(),
            teaching_version: "25.05".into(),
            nodes: BTreeMap::from([("dgx-h200-01".into(), node)]),
            partitions: BTreeMap::from([("gpu".into(), partition)]),
            qos: BTreeMap::from([("normal".into(), qos)]),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountingRecord {
    pub job_id: JobId,
    pub user: String,
    pub account: String,
    pub state: JobStatus,
    pub requested: Tres,
    pub allocation: Option<Allocation>,
    pub submit_time: SimTimeMs,
    pub start_time: Option<SimTimeMs>,
    pub end_time: Option<SimTimeMs>,
    pub elapsed_ms: u64,
    pub exit_code: Option<(u8, u8)>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ModelError {
    #[error("job {0} not found")]
    JobNotFound(JobId),
    #[error("node {0} not found")]
    NodeNotFound(String),
    #[error("partition {0} not found")]
    PartitionNotFound(String),
    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidTransition { from: JobStatus, to: JobStatus },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_matches_generic_dgx_shape() {
        let cluster = ClusterState::dgx_h200_8();
        let node = &cluster.nodes["dgx-h200-01"];
        assert_eq!(node.capacity.cpus, 224);
        assert_eq!(node.capacity.gpus, 8);
        assert_eq!(node.gpus.len(), 8);
    }

    #[test]
    fn terminal_statuses_are_terminal() {
        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::OutOfMemory.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
    }
}
