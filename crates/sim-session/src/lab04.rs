//! Built-in Lab 04 practical checks (aligned with course-src lab.yaml objectives).

use grading::{Assertion, PracticalCheck};
use slurm_model::JobStatus;

pub const LAB04_ID: &str = "04-one-gpu";

#[must_use]
pub fn lab04_checks() -> Vec<PracticalCheck> {
    vec![
        PracticalCheck {
            id: "inspect".into(),
            points: 15,
            critical: false,
            assertion: Assertion::AnyCommandUsed {
                prefixes: vec!["sinfo".into()],
            },
        },
        PracticalCheck {
            id: "allocate".into(),
            points: 25,
            critical: false,
            assertion: Assertion::LearnerJobExists {
                gpus: Some(1),
                cpus: Some(8),
                max_memory_mib: Some(64 * 1024),
            },
        },
        PracticalCheck {
            id: "verify-env".into(),
            points: 20,
            critical: true,
            assertion: Assertion::AnyCommandUsed {
                prefixes: vec!["echo".into(), "env".into()],
            },
        },
        PracticalCheck {
            id: "verify-gpu".into(),
            points: 20,
            critical: true,
            assertion: Assertion::AnyCommandUsed {
                prefixes: vec!["nvidia-smi".into()],
            },
        },
        PracticalCheck {
            id: "release".into(),
            points: 20,
            critical: true,
            assertion: Assertion::LearnerJobVisitedState {
                state: JobStatus::Completed,
            },
        },
    ]
}

#[must_use]
pub fn lab04_hints() -> [&'static str; 3] {
    [
        "Start by inspecting partitions and node state with `sinfo`.",
        "Use `srun` with `--gres=gpu:h200:1`, CPU, memory, time, and `--pty bash`.",
        "Inside the allocation, inspect `$CUDA_VISIBLE_DEVICES` and run `nvidia-smi -L`; finish with `exit`.",
    ]
}
