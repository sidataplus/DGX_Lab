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
            assertion: Assertion::VirtualFileExists {
                path: "/home/learner/train.sbatch".into(),
            },
        },
        PracticalCheck {
            id: "submit-batch".into(),
            points: 30,
            critical: false,
            assertion: Assertion::AnyCommandUsed {
                prefixes: vec!["sbatch".into()],
            },
        },
        PracticalCheck {
            id: "inspect-logs".into(),
            points: 20,
            critical: false,
            assertion: Assertion::AnyCommandUsed {
                prefixes: vec!["cat".into(), "tail".into(), "ls".into()],
            },
        },
        PracticalCheck {
            id: "accounting".into(),
            points: 30,
            critical: true,
            assertion: Assertion::AnyCommandUsed {
                prefixes: vec!["sacct".into()],
            },
        },
        // Bonus signal: batch job reached a terminal or running state.
        PracticalCheck {
            id: "job-progress".into(),
            points: 0,
            critical: false,
            assertion: Assertion::LearnerJobVisitedState {
                state: JobStatus::Running,
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
