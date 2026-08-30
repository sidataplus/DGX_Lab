//! Built-in course lab → scenario mapping and lightweight practical checks.

use grading::{Assertion, PracticalCheck};
use slurm_model::JobStatus;

#[derive(Clone, Copy, Debug)]
pub struct LabStepMeta {
    pub check_id: &'static str,
    pub label: &'static str,
    pub suggested_command: &'static str,
    pub evidence: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct LabMeta {
    pub id: &'static str,
    pub title: &'static str,
    pub scenario: &'static str,
    pub estimated_minutes: u16,
    pub track: &'static str,
    pub summary: &'static str,
    pub steps: &'static [LabStepMeta],
}

pub const COURSE_LABS: &[LabMeta] = &[
    LabMeta {
        id: "01-cluster-mental-model",
        title: "Cluster, Nodes, Partitions, Jobs, and Steps",
        scenario: "dgx-h200-8",
        estimated_minutes: 25,
        track: "Foundations",
        summary: "Map partitions, nodes, jobs, and steps before you request any resources.",
        steps: &[
            LabStepMeta {
                check_id: "step-0",
                label: "Inspect partitions and node state",
                suggested_command: "sinfo",
                evidence: "Find the default partition and distinguish idle from allocated capacity.",
            },
            LabStepMeta {
                check_id: "step-1",
                label: "Inspect the node's resource capacity",
                suggested_command: "scontrol show node dgx-h200-01",
                evidence: "Locate total and allocated CPU, memory, and GPU capacity in one node record.",
            },
        ],
    },
    LabMeta {
        id: "02-interactive-cpu",
        title: "Interactive CPU Allocation",
        scenario: "dgx-h200-8",
        estimated_minutes: 25,
        track: "Foundations",
        summary: "Move safely from the login node into a time-bounded interactive allocation.",
        steps: &[
            LabStepMeta {
                check_id: "step-0",
                label: "Request a small interactive allocation",
                suggested_command: "salloc --partition=gpu --cpus-per-task=4 --mem=8G --time=00:15:00",
                evidence: "The prompt and job queue should show that work moved into an allocation.",
            },
            LabStepMeta {
                check_id: "step-1",
                label: "Inspect the allocation environment",
                suggested_command: "env",
                evidence: "Match `SLURM_JOB_ID` and `SLURM_CPUS_PER_TASK=4` to this running allocation.",
            },
            LabStepMeta {
                check_id: "step-2",
                label: "Release the allocation cleanly",
                suggested_command: "exit",
                evidence: "The learner job should leave the running state and release its resources.",
            },
            LabStepMeta {
                check_id: "step-3",
                label: "Verify the completed allocation",
                suggested_command: "sacct",
                evidence: "Find the released allocation in accounting with a completed terminal state.",
            },
        ],
    },
    LabMeta {
        id: "03-cpu-memory",
        title: "CPU and Memory Requests",
        scenario: "dgx-h200-8",
        estimated_minutes: 30,
        track: "Foundations",
        summary: "Translate workload needs into CPU and memory requests the scheduler can reason about.",
        steps: &[
            LabStepMeta {
                check_id: "step-0",
                label: "Run a safe CPU preprocessing baseline",
                suggested_command: "srun --job-name=prep-ok --partition=gpu --cpus-per-task=8 --mem=64G --time=00:10:00 python preprocess.py --epochs 2",
                evidence: "Capture a baseline request with enough host memory to complete.",
            },
            LabStepMeta {
                check_id: "step-1",
                label: "Inspect the recorded resource request",
                suggested_command: "scontrol show job <jobid>",
                evidence: "Compare the recorded CPU and memory request with the intended baseline.",
            },
            LabStepMeta {
                check_id: "step-2",
                label: "Submit a low-memory comparison job",
                suggested_command: "srun --job-name=prep-oom --partition=gpu --cpus-per-task=8 --mem=16G --time=00:10:00 python preprocess.py --epochs 2",
                evidence: "A second learner job should record the intentionally constrained memory request.",
            },
            LabStepMeta {
                check_id: "step-3",
                label: "Observe and classify the host-memory OOM",
                suggested_command: "Use the +1 minute simulation control",
                evidence: "After time advances, accounting should show an out-of-memory terminal state.",
            },
        ],
    },
    LabMeta {
        id: "04-one-gpu",
        title: "One GPU Allocation and Isolation",
        scenario: "guided-one-gpu",
        estimated_minutes: 30,
        track: "Foundations",
        summary: "Request one GPU, verify isolation from inside the job, then release it cleanly.",
        steps: &[
            LabStepMeta {
                check_id: "inspect",
                label: "Inspect idle GPU capacity",
                suggested_command: "sinfo",
                evidence: "Confirm the compute node is available before asking the scheduler for a GPU.",
            },
            LabStepMeta {
                check_id: "allocate",
                label: "Request one GPU, 8 CPUs, and 64 GiB",
                suggested_command: "srun --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:30:00 --pty bash",
                evidence: "A learner job should run with exactly one allocated GPU tile.",
            },
            LabStepMeta {
                check_id: "verify-env",
                label: "Verify the job environment",
                suggested_command: "echo $CUDA_VISIBLE_DEVICES",
                evidence: "The environment should expose one locally remapped GPU index.",
            },
            LabStepMeta {
                check_id: "verify-gpu",
                label: "Verify visible GPU hardware",
                suggested_command: "nvidia-smi -L",
                evidence: "Only the allocated simulated GPU should be visible inside the job.",
            },
            LabStepMeta {
                check_id: "release",
                label: "Release the allocation",
                suggested_command: "exit",
                evidence: "The job should complete and the GPU tile should return to idle.",
            },
        ],
    },
    LabMeta {
        id: "05-containers",
        title: "Reproducible Container Workloads",
        scenario: "dgx-h200-8",
        estimated_minutes: 35,
        track: "Reliable jobs",
        summary: "Separate the host environment from a reproducible synthetic container workflow.",
        steps: &[
            LabStepMeta {
                check_id: "step-0",
                label: "Request an isolated GPU allocation",
                suggested_command: "salloc --partition=gpu --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:20:00",
                evidence: "The container exercise should begin inside a scheduler-owned allocation.",
            },
            LabStepMeta {
                check_id: "step-1",
                label: "Load the container module",
                suggested_command: "module load singularity/4.5.0",
                evidence: "The terminal should acknowledge the simulated module environment.",
            },
            LabStepMeta {
                check_id: "step-2",
                label: "Run the synthetic container workload",
                suggested_command: "singularity exec --nv /containers/pytorch-lab.sif python train.py --epochs 1",
                evidence: "Tie the container command to a scheduler-owned GPU allocation.",
            },
            LabStepMeta {
                check_id: "step-3",
                label: "Verify allocation-level GPU isolation",
                suggested_command: "nvidia-smi -L",
                evidence: "The active shell should expose only the GPU assigned by the scheduler.",
            },
            LabStepMeta {
                check_id: "step-4",
                label: "Submit and observe the missing-image failure",
                suggested_command: "sbatch train.sbatch",
                evidence: "The separately named `container-missing` job should reach `FAILED` after using `/missing/pytorch-lab.sif`.",
            },
            LabStepMeta {
                check_id: "step-5",
                label: "Inspect the missing-image job log",
                suggested_command: "tail -n 20 logs/container-missing-<jobid>.err",
                evidence: "Inspect stderr after the named failure before choosing a correction.",
            },
        ],
    },
    LabMeta {
        id: "06-batch-jobs",
        title: "From Interactive Command to Batch Job",
        scenario: "dgx-h200-8",
        estimated_minutes: 35,
        track: "Reliable jobs",
        summary: "Turn an interactive idea into a repeatable script with logs and accounting evidence.",
        steps: &[
            LabStepMeta {
                check_id: "edit-script",
                label: "Save and preflight the batch script",
                suggested_command: "Save train.sbatch in the Script Editor",
                evidence: "A learner write must save explicit resources, runtime, and output paths; the seeded file alone is not evidence.",
            },
            LabStepMeta {
                check_id: "submit-batch",
                label: "Submit the batch script",
                suggested_command: "sbatch train.sbatch",
                evidence: "Capture the returned job ID and locate it in the queue.",
            },
            LabStepMeta {
                check_id: "job-progress",
                label: "Advance until the batch job completes",
                suggested_command: "Use the +1 minute simulation control",
                evidence: "Completion must exist in simulator state before the lab can close.",
            },
            LabStepMeta {
                check_id: "inspect-logs",
                label: "Inspect the generated logs",
                suggested_command: "tail -n 20 logs/train-h200-<jobid>.out",
                evidence: "After completion, connect the virtual output file to the submitted job.",
            },
            LabStepMeta {
                check_id: "accounting",
                label: "Review the completed job in accounting",
                suggested_command: "sacct",
                evidence: "Use terminal state and elapsed time as durable completion evidence.",
            },
        ],
    },
    LabMeta {
        id: "07-pending-reasons",
        title: "Queue Contention and Pending Reasons",
        scenario: "dgx-contended",
        estimated_minutes: 35,
        track: "Reliable jobs",
        summary: "Read a pending job as an explanation, not an error, and watch contention resolve.",
        steps: &[
            LabStepMeta {
                check_id: "submit",
                label: "Submit a one-GPU job under contention",
                suggested_command: "sbatch train.sbatch",
                evidence: "Your job should enter the queue even when no GPU is immediately free.",
            },
            LabStepMeta {
                check_id: "observe-pending",
                label: "Observe the pending state",
                suggested_command: "squeue",
                evidence: "Identify your job and read its current scheduler reason.",
            },
            LabStepMeta {
                check_id: "inspect-reason",
                label: "Explain why the job is waiting",
                suggested_command: "scontrol show job <jobid>",
                evidence: "Compare the pending explanation with currently allocated GPU tiles.",
            },
            LabStepMeta {
                check_id: "start-after-wait",
                label: "Advance time until resources release",
                suggested_command: "Use the +1 minute simulation control",
                evidence: "The learner job should transition from pending to running without resubmission.",
            },
        ],
    },
    LabMeta {
        id: "08-arrays-dependencies",
        title: "Arrays and Dependencies",
        scenario: "dgx-contended",
        estimated_minutes: 40,
        track: "Reliable jobs",
        summary: "Coordinate repeated work and downstream tasks without manually babysitting the queue.",
        steps: &[
            LabStepMeta {
                check_id: "step-0",
                label: "Submit the edited four-task array",
                suggested_command: "sbatch train.sbatch",
                evidence: "Look for the expanded learner tasks created from one script.",
            },
            LabStepMeta {
                check_id: "step-1",
                label: "Inspect task states in the queue",
                suggested_command: "squeue",
                evidence: "Distinguish independently scheduled tasks from the parent submission.",
            },
            LabStepMeta {
                check_id: "step-2",
                label: "Submit the edited evaluation dependency",
                suggested_command: "sbatch train.sbatch",
                evidence: "After replacing the array with an `afterok` dependency, confirm the submitted job retains that prerequisite.",
            },
            LabStepMeta {
                check_id: "step-3",
                label: "Observe the evaluation dependency release",
                suggested_command: "Use the +1 minute simulation control",
                evidence: "Advance in bounded steps until the named `evaluate` job becomes eligible and starts or completes.",
            },
        ],
    },
    LabMeta {
        id: "09-failure-resume",
        title: "Diagnose Failure and Resume",
        scenario: "dgx-degraded",
        estimated_minutes: 45,
        track: "Recovery",
        summary: "Turn a failed run into evidence: classify it, find a checkpoint, and complete a safer retry.",
        steps: &[
            LabStepMeta {
                check_id: "sacct",
                label: "Find the failed job in accounting",
                suggested_command: "sacct",
                evidence: "Identify the terminal state before choosing a remedy.",
            },
            LabStepMeta {
                check_id: "inspect-logs",
                label: "Inspect logs for the failure signal",
                suggested_command: "tail -n 30 logs/train-llm-<jobid>.err",
                evidence: "After the OOM, read stderr to distinguish symptoms from causes.",
            },
            LabStepMeta {
                check_id: "oom-observed",
                label: "Explain the out-of-memory state",
                suggested_command: "scontrol show job <jobid>",
                evidence: "Connect the terminal state to a smaller batch size or larger memory request.",
            },
            LabStepMeta {
                check_id: "checkpoint",
                label: "Locate the newest usable checkpoint",
                suggested_command: "cat <checkpoint>",
                evidence: "Read the newest pre-retry checkpoint before using it as a restart point.",
            },
            LabStepMeta {
                check_id: "resume-submitted",
                label: "Run a corrected recovery job to completion",
                suggested_command: "srun --job-name=train-resume --partition=gpu --gres=gpu:h200:4 --cpus-per-task=16 --mem=64G --time=02:00:00 python train.py --batch-size 64 --epochs 5 --resume-from-checkpoint <checkpoint>",
                evidence: "The completed retry must preserve the partition, 4 GPUs, 16 CPUs, two-hour limit, exact 64 GiB request, and latest readable pre-retry checkpoint.",
            },
        ],
    },
    LabMeta {
        id: "10-multi-gpu",
        title: "Multi-GPU Training",
        scenario: "dgx-h200-8",
        estimated_minutes: 45,
        track: "Recovery",
        summary: "Scale a workload only after you can explain allocation, isolation, and launch behavior.",
        steps: &[
            LabStepMeta {
                check_id: "step-0",
                label: "Request a multi-GPU allocation",
                suggested_command: "salloc --partition=gpu --gres=gpu:h200:4 --cpus-per-task=32 --mem=256G --time=00:30:00",
                evidence: "Four GPU tiles should become owned by one learner job.",
            },
            LabStepMeta {
                check_id: "step-1",
                label: "Verify four visible GPUs",
                suggested_command: "nvidia-smi -L",
                evidence: "Confirm the allocation exposes four locally remapped GPU devices.",
            },
            LabStepMeta {
                check_id: "step-2",
                label: "Launch the synthetic distributed workload",
                suggested_command: "torchrun --nproc_per_node=4 train.py --epochs 3",
                evidence: "The simulator should accept the launch without creating host processes.",
            },
        ],
    },
    LabMeta {
        id: "11-policy-efficiency",
        title: "QOS, Reservations, and Efficiency",
        scenario: "dgx-shared",
        estimated_minutes: 40,
        track: "Recovery",
        summary: "Use queue and accounting evidence to make requests fairer and easier to schedule.",
        steps: &[
            LabStepMeta {
                check_id: "step-0",
                label: "Inspect shared-cluster queue pressure",
                suggested_command: "squeue",
                evidence: "Find the prepared learner job waiting on the per-user QOS limit.",
            },
            LabStepMeta {
                check_id: "step-1",
                label: "Inspect the QOS-limited job",
                suggested_command: "scontrol show job <jobid>",
                evidence: "Connect its typed pending reason to the configured concurrent-job limit.",
            },
            LabStepMeta {
                check_id: "step-2",
                label: "Review accounting before right-sizing",
                suggested_command: "sacct",
                evidence: "Use completed-job resource and elapsed-time evidence instead of guessing.",
            },
        ],
    },
    LabMeta {
        id: "12-capstone",
        title: "Capstone: Robust Training Campaign",
        scenario: "dgx-contended",
        estimated_minutes: 60,
        track: "Capstone",
        summary: "Plan, observe, recover, and review a complete training campaign under contention.",
        steps: &[
            LabStepMeta {
                check_id: "step-0",
                label: "Submit a robust training script",
                suggested_command: "sbatch train.sbatch",
                evidence: "Follow the script's named `train-h200` baseline through `COMPLETED` before injecting a separate failure.",
            },
            LabStepMeta {
                check_id: "step-1",
                label: "Monitor and explain its queue state",
                suggested_command: "squeue",
                evidence: "Use contention and state transitions to decide when intervention is unnecessary.",
            },
            LabStepMeta {
                check_id: "step-2",
                label: "Launch the controlled failure attempt",
                suggested_command: "srun --job-name=capstone-failure --partition=gpu --gres=gpu:h200:1 --cpus-per-task=8 --mem=16G --time=00:30:00 python train.py --batch-size 64 --epochs 3",
                evidence: "The intentionally constrained request should create a separate learner job for diagnosis.",
            },
            LabStepMeta {
                check_id: "step-3",
                label: "Observe the out-of-memory failure",
                suggested_command: "Use the +1 minute simulation control",
                evidence: "Wait for the controlled attempt to reach an out-of-memory terminal state before correcting it.",
            },
            LabStepMeta {
                check_id: "step-4",
                label: "Complete the corrected recovery",
                suggested_command: "srun --job-name=capstone-recovery --partition=gpu --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:30:00 python train.py --batch-size 64 --epochs 3 --resume-from-checkpoint <checkpoint>",
                evidence: "Complete a 1-GPU, 8-CPU, 64-GiB, 30-minute recovery after the controlled failure using the latest readable checkpoint.",
            },
            LabStepMeta {
                check_id: "step-5",
                label: "Produce final accounting evidence",
                suggested_command: "sacct",
                evidence: "Summarize original, failure, and recovery terminal states with elapsed time.",
            },
        ],
    },
];

#[must_use]
pub fn lab_meta(lab_id: &str) -> Option<&'static LabMeta> {
    COURSE_LABS.iter().find(|lab| lab.id == lab_id)
}

