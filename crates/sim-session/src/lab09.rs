//! Lab 09: diagnose failure and resume from checkpoint.

use grading::{Assertion, PracticalCheck};
use slurm_model::JobStatus;

pub const LAB09_ID: &str = "09-failure-resume";

#[must_use]
pub fn lab09_checks() -> Vec<PracticalCheck> {
    vec![
        PracticalCheck {
            id: "sacct".into(),
            points: 25,
            critical: false,
            assertion: Assertion::AnyCommandUsed {
                prefixes: vec!["sacct".into()],
            },
        },
        PracticalCheck {
            id: "inspect-logs".into(),
            points: 25,
            critical: false,
            assertion: Assertion::AnyCommandUsed {
                prefixes: vec!["cat".into(), "tail".into(), "ls".into()],
            },
        },
        PracticalCheck {
            id: "checkpoint".into(),
            points: 25,
            critical: false,
            assertion: Assertion::VirtualFileExists {
                // At least one checkpoint root used by synthetic workloads.
                path: "/home/learner/checkpoints".into(),
            },
        },
        PracticalCheck {
            id: "oom-observed".into(),
            points: 25,
            critical: true,
            assertion: Assertion::LearnerJobVisitedState {
                state: JobStatus::OutOfMemory,
            },
        },
    ]
}

#[must_use]
pub fn lab09_hints() -> [&'static str; 3] {
    [
        "Start with `sacct` to find jobs that ended OUT_OF_MEMORY.",
        "Inspect logs under `logs/` and list `checkpoints/` for resume candidates.",
        "Resubmit with lower batch size or more memory, pointing at the newest checkpoint.",
    ]
}
