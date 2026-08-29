//! Built-in course lab → scenario mapping and lightweight practical checks.

use grading::{Assertion, PracticalCheck};

#[derive(Clone, Copy, Debug)]
pub struct LabMeta {
    pub id: &'static str,
    pub title: &'static str,
    pub scenario: &'static str,
}

pub const COURSE_LABS: &[LabMeta] = &[
    LabMeta {
        id: "01-cluster-mental-model",
        title: "Cluster Mental Model",
        scenario: "dgx-h200-8",
    },
    LabMeta {
        id: "02-interactive-cpu",
        title: "Interactive CPU Allocation",
        scenario: "dgx-h200-8",
    },
    LabMeta {
        id: "03-cpu-memory",
        title: "CPU and Memory Requests",
        scenario: "dgx-h200-8",
    },
    LabMeta {
        id: "04-one-gpu",
        title: "One GPU Allocation",
        scenario: "guided-one-gpu",
    },
    LabMeta {
        id: "05-containers",
        title: "Containers (Synthetic)",
        scenario: "dgx-h200-8",
    },
    LabMeta {
        id: "06-batch-jobs",
        title: "Batch Jobs",
        scenario: "dgx-h200-8",
    },
    LabMeta {
        id: "07-pending-reasons",
        title: "Pending Reasons",
        scenario: "dgx-contended",
    },
    LabMeta {
        id: "08-arrays-dependencies",
        title: "Arrays and Dependencies",
        scenario: "dgx-h200-8",
    },
    LabMeta {
        id: "09-failure-resume",
        title: "Failure and Resume",
        scenario: "dgx-degraded",
    },
    LabMeta {
        id: "10-multi-gpu",
        title: "Multi-GPU Workloads",
        scenario: "dgx-h200-8",
    },
    LabMeta {
        id: "11-policy-efficiency",
        title: "Policy and Efficiency",
        scenario: "dgx-h200-8",
    },
    LabMeta {
        id: "12-capstone",
        title: "Capstone Campaign",
        scenario: "dgx-h200-8",
    },
];

#[must_use]
pub fn lab_for_scenario(scenario_id: &str) -> Option<&'static str> {
    match scenario_id {
        "guided-one-gpu" => Some("04-one-gpu"),
        "dgx-contended" | "pending-gpu-contention-01" => Some("07-pending-reasons"),
        "dgx-degraded" | "failure-resume-01" => Some("09-failure-resume"),
        "dgx-h200-8" => Some("06-batch-jobs"),
        _ => None,
    }
}

/// Generic command-prefix checks used for labs without dedicated assertion packs.
#[must_use]
pub fn generic_lab_checks(lab_id: &str) -> Vec<PracticalCheck> {
    let prefixes: &[&str] = match lab_id {
        "01-cluster-mental-model" => &["sinfo", "squeue"],
        "02-interactive-cpu" => &["srun", "salloc", "exit"],
        "03-cpu-memory" => &["srun", "sbatch", "scontrol"],
        "05-containers" => &["singularity", "module", "srun"],
        "08-arrays-dependencies" => &["sbatch", "squeue", "scontrol"],
        "10-multi-gpu" => &["srun", "torchrun", "sbatch"],
        "11-policy-efficiency" => &["sacct", "squeue", "scontrol"],
        "12-capstone" => &["sbatch", "squeue", "sacct"],
        _ => &["sinfo"],
    };
    prefixes
        .iter()
        .enumerate()
        .map(|(index, prefix)| PracticalCheck {
            id: format!("step-{index}"),
            points: 100 / prefixes.len() as u32,
            critical: index == prefixes.len() - 1,
            assertion: Assertion::AnyCommandUsed {
                prefixes: vec![(*prefix).into()],
            },
        })
        .collect()
}

#[must_use]
pub fn generic_hints(lab_id: &str) -> Vec<&'static str> {
    match lab_id {
        "01-cluster-mental-model" => vec![
            "Run `sinfo` and `squeue` to map partitions and jobs.",
            "Compare idle vs mixed node state as jobs arrive.",
            "Open the command reference for sinfo flags used in this course.",
        ],
        "08-arrays-dependencies" => vec![
            "Submit with `#SBATCH --array=1-3` or `--dependency=afterok:<jobid>`.",
            "Watch array tasks in `squeue` and `sacct`.",
            "Confirm dependents only start after successful parents.",
        ],
        "10-multi-gpu" => vec![
            "Request multiple GPUs with `--gres=gpu:h200:4`.",
            "Use synthetic `torchrun` only inside an allocation.",
            "Verify isolation with `nvidia-smi -L`.",
        ],
        "12-capstone" => vec![
            "Compose a small campaign: batch + optional dependency.",
            "Use time acceleration to observe completion.",
            "Collect sacct evidence for the campaign.",
        ],
        _ => vec![
            "Inspect state before changing resources.",
            "Prefer read-only scheduler commands first.",
            "Use hints sparingly; evidence tracks hint use separately.",
        ],
    }
}