#[must_use]
pub fn lab_step_meta(lab_id: &str, check_id: &str) -> Option<&'static LabStepMeta> {
    lab_meta(lab_id).and_then(|lab| lab.steps.iter().find(|step| step.check_id == check_id))
}

#[must_use]
pub fn lab_for_scenario(scenario_id: &str) -> Option<&'static str> {
    match scenario_id {
        "guided-one-gpu" => Some("04-one-gpu"),
        "dgx-contended" | "pending-gpu-contention-01" => Some("07-pending-reasons"),
        "dgx-degraded" | "failure-resume-01" => Some("09-failure-resume"),
        "dgx-shared" => Some("11-policy-efficiency"),
        "dgx-h200-8" => Some("06-batch-jobs"),
        _ => None,
    }
}

/// Generic command-prefix checks used for labs without dedicated assertion packs.
#[must_use]
pub fn generic_lab_checks(lab_id: &str) -> Vec<PracticalCheck> {
    match lab_id {
        "02-interactive-cpu" => vec![
            practical_check(
                "step-0",
                20,
                false,
                exact_job(None, Some(0), Some(4), Some(8 * 1024), &[JobStatus::Running]),
            ),
            practical_check(
                "step-1",
                20,
                false,
                command_in_exact_allocation(
                    &["env"],
                    Some(0),
                    Some(4),
                    Some(8 * 1024),
                    &[JobStatus::Running],
                ),
            ),
            practical_check(
                "step-2",
                20,
                false,
                Assertion::All {
                    assertions: vec![
                        exact_job(
                            None,
                            Some(0),
                            Some(4),
                            Some(8 * 1024),
                            &[JobStatus::Running, JobStatus::Completed],
                        ),
                        Assertion::ActiveAllocationReleased,
                    ],
                },
            ),
            practical_check(
                "step-3",
                40,
                true,
                Assertion::All {
                    assertions: vec![
                        command_in_exact_allocation(
                            &["env"],
                            Some(0),
                            Some(4),
                            Some(8 * 1024),
                            &[JobStatus::Running, JobStatus::Completed],
                        ),
                        command_after_exact_job(
                            &["sacct"],
                            None,
                            Some(0),
                            Some(4),
                            Some(8 * 1024),
                            JobStatus::Completed,
                        ),
                    ],
                },
            ),
        ],
        "03-cpu-memory" => vec![
            practical_check(
                "step-0",
                20,
                false,
                exact_job(
                    Some("prep-ok"),
                    Some(0),
                    Some(8),
                    Some(64 * 1024),
                    &[JobStatus::Completed],
                ),
            ),
            command_check("step-1", 20, false, &["scontrol show job"]),
            practical_check(
                "step-2",
                20,
                false,
                exact_job(
                    Some("prep-oom"),
                    Some(0),
                    Some(8),
                    Some(16 * 1024),
                    &[JobStatus::Submitted],
                ),
            ),
            practical_check(
                "step-3",
                40,
                true,
                Assertion::All {
                    assertions: vec![
                        exact_job(
                            Some("prep-ok"),
                            Some(0),
                            Some(8),
                            Some(64 * 1024),
                            &[JobStatus::Completed],
                        ),
                        exact_job(
                            Some("prep-oom"),
                            Some(0),
                            Some(8),
                            Some(16 * 1024),
                            &[JobStatus::OutOfMemory],
                        ),
                    ],
                },
            ),
        ],
        "05-containers" => vec![
            practical_check(
                "step-0",
                15,
                false,
                exact_job(None, Some(1), Some(8), Some(64 * 1024), &[JobStatus::Running]),
            ),
            practical_check(
                "step-1",
                15,
                false,
                Assertion::All {
                    assertions: vec![
                        Assertion::ExactCommandUsed {
                            command: "module load singularity/4.5.0".into(),
                        },
                        command_in_exact_allocation(
                            &["module load singularity/4.5.0"],
                            Some(1),
                            Some(8),
                            Some(64 * 1024),
                            &[JobStatus::Running],
                        ),
                    ],
                },
            ),
            practical_check(
                "step-2",
                20,
                false,
                command_in_exact_allocation(
                    &["singularity exec --nv /containers/pytorch-lab.sif"],
                    Some(1),
                    Some(8),
                    Some(64 * 1024),
                    &[JobStatus::Running],
                ),
            ),
            practical_check(
                "step-3",
                15,
                false,
                command_in_exact_allocation(
                    &["nvidia-smi"],
                    Some(1),
                    Some(8),
                    Some(64 * 1024),
                    &[JobStatus::Running],
                ),
            ),
            practical_check(
                "step-4",
                20,
                true,
                exact_job(Some("container-missing"), None, None, None, &[JobStatus::Failed]),
            ),
            practical_check(
                "step-5",
                15,
                true,
                command_after_exact_job(
                    &["tail -n"],
                    Some("container-missing"),
                    None,
                    None,
                    None,
                    JobStatus::Failed,
                ),
            ),
        ],
        "08-arrays-dependencies" => vec![
            practical_check("step-0", 30, false, Assertion::LearnerArrayTaskCount { minimum: 4 }),
            command_check("step-1", 20, false, &["squeue"]),
            practical_check("step-2", 25, false, Assertion::LearnerJobHasDependency),
            practical_check("step-3", 25, true, Assertion::LearnerSweepEvaluation),
        ],
        "10-multi-gpu" => vec![
            practical_check(
                "step-0",
                30,
                false,
                exact_job(None, Some(4), Some(32), Some(256 * 1024), &[JobStatus::Running]),
            ),
            practical_check(
                "step-1",
                30,
                false,
                command_in_exact_allocation(
                    &["nvidia-smi"],
                    Some(4),
                    Some(32),
                    Some(256 * 1024),
                    &[JobStatus::Running],
                ),
            ),
            practical_check(
                "step-2",
                40,
                true,
                command_in_exact_allocation(
                    &["torchrun --nproc_per_node=4", "torchrun --nproc-per-node=4"],
                    Some(4),
                    Some(32),
                    Some(256 * 1024),
                    &[JobStatus::Running],
                ),
            ),
        ],
        "11-policy-efficiency" => vec![
            command_check("step-0", 30, false, &["squeue"]),
            practical_check(
                "step-1",
                40,
                true,
                Assertion::JobInspectionWhilePending {
                    pending_reason: slurm_model::PendingReason::QosMaxJobsPerUserLimit,
                    gpus: Some(1),
                    cpus: Some(8),
                    min_memory_mib: Some(64 * 1024),
                    max_memory_mib: Some(64 * 1024),
                    states: vec![JobStatus::Pending],
                },
            ),
            command_check("step-2", 30, true, &["sacct"]),
        ],
        "12-capstone" => vec![
            practical_check(
                "step-0",
                15,
                true,
                exact_job(
                    Some("train-h200"),
                    Some(1),
                    Some(8),
                    Some(64 * 1024),
                    &[JobStatus::Completed],
                ),
            ),
            command_check("step-1", 15, false, &["squeue", "scontrol"]),
            practical_check(
                "step-2",
                15,
                false,
                exact_job(
                    Some("capstone-failure"),
                    Some(1),
                    Some(8),
                    Some(16 * 1024),
                    &[JobStatus::Submitted],
                ),
            ),
            practical_check(
                "step-3",
                20,
                true,
                exact_job(
                    Some("capstone-failure"),
                    Some(1),
                    Some(8),
                    Some(16 * 1024),
                    &[JobStatus::OutOfMemory],
                ),
            ),
            practical_check(
                "step-4",
                20,
                true,
                Assertion::LearnerRecoveryJobCompleted {
                    name: "capstone-recovery".into(),
                    partition: Some("gpu".into()),
                    gpus: 1,
                    cpus: 8,
                    minimum_memory_mib: 64 * 1024,
                    maximum_memory_mib: Some(64 * 1024),
                    minimum_time_limit_ms: 30 * 60 * 1_000,
                    maximum_time_limit_ms: Some(30 * 60 * 1_000),
                    after_job_name: "capstone-failure".into(),
                },
            ),
            practical_check(
                "step-5",
                15,
                true,
                command_after_exact_job(
                    &["sacct"],
                    Some("capstone-recovery"),
                    Some(1),
                    Some(8),
                    Some(64 * 1024),
                    JobStatus::Completed,
                ),
            ),
        ],
        "01-cluster-mental-model" => vec![
            command_check("step-0", 50, false, &["sinfo"]),
            command_check("step-1", 50, true, &["scontrol show node"]),
        ],
        _ => vec![command_check("step-0", 100, true, &["sinfo"])],
    }
}

