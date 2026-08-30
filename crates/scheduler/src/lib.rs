#![forbid(unsafe_code)]

//! Deterministic FIFO/resource scheduler for DGX Lab P0 behavior.

use dgxlab_contracts::{JobId, SimTimeMs};
use serde::{Deserialize, Serialize};
use slurm_model::{
    Allocation, ClusterState, GpuHealth, JobRecord, JobSpec, JobStatus, NodeStatus,
    PartitionStatus, PendingReason, Tres,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartDecision {
    pub job_id: JobId,
    pub allocation: Allocation,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleResult {
    pub started: Vec<StartDecision>,
    pub pending_updates: Vec<(JobId, PendingReason)>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("partition not found: {0}")]
    PartitionNotFound(String),
    #[error("partition is not up: {0}")]
    PartitionUnavailable(String),
    #[error("QOS not found: {0}")]
    QosNotFound(String),
    #[error("request exceeds every node in partition {partition}: {requested:?}")]
    RequestUnsatisfiable { partition: String, requested: Tres },
    #[error("time limit exceeds partition maximum")]
    TimeLimitTooLong,
    #[error("resource request must include at least one CPU")]
    ZeroCpu,
}

pub fn validate_job(cluster: &ClusterState, spec: &JobSpec) -> Result<(), ValidationError> {
    if spec.resources.cpus == 0 {
        return Err(ValidationError::ZeroCpu);
    }
    let partition = cluster
        .partitions
        .get(&spec.partition)
        .ok_or_else(|| ValidationError::PartitionNotFound(spec.partition.clone()))?;
    if partition.status != PartitionStatus::Up {
        return Err(ValidationError::PartitionUnavailable(spec.partition.clone()));
    }
    if !cluster.qos.contains_key(&spec.qos) {
        return Err(ValidationError::QosNotFound(spec.qos.clone()));
    }
    if partition.max_time_ms.is_some_and(|max_time| spec.time_limit_ms > max_time) {
        return Err(ValidationError::TimeLimitTooLong);
    }
    let satisfiable = partition
        .node_ids
        .iter()
        .filter_map(|id| cluster.nodes.get(id))
        .any(|node| spec.resources.fits_within(&node.capacity));
    if !satisfiable {
        return Err(ValidationError::RequestUnsatisfiable {
            partition: spec.partition.clone(),
            requested: spec.resources.clone(),
        });
    }
    Ok(())
}

pub fn schedule_pending(
    cluster: &mut ClusterState,
    jobs: &mut BTreeMap<JobId, JobRecord>,
    now: SimTimeMs,
) -> ScheduleResult {
    let mut result = ScheduleResult::default();
    let mut pending: Vec<JobId> = jobs
        .iter()
        .filter_map(|(id, job)| (job.status == JobStatus::Pending).then_some(*id))
        .collect();
    pending.sort_by_key(|id| {
        let job = &jobs[id];
        (job.eligible_at, job.submitted_at, *id)
    });

    for job_id in pending {
        let reason = eligibility_reason(cluster, jobs, job_id, now);
        if reason != PendingReason::None {
            if let Some(job) = jobs.get_mut(&job_id) {
                job.pending_reason = reason;
            }
            result.pending_updates.push((job_id, reason));
            continue;
        }

        let Some(spec) = jobs.get(&job_id).map(|job| job.spec.clone()) else {
            continue;
        };
        let Some(allocation) = find_allocation(cluster, &spec) else {
            if let Some(job) = jobs.get_mut(&job_id) {
                job.pending_reason = PendingReason::Resources;
            }
            result.pending_updates.push((job_id, PendingReason::Resources));
            continue;
        };
        apply_allocation(cluster, job_id, &spec.resources, &allocation);
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = JobStatus::Running;
            job.pending_reason = PendingReason::None;
            job.started_at = Some(now);
            job.allocation = Some(allocation.clone());
        }
        result.started.push(StartDecision { job_id, allocation });
    }
    result
}

fn eligibility_reason(
    cluster: &ClusterState,
    jobs: &BTreeMap<JobId, JobRecord>,
    job_id: JobId,
    _now: SimTimeMs,
) -> PendingReason {
    let Some(job) = jobs.get(&job_id) else {
        return PendingReason::Priority;
    };
    let Some(partition) = cluster.partitions.get(&job.spec.partition) else {
        return PendingReason::PartitionDown;
    };
    if partition.status != PartitionStatus::Up {
        return PendingReason::PartitionDown;
    }
    if let Some(dependency) = job.spec.dependency_after_ok {
        match jobs.get(&dependency).map(|candidate| candidate.status) {
            Some(JobStatus::Completed) => {}
            _ => return PendingReason::Dependency,
        }
    }
    let Some(qos) = cluster.qos.get(&job.spec.qos) else {
        return PendingReason::InvalidAccount;
    };
    if let Some(max_jobs) = qos.max_running_jobs_per_user {
        let running = jobs
            .values()
            .filter(|candidate| {
                candidate.spec.user == job.spec.user && candidate.status == JobStatus::Running
            })
            .count() as u32;
        if running >= max_jobs {
            return PendingReason::QosMaxJobsPerUserLimit;
        }
    }
    if let Some(max_gpus) = qos.max_gpus_per_user {
        let allocated_gpus: u16 = jobs
            .values()
            .filter(|candidate| {
                candidate.spec.user == job.spec.user && candidate.status == JobStatus::Running
            })
            .map(|candidate| candidate.spec.resources.gpus)
            .sum();
        if allocated_gpus.saturating_add(job.spec.resources.gpus) > max_gpus {
            return PendingReason::QosMaxGresPerUser;
        }
    }
    PendingReason::None
}

