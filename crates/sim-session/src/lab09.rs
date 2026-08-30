//! Lab 09: diagnose failure and resume from checkpoint.

use grading::{Assertion, PracticalCheck};
use slurm_model::JobStatus;

pub const LAB09_ID: &str = "09-failure-resume";

#[must_use]
pub fn lab09_checks() -> Vec<PracticalCheck> {
    vec![
        PracticalCheck {
            id: "sacct".into(),
            points: 8,
            critical: false,
            assertion: command_after_train_failure(&["sacct"]),
        },
        PracticalCheck {
            id: "inspect-logs".into(),
            points: 21,
            critical: false,
            assertion: command_after_train_failure(&["tail -n"]),
        },
        PracticalCheck {
            id: "oom-observed".into(),
            points: 21,
            critical: false,
            assertion: command_after_train_failure(&["scontrol show job"]),
        },
        PracticalCheck {
            id: "checkpoint".into(),
            points: 10,
            critical: false,
            assertion: command_after_train_failure(&["cat checkpoints/"]),
        },
        PracticalCheck {
            id: "resume-submitted".into(),
            points: 40,
            critical: true,
            assertion: Assertion::LearnerRecoveryJobCompleted {
                name: "train-resume".into(),
                partition: Some("gpu".into()),
                gpus: 4,
                cpus: 16,
                minimum_memory_mib: 64 * 1024,
                maximum_memory_mib: Some(64 * 1024),
                minimum_time_limit_ms: 2 * 60 * 60 * 1_000,
                maximum_time_limit_ms: Some(2 * 60 * 60 * 1_000),
                after_job_name: "train-llm".into(),
            },
        },
    ]
}

fn command_after_train_failure(prefixes: &[&str]) -> Assertion {
    Assertion::CommandAfterMatchingJobState {
        prefixes: prefixes.iter().map(|prefix| (*prefix).into()).collect(),
        name: Some("train-llm".into()),
        gpus: Some(4),
        cpus: Some(16),
        min_memory_mib: Some(32 * 1024),
        max_memory_mib: Some(32 * 1024),
        state: JobStatus::OutOfMemory,
    }
}

#[must_use]
pub fn lab09_hints() -> [&'static str; 3] {
    [
        "Start with `sacct` to find jobs that ended OUT_OF_MEMORY.",
        "Inspect logs under `logs/` and list `checkpoints/` for resume candidates.",
        "Resubmit with lower batch size or more memory, pointing at the newest checkpoint.",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_preserves_the_failed_job_shape_and_latest_checkpoint() {
        let checks = lab09_checks();
        assert!(checks[1].points > 20, "log inspection must not be skippable at 80%");
        assert!(checks[2].points > 20, "diagnosis must not be skippable at 80%");
        assert!(matches!(
            &checks[4].assertion,
            Assertion::LearnerRecoveryJobCompleted {
                name,
                partition: Some(partition),
                gpus: 4,
                cpus: 16,
                minimum_memory_mib: 65_536,
                maximum_memory_mib: Some(65_536),
                minimum_time_limit_ms: 7_200_000,
                maximum_time_limit_ms: Some(7_200_000),
                after_job_name,
            } if name == "train-resume" && partition == "gpu" && after_job_name == "train-llm"
        ));
    }
}