fn practical_check(id: &str, points: u32, critical: bool, assertion: Assertion) -> PracticalCheck {
    PracticalCheck { id: id.into(), points, critical, assertion }
}

fn exact_job(
    name: Option<&str>,
    gpus: Option<u16>,
    cpus: Option<u32>,
    memory_mib: Option<u64>,
    states: &[JobStatus],
) -> Assertion {
    Assertion::LearnerJobMatches {
        name: name.map(str::to_owned),
        gpus,
        cpus,
        min_memory_mib: memory_mib,
        max_memory_mib: memory_mib,
        states: states.to_vec(),
    }
}

fn command_in_exact_allocation(
    prefixes: &[&str],
    gpus: Option<u16>,
    cpus: Option<u32>,
    memory_mib: Option<u64>,
    states: &[JobStatus],
) -> Assertion {
    Assertion::CommandInMatchingAllocation {
        prefixes: prefixes.iter().map(|prefix| (*prefix).into()).collect(),
        gpus,
        cpus,
        min_memory_mib: memory_mib,
        max_memory_mib: memory_mib,
        states: states.to_vec(),
    }
}

fn command_after_exact_job(
    prefixes: &[&str],
    name: Option<&str>,
    gpus: Option<u16>,
    cpus: Option<u32>,
    memory_mib: Option<u64>,
    state: JobStatus,
) -> Assertion {
    Assertion::CommandAfterMatchingJobState {
        prefixes: prefixes.iter().map(|prefix| (*prefix).into()).collect(),
        name: name.map(str::to_owned),
        gpus,
        cpus,
        min_memory_mib: memory_mib,
        max_memory_mib: memory_mib,
        state,
    }
}

