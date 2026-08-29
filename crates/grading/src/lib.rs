#![forbid(unsafe_code)]

//! State-based practical grading and evidence ledger.

use dgxlab_contracts::{JobId, SimTimeMs};
use serde::{Deserialize, Serialize};
use sim_core::{SimulationWorld, WorldEventKind};
use slurm_model::JobStatus;
use virtual_shell::ShellSession;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLedger {
    pub events: Vec<EvidenceEvent>,
    pub diagnoses: Vec<DiagnosisRecord>,
}

impl EvidenceLedger {
    #[must_use]
    pub fn new() -> Self {
        Self { events: Vec::new(), diagnoses: Vec::new() }
    }

    pub fn record_command(&mut self, at: SimTimeMs, command: impl Into<String>) {
        self.events.push(EvidenceEvent::Command { at, command: command.into() });
    }

    pub fn record_diagnosis(&mut self, at: SimTimeMs, diagnosis: Diagnosis) {
        self.diagnoses.push(DiagnosisRecord { at, diagnosis });
    }
}

impl Default for EvidenceLedger {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EvidenceEvent {
    Command { at: SimTimeMs, command: String },
    HintUsed { at: SimTimeMs, hint_id: String, level: u8 },
    AnswerSubmitted { at: SimTimeMs, question_id: String },
    FileOpened { at: SimTimeMs, path: String },
    FileWritten { at: SimTimeMs, path: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Diagnosis {
    Resources,
    Priority,
    Dependency,
    GpuOutOfMemory,
    HostOutOfMemory,
    TimeLimit,
    NodeFailure,
    ScriptFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosisRecord {
    pub at: SimTimeMs,
    pub diagnosis: Diagnosis,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PracticalCheck {
    pub id: String,
    pub points: u32,
    pub critical: bool,
    pub assertion: Assertion,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Assertion {
    LearnerJobExists { gpus: Option<u16>, cpus: Option<u32>, max_memory_mib: Option<u64> },
    AnyCommandUsed { prefixes: Vec<String> },
    LearnerJobVisitedState { state: JobStatus },
    LearnerDiagnosis { diagnosis: Diagnosis },
    VirtualFileExists { path: String },
    ActiveAllocationReleased,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub passed: bool,
    pub earned_points: u32,
    pub possible_points: u32,
    pub critical: bool,
    pub evidence: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PracticalScore {
    pub results: Vec<CheckResult>,
    pub earned_points: u32,
    pub possible_points: u32,
    pub all_critical_passed: bool,
}

pub fn evaluate_practical(
    checks: &[PracticalCheck],
    world: &SimulationWorld,
    shell: &ShellSession,
    ledger: &EvidenceLedger,
    learner: &str,
) -> PracticalScore {
    let results: Vec<CheckResult> = checks
        .iter()
        .map(|check| {
            let (passed, evidence) = evaluate_assertion(&check.assertion, world, shell, ledger, learner);
            CheckResult {
                id: check.id.clone(),
                passed,
                earned_points: if passed { check.points } else { 0 },
                possible_points: check.points,
                critical: check.critical,
                evidence,
            }
        })
        .collect();
    PracticalScore {
        earned_points: results.iter().map(|result| result.earned_points).sum(),
        possible_points: results.iter().map(|result| result.possible_points).sum(),
        all_critical_passed: results.iter().filter(|result| result.critical).all(|result| result.passed),
        results,
    }
}

fn evaluate_assertion(
    assertion: &Assertion,
    world: &SimulationWorld,
    shell: &ShellSession,
    ledger: &EvidenceLedger,
    learner: &str,
) -> (bool, String) {
    match assertion {
        Assertion::LearnerJobExists { gpus, cpus, max_memory_mib } => {
            let matched = world.jobs.values().find(|job| {
                job.spec.user == learner
                    && gpus.is_none_or(|expected| job.spec.resources.gpus == expected)
                    && cpus.is_none_or(|expected| job.spec.resources.cpus == expected)
                    && max_memory_mib.is_none_or(|maximum| job.spec.resources.memory_mib <= maximum)
            });
            matched
                .map(|job| (true, format!("matched learner job {}", job.id.0)))
                .unwrap_or_else(|| (false, "no learner job matched the resource predicate".into()))
        }
        Assertion::AnyCommandUsed { prefixes } => {
            let matched = ledger.events.iter().find_map(|event| match event {
                EvidenceEvent::Command { command, .. }
                    if prefixes.iter().any(|prefix| command.trim_start().starts_with(prefix)) =>
                {
                    Some(command)
                }
                _ => None,
            });
            matched
                .map(|command| (true, format!("observed command: {command}")))
                .unwrap_or_else(|| (false, format!("none of the command prefixes were observed: {prefixes:?}")))
        }
        Assertion::LearnerJobVisitedState { state } => {
            let jobs: Vec<JobId> = world
                .jobs
                .values()
                .filter(|job| job.spec.user == learner)
                .map(|job| job.id)
                .collect();
            let matched = world.event_log.iter().any(|event| match &event.kind {
                WorldEventKind::JobStarted { job_id, .. } if state == &JobStatus::Running => jobs.contains(job_id),
                WorldEventKind::JobFinished { job_id, status } if status == state => jobs.contains(job_id),
                WorldEventKind::JobPending { job_id, .. } if state == &JobStatus::Pending => jobs.contains(job_id),
                _ => false,
            });
            (matched, format!("learner jobs examined: {jobs:?}"))
        }
        Assertion::LearnerDiagnosis { diagnosis } => {
            let matched = ledger.diagnoses.iter().any(|record| &record.diagnosis == diagnosis);
            (matched, format!("recorded diagnoses: {:?}", ledger.diagnoses))
        }
        Assertion::VirtualFileExists { path } => {
            let exists = world.fs.exists(path);
            (exists, format!("virtual file {path} exists={exists}"))
        }
        Assertion::ActiveAllocationReleased => {
            let released = shell.active_job_id.is_none();
            (released, format!("active_job_id={:?}", shell.active_job_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::SimulationWorld;
    use slurm_model::{JobSpec, Tres};

    #[test]
    fn grades_state_not_exact_whitespace() {
        let mut world = SimulationWorld::dgx_h200_8(1);
        let spec = JobSpec {
            user: "learner".into(),
            resources: Tres {
                cpus: 8,
                memory_mib: 64 * 1024,
                gpu_type: Some("h200".into()),
                gpus: 1,
            },
            ..JobSpec::default()
        };
        world.submit_job(spec).unwrap();
        let shell = ShellSession::learner();
        let mut ledger = EvidenceLedger::new();
        ledger.record_command(SimTimeMs::ZERO, "  squeue   -u learner");
        let checks = vec![
            PracticalCheck {
                id: "job".into(),
                points: 60,
                critical: true,
                assertion: Assertion::LearnerJobExists {
                    gpus: Some(1),
                    cpus: Some(8),
                    max_memory_mib: Some(64 * 1024),
                },
            },
            PracticalCheck {
                id: "inspect".into(),
                points: 40,
                critical: false,
                assertion: Assertion::AnyCommandUsed { prefixes: vec!["squeue".into()] },
            },
        ];
        let score = evaluate_practical(&checks, &world, &shell, &ledger, "learner");
        assert_eq!(score.earned_points, 100);
        assert!(score.all_critical_passed);
    }
}
