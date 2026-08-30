//! Lab 06 practical checks: batch submission and accounting.

use grading::{Assertion, PracticalCheck};
use slurm_model::JobStatus;

pub const LAB06_ID: &str = "06-batch-jobs";

#[must_use]
pub fn lab06_checks() -> Vec<PracticalCheck> {
    vec![
        PracticalCheck {
            id: "edit-script".into(),
            points: 20,
            critical: false,
            assertion: Assertion::VirtualFileWritten { path: "/home/learner/train.sbatch".into() },
        },
        PracticalCheck {
            id: "submit-batch".into(),
            points: 30,
            critical: false,
            assertion: Assertion::LearnerJobMatches {
                name: None,
                gpus: None,
                cpus: None,
                min_memory_mib: None,
                max_memory_mib: None,
                states: vec![JobStatus::Submitted],
            },
        },
        PracticalCheck {
            id: "job-progress".into(),
            points: 0,
            critical: true,
            assertion: Assertion::LearnerJobMatches {
                name: None,
                gpus: None,
                cpus: None,
                min_memory_mib: None,
                max_memory_mib: None,
                states: vec![JobStatus::Completed],
            },
        },
        PracticalCheck {
            id: "inspect-logs".into(),
            points: 20,
            critical: true,
            assertion: Assertion::CommandAfterMatchingJobState {
                prefixes: vec!["tail -n".into()],
                name: None,
                gpus: None,
                cpus: None,
                min_memory_mib: None,
                max_memory_mib: None,
                state: JobStatus::Completed,
            },
        },
        PracticalCheck {
            id: "accounting".into(),
            points: 30,
            critical: true,
            assertion: Assertion::CommandAfterMatchingJobState {
                prefixes: vec!["sacct".into()],
                name: None,
                gpus: None,
                cpus: None,
                min_memory_mib: None,
                max_memory_mib: None,
                state: JobStatus::Completed,
            },
        },
    ]
}

#[must_use]
pub fn lab06_hints() -> [&'static str; 3] {
    [
        "Open `/home/learner/train.sbatch` in the script editor and confirm `#SBATCH` resources.",
        "Submit with `sbatch train.sbatch`, then watch the queue.",
        "After the job finishes (advance time if needed), inspect logs and run `sacct`.",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_script_and_early_observation_cannot_finish_the_lab() {
        let checks = lab06_checks();
        assert!(matches!(
            &checks[0].assertion,
            Assertion::VirtualFileWritten { path } if path == "/home/learner/train.sbatch"
        ));
        assert!(checks[3].critical);
        assert!(matches!(
            &checks[3].assertion,
            Assertion::CommandAfterMatchingJobState { state: JobStatus::Completed, .. }
        ));
        assert!(matches!(
            &checks[4].assertion,
            Assertion::CommandAfterMatchingJobState { state: JobStatus::Completed, .. }
        ));
    }
}