fn command_check(id: &str, points: u32, critical: bool, prefixes: &[&str]) -> PracticalCheck {
    PracticalCheck {
        id: id.into(),
        points,
        critical,
        assertion: Assertion::AnyCommandUsed {
            prefixes: prefixes.iter().map(|prefix| (*prefix).into()).collect(),
        },
    }
}

#[must_use]
pub fn generic_hints(lab_id: &str) -> Vec<&'static str> {
    match lab_id {
        "01-cluster-mental-model" => vec![
            "Run `sinfo` to find the default partition and current node state.",
            "Use `scontrol show node dgx-h200-01` to inspect CPU, memory, and GPU capacity.",
            "Compare total and allocated fields before you make any resource request.",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_labs_require_state_backed_exact_evidence() {
        let lab02 = generic_lab_checks("02-interactive-cpu");
        assert!(matches!(
            &lab02[0].assertion,
            Assertion::LearnerJobMatches {
                gpus: Some(0),
                cpus: Some(4),
                min_memory_mib: Some(8_192),
                max_memory_mib: Some(8_192),
                ..
            }
        ));
        assert!(matches!(&lab02[1].assertion, Assertion::CommandInMatchingAllocation { .. }));
        assert!(matches!(&lab02[3].assertion, Assertion::All { .. }));

        let lab03 = generic_lab_checks("03-cpu-memory");
        assert!(matches!(
            &lab03[0].assertion,
            Assertion::LearnerJobMatches {
                name: Some(name),
                gpus: Some(0),
                cpus: Some(8),
                min_memory_mib: Some(65_536),
                max_memory_mib: Some(65_536),
                states,
            } if name == "prep-ok" && states.contains(&JobStatus::Completed)
        ));
        assert!(matches!(&lab03[3].assertion, Assertion::All { .. }));

        let lab05 = generic_lab_checks("05-containers");
        assert_eq!(lab05.len(), 6);
        assert!(matches!(&lab05[2].assertion, Assertion::CommandInMatchingAllocation { .. }));
        assert!(matches!(
            &lab05[5].assertion,
            Assertion::CommandAfterMatchingJobState {
                name: Some(name),
                state: JobStatus::Failed,
                ..
            } if name == "container-missing"
        ));

        let lab08 = generic_lab_checks("08-arrays-dependencies");
        assert!(matches!(&lab08[3].assertion, Assertion::LearnerSweepEvaluation));

        let lab10 = generic_lab_checks("10-multi-gpu");
        assert!(matches!(
            &lab10[2].assertion,
            Assertion::CommandInMatchingAllocation {
                gpus: Some(4),
                cpus: Some(32),
                min_memory_mib: Some(262_144),
                max_memory_mib: Some(262_144),
                ..
            }
        ));

        let lab11 = generic_lab_checks("11-policy-efficiency");
        assert!(matches!(
            &lab11[1].assertion,
            Assertion::JobInspectionWhilePending {
                pending_reason: slurm_model::PendingReason::QosMaxJobsPerUserLimit,
                gpus: Some(1),
                states,
                ..
            }
            if states == &vec![JobStatus::Pending]
        ));

        let lab12 = generic_lab_checks("12-capstone");
        assert!(matches!(
            &lab12[0].assertion,
            Assertion::LearnerJobMatches {
                name: Some(name),
                states,
                ..
            } if name == "train-h200" && states.contains(&JobStatus::Completed)
        ));
        assert!(lab12[3].critical);
        assert!(matches!(
            &lab12[4].assertion,
            Assertion::LearnerRecoveryJobCompleted {
                name,
                gpus: 1,
                cpus: 8,
                minimum_memory_mib: 65_536,
                maximum_memory_mib: Some(65_536),
                minimum_time_limit_ms: 1_800_000,
                after_job_name,
                ..
            } if name == "capstone-recovery" && after_job_name == "capstone-failure"
        ));
        assert!(matches!(
            &lab12[5].assertion,
            Assertion::CommandAfterMatchingJobState {
                name: Some(name),
                state: JobStatus::Completed,
                ..
            } if name == "capstone-recovery"
        ));
    }
}
