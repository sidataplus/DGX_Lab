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
            assertion: Assertion::AnyCommandUsed { prefixes: vec!["sinfo".into()] },
        },
        PracticalCheck {
            id: "allocate".into(),
            points: 25,
            critical: false,
            assertion: Assertion::LearnerJobMatches {
                name: None,
                gpus: Some(1),
                cpus: Some(8),
                min_memory_mib: Some(64 * 1024),
                max_memory_mib: Some(64 * 1024),
                states: vec![JobStatus::Running],
            },
        },
        PracticalCheck {
            id: "verify-env".into(),
            points: 20,
            critical: true,
            assertion: Assertion::CommandInMatchingAllocation {
                prefixes: vec!["echo $CUDA_VISIBLE_DEVICES".into()],
                gpus: Some(1),
                cpus: Some(8),
                min_memory_mib: Some(64 * 1024),
                max_memory_mib: Some(64 * 1024),
                states: vec![JobStatus::Running],
            },
        },
        PracticalCheck {
            id: "verify-gpu".into(),
            points: 20,
            critical: true,
            assertion: Assertion::CommandInMatchingAllocation {
                prefixes: vec!["nvidia-smi".into()],
                gpus: Some(1),
                cpus: Some(8),
                min_memory_mib: Some(64 * 1024),
                max_memory_mib: Some(64 * 1024),
                states: vec![JobStatus::Running],
            },
        },
        PracticalCheck {
            id: "release".into(),
            points: 20,
            critical: true,
            assertion: Assertion::All {
                assertions: vec![
                    Assertion::LearnerJobMatches {
                        name: None,
                        gpus: Some(1),
                        cpus: Some(8),
                        min_memory_mib: Some(64 * 1024),
                        max_memory_mib: Some(64 * 1024),
                        states: vec![JobStatus::Running, JobStatus::Completed],
                    },
                    Assertion::CommandInMatchingAllocation {
                        prefixes: vec!["echo $CUDA_VISIBLE_DEVICES".into()],
                        gpus: Some(1),
                        cpus: Some(8),
                        min_memory_mib: Some(64 * 1024),
                        max_memory_mib: Some(64 * 1024),
                        states: vec![JobStatus::Running, JobStatus::Completed],
                    },
                    Assertion::CommandInMatchingAllocation {
                        prefixes: vec!["nvidia-smi".into()],
                        gpus: Some(1),
                        cpus: Some(8),
                        min_memory_mib: Some(64 * 1024),
                        max_memory_mib: Some(64 * 1024),
                        states: vec![JobStatus::Running, JobStatus::Completed],
                    },
                    Assertion::ActiveAllocationReleased,
                ],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_completed_allocation_owns_both_visibility_checks() {
        let checks = lab04_checks();
        assert!(matches!(
            &checks[1].assertion,
            Assertion::LearnerJobMatches {
                gpus: Some(1),
                cpus: Some(8),
                min_memory_mib: Some(65_536),
                max_memory_mib: Some(65_536),
                states,
                ..
            } if states.contains(&JobStatus::Running)
        ));
        assert!(matches!(
            &checks[2].assertion,
            Assertion::CommandInMatchingAllocation {
                gpus: Some(1),
                cpus: Some(8),
                min_memory_mib: Some(65_536),
                max_memory_mib: Some(65_536),
                ..
            }
        ));
        assert!(matches!(&checks[3].assertion, Assertion::CommandInMatchingAllocation { .. }));
        assert!(matches!(&checks[4].assertion, Assertion::All { .. }));
    }
}
