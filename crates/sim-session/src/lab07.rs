//! Lab 07 practical checks: contention and pending reasons.

use grading::{Assertion, PracticalCheck};
use slurm_model::JobStatus;

pub const LAB07_ID: &str = "07-pending-reasons";

#[must_use]
pub fn lab07_checks() -> Vec<PracticalCheck> {
    vec![
        PracticalCheck {
            id: "submit".into(),
            points: 25,
            critical: false,
            assertion: Assertion::LearnerJobExists {
                gpus: Some(1),
                cpus: None,
                max_memory_mib: None,
            },
        },
        PracticalCheck {
            id: "observe-pending".into(),
            points: 25,
            critical: false,
            assertion: Assertion::AnyCommandUsed {
                prefixes: vec!["squeue".into()],
            },
        },
        PracticalCheck {
            id: "inspect-reason".into(),
            points: 25,
            critical: false,
            assertion: Assertion::AnyCommandUsed {
                prefixes: vec!["scontrol".into()],
            },
        },
        PracticalCheck {
            id: "start-after-wait".into(),
            points: 25,
            critical: true,
            assertion: Assertion::LearnerJobVisitedState {
                state: JobStatus::Running,
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