fn find_allocation(cluster: &ClusterState, spec: &JobSpec) -> Option<Allocation> {
    let partition = cluster.partitions.get(&spec.partition)?;
    for node_id in &partition.node_ids {
        let Some(node) = cluster.nodes.get(node_id) else {
            // A stale or malformed partition reference must not prevent a later,
            // valid node from satisfying the request.
            continue;
        };
        if matches!(node.status, NodeStatus::Down | NodeStatus::Draining | NodeStatus::Drained) {
            continue;
        }
        if !spec.resources.fits_within(&node.available()) {
            continue;
        }
        let gpu_indices: Vec<u16> = node
            .gpus
            .iter()
            .filter(|gpu| gpu.allocated_to.is_none() && gpu.health != GpuHealth::Failed)
            .take(spec.resources.gpus as usize)
            .map(|gpu| gpu.index)
            .collect();
        if gpu_indices.len() != spec.resources.gpus as usize {
            continue;
        }
        return Some(Allocation {
            node_id: node_id.clone(),
            cpus: spec.resources.cpus,
            memory_mib: spec.resources.memory_mib,
            gpu_indices,
        });
    }
    None
}

fn apply_allocation(
    cluster: &mut ClusterState,
    job_id: JobId,
    requested: &Tres,
    allocation: &Allocation,
) {
    let node = cluster.nodes.get_mut(&allocation.node_id).expect("allocation node is known");
    node.allocated =
        node.allocated.checked_add(requested).expect("scheduler only combines compatible TRES");
    for index in &allocation.gpu_indices {
        if let Some(gpu) = node.gpus.iter_mut().find(|gpu| gpu.index == *index) {
            gpu.allocated_to = Some(job_id);
        }
    }
    node.running_jobs.insert(job_id);
    node.recompute_status();
}

pub fn release_allocation(cluster: &mut ClusterState, job_id: JobId, allocation: &Allocation) {
    if let Some(node) = cluster.nodes.get_mut(&allocation.node_id) {
        let released = Tres {
            cpus: allocation.cpus,
            memory_mib: allocation.memory_mib,
            gpu_type: node.capacity.gpu_type.clone(),
            gpus: allocation.gpu_indices.len() as u16,
        };
        node.allocated = node.allocated.saturating_sub(&released);
        for index in &allocation.gpu_indices {
            if let Some(gpu) = node.gpus.iter_mut().find(|gpu| gpu.index == *index)
                && gpu.allocated_to == Some(job_id)
            {
                gpu.allocated_to = None;
            }
        }
        node.running_jobs.remove(&job_id);
        node.recompute_status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dgxlab_contracts::JobId;

    fn job(id: u64, gpus: u16) -> JobRecord {
        let spec = JobSpec {
            resources: Tres { cpus: 8, memory_mib: 65_536, gpu_type: Some("h200".into()), gpus },
            ..JobSpec::default()
        };
        JobRecord {
            id: JobId(id),
            spec,
            status: JobStatus::Pending,
            pending_reason: PendingReason::Priority,
            submitted_at: SimTimeMs(id),
            eligible_at: SimTimeMs(id),
            started_at: None,
            ended_at: None,
            allocation: None,
            exit_code: None,
            steps: vec![],
            stdout_path: format!("/home/learner/slurm-{id}.out"),
            stderr_path: format!("/home/learner/slurm-{id}.err"),
        }
    }

    #[test]
    fn fifo_jobs_allocate_distinct_gpus() {
        let mut cluster = ClusterState::dgx_h200_8();
        let mut jobs = BTreeMap::from([(JobId(1), job(1, 1)), (JobId(2), job(2, 1))]);
        let result = schedule_pending(&mut cluster, &mut jobs, SimTimeMs(10));
        assert_eq!(result.started.len(), 2);
        assert_ne!(
            jobs[&JobId(1)].allocation.as_ref().unwrap().gpu_indices,
            jobs[&JobId(2)].allocation.as_ref().unwrap().gpu_indices
        );
    }

    #[test]
    fn stale_partition_node_reference_does_not_hide_valid_node() {
        let mut cluster = ClusterState::dgx_h200_8();
        let partition = cluster.partitions.get_mut("gpu").unwrap();
        partition.node_ids.insert(0, "missing-node".into());
        let spec = job(1, 1).spec;
        let allocation = find_allocation(&cluster, &spec).expect("valid node remains allocatable");
        assert_eq!(allocation.node_id, "dgx-h200-01");
    }

    #[test]
    fn unsatisfied_job_stays_pending_for_resources() {
        let mut cluster = ClusterState::dgx_h200_8();
        let mut jobs = BTreeMap::new();
        jobs.insert(JobId(1), job(1, 8));
        jobs.insert(JobId(2), job(2, 1));
        let result = schedule_pending(&mut cluster, &mut jobs, SimTimeMs(10));
        assert_eq!(result.started.len(), 1);
        assert_eq!(jobs[&JobId(2)].pending_reason, PendingReason::Resources);
    }
}
