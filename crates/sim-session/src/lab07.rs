//! Lab 07 practical checks: contention and pending reasons.

use grading::{Assertion, PracticalCheck};
use slurm_model::{JobStatus, PendingReason};

pub const LAB07_ID: &str = "07-pending-reasons";

#[must_use]
pub fn lab07_checks() -> Vec<PracticalCheck> {
    vec![
        PracticalCheck {
            id: "submit".into(),
            points: 25,
            critical: false,
            assertion: Assertion::LearnerJobMatches {
                name: None,
                gpus: Some(1),
                cpus: None,
                min_memory_mib: None,
                max_memory_mib: None,
                states: vec![JobStatus::Submitted],
            },
        },
        PracticalCheck {
            id: "observe-pending".into(),
            points: 25,
            critical: false,
            assertion: Assertion::LearnerJobMatches {
                name: None,
                gpus: Some(1),
                cpus: None,
                min_memory_mib: None,
                max_memory_mib: None,
                states: vec![JobStatus::Pending],
            },
        },
        PracticalCheck {
            id: "inspect-reason".into(),
            points: 25,
            critical: false,
            assertion: Assertion::JobInspectionWhilePending {
                pending_reason: PendingReason::Resources,
                gpus: Some(1),
                cpus: None,
                min_memory_mib: None,
                max_memory_mib: None,
                states: vec![JobStatus::Pending, JobStatus::Running],
            },
        },
        PracticalCheck {
            id: "start-after-wait".into(),
            points: 25,
            critical: true,
            assertion: Assertion::LearnerJobMatches {
                name: None,
                gpus: Some(1),
                cpus: None,
                min_memory_mib: None,
                max_memory_mib: None,
                states: vec![JobStatus::Pending, JobStatus::Running],
            },
        },
    ]
}

#[must_use]
pub fn lab07_hints() -> [&'static str; 3] {
    [
        "Submit a one-GPU job while virtual users occupy the node (try `sbatch train.sbatch`).",
        "Use `squeue` and `scontrol show job <id>` to read the pending reason.",
        "Advance simulation time until resources free and the job starts (clock controls).",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_job_must_be_inspected_during_resources_wait_then_start() {
        let checks = lab07_checks();
        assert!(matches!(
            &checks[2].assertion,
            Assertion::JobInspectionWhilePending {
                pending_reason: PendingReason::Resources,
                gpus: Some(1),
                states,
                ..
            } if states == &vec![JobStatus::Pending, JobStatus::Running]
        ));
        assert!(matches!(
            &checks[3].assertion,
            Assertion::LearnerJobMatches {
                gpus: Some(1),
                states,
                ..
            } if states == &vec![JobStatus::Pending, JobStatus::Running]
        ));
    }
}
