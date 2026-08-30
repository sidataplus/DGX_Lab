//! Pure learner-facing derivations kept separate from Leptos rendering.

use dgxlab_contracts::{UiJobSummary, UiLabStep};
use sim_session::{BUILTIN_LABS, LabMeta, learner_step_meta};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProgressSummary {
    pub completed: usize,
    pub total: usize,
    pub percent: u8,
    pub next_step: Option<UiLabStep>,
}

pub(crate) fn summarize_progress(steps: &[UiLabStep]) -> ProgressSummary {
    let total = steps.len();
    let completed = steps.iter().filter(|step| step.complete).count();
    let percent = (completed * 100).checked_div(total).unwrap_or(0) as u8;

    ProgressSummary {
        completed,
        total,
        percent,
        next_step: steps.iter().find(|step| !step.complete).cloned(),
    }
}

pub(crate) fn active_lab(lab_id: &str) -> Option<(usize, &'static LabMeta)> {
    BUILTIN_LABS.iter().enumerate().find(|(_, lab)| lab.id == lab_id)
}

pub(crate) fn suggested_command(
    lab_id: &str,
    step_id: &str,
    jobs: &[UiJobSummary],
    checkpoint_paths: &[String],
) -> Option<String> {
    let mut command = learner_step_meta(lab_id, step_id)?.suggested_command.to_string();
    if command.contains("<jobid>") {
        let job_id = inspection_job_id(lab_id, jobs)?;
        command = command.replace("<jobid>", &job_id.to_string());
    }
    if command.contains("<checkpoint>") {
        let checkpoint = checkpoint_paths.last()?;
        command = command.replace("<checkpoint>", checkpoint);
    }
    (!command.contains('<')).then_some(command)
}

pub(crate) fn preferred_job_id(jobs: &[UiJobSummary]) -> Option<u64> {
    let learner_jobs = || jobs.iter().filter(|job| job.user == "learner");
    learner_jobs()
        .find(|job| job.status == "PENDING")
        .or_else(|| {
            learner_jobs().find(|job| {
                matches!(
                    job.status.as_str(),
                    "OUTOFMEMORY" | "FAILED" | "TIMEOUT" | "NODEFAIL" | "CANCELLED"
                )
            })
        })
        .or_else(|| learner_jobs().find(|job| job.status == "RUNNING"))
        .or_else(|| learner_jobs().max_by_key(|job| job.id))
        .map(|job| job.id)
}

/// Certification-grade practical evidence comes from the comprehensive capstone,
/// not whichever focused lab happens to be open when the learner starts Assess.
pub(crate) fn readiness_practical(
    lab_id: &str,
    practical_percent: u8,
    critical_practical_passed: bool,
) -> (u8, bool) {
    if lab_id == "12-capstone" {
        (practical_percent, critical_practical_passed)
    } else {
        (0, false)
    }
}

pub(crate) fn active_job_count(jobs: &[UiJobSummary]) -> usize {
    jobs.iter().filter(|job| matches!(job.status.as_str(), "PENDING" | "RUNNING")).count()
}

fn inspection_job_id(lab_id: &str, jobs: &[UiJobSummary]) -> Option<u64> {
    let targeted = match lab_id {
        "09-failure-resume" => {
            jobs.iter().find(|job| job.user == "learner" && job.status == "OUTOFMEMORY")
        }
        "11-policy-efficiency" => jobs.iter().find(|job| {
            job.user == "learner"
                && job.status == "PENDING"
                && job.pending_reason.as_deref() == Some("QOSMaxJobsPerUserLimit")
        }),
        _ => None,
    };
    targeted.map(|job| job.id).or_else(|| preferred_job_id(jobs))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(id: &str, complete: bool) -> UiLabStep {
        UiLabStep { id: id.into(), label: id.into(), complete, critical: false }
    }

    #[test]
    fn progress_summary_points_to_first_incomplete_action() {
        let steps = vec![step("inspect", true), step("allocate", false), step("verify-env", false)];

        let summary = summarize_progress(&steps);

        assert_eq!(summary.completed, 1);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.percent, 33);
        assert_eq!(summary.next_step.map(|item| item.id), Some("allocate".into()));
    }

    #[test]
    fn active_lab_uses_course_identity_even_when_scenarios_are_shared() {
        let (index, lab) = active_lab("03-cpu-memory").expect("lab 03");

        assert_eq!(index, 2);
        assert_eq!(lab.id, "03-cpu-memory");
        assert_eq!(lab.scenario, "dgx-h200-8");
    }

    #[test]
    fn suggested_command_is_specific_to_the_current_step() {
        assert_eq!(
            suggested_command("04-one-gpu", "verify-env", &[], &[]),
            Some("echo $CUDA_VISIBLE_DEVICES".into())
        );
        assert_eq!(suggested_command("04-one-gpu", "unknown", &[], &[]), None);
    }

    #[test]
    fn suggested_job_command_resolves_the_relevant_pending_job() {
        let jobs = vec![
            job(10000, "COMPLETED", None),
            job(10001, "RUNNING", None),
            job(10002, "PENDING", Some("QOSMaxJobsPerUserLimit")),
        ];

        assert_eq!(
            suggested_command("11-policy-efficiency", "step-1", &jobs, &[]),
            Some("scontrol show job 10002".into())
        );
        assert_eq!(preferred_job_id(&jobs), Some(10002));
    }

    #[test]
    fn unresolved_job_template_is_not_offered_as_a_runnable_command() {
        assert_eq!(suggested_command("03-cpu-memory", "step-1", &[], &[]), None);
    }

    #[test]
    fn recovery_command_uses_the_latest_available_checkpoint() {
        let checkpoints =
            vec!["checkpoints/epoch-001.pt".into(), "checkpoints/epoch-004.pt".into()];

        assert_eq!(
            suggested_command("09-failure-resume", "resume-submitted", &[], &checkpoints),
            Some("srun --job-name=train-resume --partition=gpu --gres=gpu:h200:4 --cpus-per-task=16 --mem=64G --time=02:00:00 python train.py --batch-size 64 --epochs 5 --resume-from-checkpoint checkpoints/epoch-004.pt".into())
        );
        assert_eq!(suggested_command("09-failure-resume", "resume-submitted", &[], &[]), None);
    }

    #[test]
    fn readiness_practical_evidence_is_anchored_to_the_capstone() {
        assert_eq!(readiness_practical("04-one-gpu", 100, true), (0, false));
        assert_eq!(readiness_practical("12-capstone", 75, false), (75, false));
        assert_eq!(readiness_practical("12-capstone", 100, true), (100, true));
    }

    #[test]
    fn active_job_count_excludes_terminal_history() {
        let jobs = vec![
            job(10000, "COMPLETED", None),
            job(10001, "RUNNING", None),
            job(10002, "PENDING", Some("Resources")),
            job(10003, "FAILED", None),
        ];

        assert_eq!(active_job_count(&jobs), 2);
    }

    fn job(id: u64, status: &str, pending_reason: Option<&str>) -> UiJobSummary {
        UiJobSummary {
            id,
            name: format!("job-{id}"),
            user: "learner".into(),
            status: status.into(),
            pending_reason: pending_reason.map(str::to_string),
            pending_explanation: None,
            gpus: 1,
            cpus: 8,
            memory_mib: 64 * 1024,
        }
    }
}
