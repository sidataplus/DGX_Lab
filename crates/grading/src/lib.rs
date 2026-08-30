#![forbid(unsafe_code)]

//! State-based practical grading and evidence ledger.

use dgxlab_contracts::{JobId, SimTimeMs};
use serde::{Deserialize, Serialize};
use sim_core::{SimulationWorld, WorldEventKind};
use slurm_model::{JobRecord, JobStatus, PendingReason};
use std::collections::BTreeSet;
use virtual_fs::normalize_path;
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
        self.record_command_with_context(at, command, None, Vec::new());
    }

    pub fn record_command_with_context(
        &mut self,
        at: SimTimeMs,
        command: impl Into<String>,
        active_job_id: Option<JobId>,
        created_job_ids: Vec<JobId>,
    ) {
        self.events.push(EvidenceEvent::Command {
            at,
            command: command.into(),
            active_job_id,
            created_job_ids,
        });
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
    Command {
        at: SimTimeMs,
        command: String,
        #[serde(default)]
        active_job_id: Option<JobId>,
        #[serde(default)]
        created_job_ids: Vec<JobId>,
    },
    HintUsed {
        at: SimTimeMs,
        hint_id: String,
        level: u8,
    },
    AnswerSubmitted {
        at: SimTimeMs,
        question_id: String,
    },
    FileOpened {
        at: SimTimeMs,
        path: String,
    },
    FileWritten {
        at: SimTimeMs,
        path: String,
    },
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
    All {
        assertions: Vec<Assertion>,
    },
    LearnerJobExists {
        gpus: Option<u16>,
        cpus: Option<u32>,
        max_memory_mib: Option<u64>,
    },
    LearnerJobMatches {
        name: Option<String>,
        gpus: Option<u16>,
        cpus: Option<u32>,
        min_memory_mib: Option<u64>,
        max_memory_mib: Option<u64>,
        states: Vec<JobStatus>,
    },
    LearnerArrayTaskCount {
        minimum: usize,
    },
    LearnerJobHasDependency,
    AnyCommandUsed {
        prefixes: Vec<String>,
    },
    ExactCommandUsed {
        command: String,
    },
    CommandInMatchingAllocation {
        prefixes: Vec<String>,
        gpus: Option<u16>,
        cpus: Option<u32>,
        min_memory_mib: Option<u64>,
        max_memory_mib: Option<u64>,
        states: Vec<JobStatus>,
    },
    CommandAfterMatchingJobState {
        prefixes: Vec<String>,
        name: Option<String>,
        gpus: Option<u16>,
        cpus: Option<u32>,
        min_memory_mib: Option<u64>,
        max_memory_mib: Option<u64>,
        state: JobStatus,
    },
    JobInspectionWhilePending {
        pending_reason: PendingReason,
        gpus: Option<u16>,
        cpus: Option<u32>,
        min_memory_mib: Option<u64>,
        max_memory_mib: Option<u64>,
        states: Vec<JobStatus>,
    },
    LearnerSweepEvaluation,
    LearnerJobVisitedState {
        state: JobStatus,
    },
    LearnerJobNamedVisitedState {
        name: String,
        states: Vec<JobStatus>,
    },
    LearnerRecoveryJobCompleted {
        name: String,
        partition: Option<String>,
        gpus: u16,
        cpus: u32,
        minimum_memory_mib: u64,
        maximum_memory_mib: Option<u64>,
        minimum_time_limit_ms: u64,
        maximum_time_limit_ms: Option<u64>,
        after_job_name: String,
    },
    LearnerDiagnosis {
        diagnosis: Diagnosis,
    },
    VirtualFileExists {
        path: String,
    },
    VirtualFileWritten {
        path: String,
    },
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
            let (passed, evidence) =
                evaluate_assertion(&check.assertion, world, shell, ledger, learner);
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
        all_critical_passed: results
            .iter()
            .filter(|result| result.critical)
            .all(|result| result.passed),
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
        Assertion::All { assertions } => {
            let results = assertions
                .iter()
                .map(|nested| evaluate_assertion(nested, world, shell, ledger, learner))
                .collect::<Vec<_>>();
            let passed = results.iter().all(|(passed, _)| *passed);
            let evidence = results
                .into_iter()
                .map(|(passed, evidence)| {
                    format!("{}: {evidence}", if passed { "pass" } else { "fail" })
                })
                .collect::<Vec<_>>()
                .join("; ");
            (passed, evidence)
        }
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
        Assertion::LearnerJobMatches {
            name,
            gpus,
            cpus,
            min_memory_mib,
            max_memory_mib,
            states,
        } => {
            let candidates = world
                .jobs
                .values()
                .filter(|job| {
                    job_matches(
                        job,
                        learner,
                        name.as_deref(),
                        *gpus,
                        *cpus,
                        *min_memory_mib,
                        *max_memory_mib,
                    )
                })
                .collect::<Vec<_>>();
            let matched = candidates
                .iter()
                .find(|job| states.iter().all(|state| job_visited_state(world, job.id, *state)));
            matched
                .map(|job| {
                    (
                        true,
                        format!("learner job {} matched and visited every state {states:?}", job.id.0),
                    )
                })
                .unwrap_or_else(|| {
                    (
                        false,
                        format!(
                            "no single matching learner job visited every state {states:?}; candidates: {:?}",
                            candidates.iter().map(|job| job.id).collect::<Vec<_>>()
                        ),
                    )
                })
        }
        Assertion::LearnerArrayTaskCount { minimum } => {
            let count = world
                .jobs
                .values()
                .filter(|job| job.spec.user == learner && job.spec.array_index.is_some())
                .count();
            (count >= *minimum, format!("learner array tasks observed: {count}"))
        }
        Assertion::LearnerJobHasDependency => {
            let matched = world
                .jobs
                .values()
                .find(|job| job.spec.user == learner && job.spec.dependency_after_ok.is_some());
            matched
                .map(|job| (true, format!("learner job {} has an afterok dependency", job.id.0)))
                .unwrap_or_else(|| (false, "no learner job with a dependency was observed".into()))
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
            matched.map(|command| (true, format!("observed command: {command}"))).unwrap_or_else(
                || (false, format!("none of the command prefixes were observed: {prefixes:?}")),
            )
        }
        Assertion::ExactCommandUsed { command } => {
            let matched = ledger.events.iter().any(|event| match event {
                EvidenceEvent::Command { command: observed, .. } => {
                    observed.trim() == command.trim()
                }
                _ => false,
            });
            (matched, format!("exact successful command observed={matched}: {command}"))
        }
        Assertion::CommandInMatchingAllocation {
            prefixes,
            gpus,
            cpus,
            min_memory_mib,
            max_memory_mib,
            states,
        } => {
            let matched = ledger.events.iter().find_map(|event| match event {
                EvidenceEvent::Command { command, active_job_id: Some(job_id), .. }
                    if command_has_prefix(command, prefixes) =>
                {
                    world
                        .jobs
                        .get(job_id)
                        .filter(|job| {
                            job_matches(
                                job,
                                learner,
                                None,
                                *gpus,
                                *cpus,
                                *min_memory_mib,
                                *max_memory_mib,
                            ) && states.iter().all(|state| job_visited_state(world, job.id, *state))
                        })
                        .map(|job| (command, job))
                }
                _ => None,
            });
            matched
                .map(|(command, job)| {
                    (
                        true,
                        format!("observed {command:?} inside matching learner allocation {}", job.id.0),
                    )
                })
                .unwrap_or_else(|| {
                    (
                        false,
                        format!("no command with prefixes {prefixes:?} ran inside a matching learner allocation that visited every state {states:?}"),
                    )
                })
        }
        Assertion::CommandAfterMatchingJobState {
            prefixes,
            name,
            gpus,
            cpus,
            min_memory_mib,
            max_memory_mib,
            state,
        } => {
            let candidates = world
                .jobs
                .values()
                .filter(|job| {
                    job_matches(
                        job,
                        learner,
                        name.as_deref(),
                        *gpus,
                        *cpus,
                        *min_memory_mib,
                        *max_memory_mib,
                    )
                })
                .collect::<Vec<_>>();
            let matched = candidates.iter().find_map(|job| {
                let state_at = latest_job_state_event_at(world, job.id, *state)?;
                ledger.events.iter().find_map(|event| match event {
                    EvidenceEvent::Command { at, command, .. }
                        if *at >= state_at && command_has_prefix(command, prefixes) =>
                    {
                        Some((job.id, *at, command))
                    }
                    _ => None,
                })
            });
            matched
                .map(|(job_id, command_at, command)| {
                    (
                        true,
                        format!(
                            "observed {command:?} at {} after matching job {} visited {state:?}",
                            command_at.0, job_id.0
                        ),
                    )
                })
                .unwrap_or_else(|| {
                    (
                        false,
                        format!(
                            "no command with prefixes {prefixes:?} followed {state:?} on a matching learner job"
                        ),
                    )
                })
        }
        Assertion::JobInspectionWhilePending {
            pending_reason,
            gpus,
            cpus,
            min_memory_mib,
            max_memory_mib,
            states,
        } => {
            let matched = ledger.events.iter().find_map(|event| match event {
                EvidenceEvent::Command { at, command, .. } => {
                    let job_id = inspected_job_id(command)?;
                    let job = world.jobs.get(&job_id)?;
                    if !job_matches(
                        job,
                        learner,
                        None,
                        *gpus,
                        *cpus,
                        *min_memory_mib,
                        *max_memory_mib,
                    ) || !states.iter().all(|state| job_visited_state(world, job.id, *state))
                    {
                        return None;
                    }
                    let pending_at =
                        world.event_log.iter().find_map(|world_event| match world_event.kind {
                            WorldEventKind::JobPending { job_id: pending_job_id, reason }
                                if pending_job_id == job_id && reason == *pending_reason =>
                            {
                                Some(world_event.at)
                            }
                            _ => None,
                        })?;
                    let started_at =
                        world.event_log.iter().find_map(|world_event| match world_event.kind {
                            WorldEventKind::JobStarted { job_id: started_job_id, .. }
                                if started_job_id == job_id =>
                            {
                                Some(world_event.at)
                            }
                            _ => None,
                        });
                    (*at >= pending_at && started_at.is_none_or(|started| *at < started))
                        .then_some((job_id, command, *at, pending_at, started_at))
                }
                _ => None,
            });
            matched
                .map(|(job_id, command, command_at, pending_at, started_at)| {
                    (
                        true,
                        format!(
                            "observed {command:?} for learner job {} at {} during {pending_reason:?} window {}..{:?}",
                            job_id.0,
                            command_at.0,
                            pending_at.0,
                            started_at.map(|at| at.0)
                        ),
                    )
                })
                .unwrap_or_else(|| {
                    (
                        false,
                        format!("no successful scontrol job inspection occurred while one matching learner job was pending for {pending_reason:?} and visited every state {states:?}"),
                    )
                })
        }
        Assertion::LearnerSweepEvaluation => evaluate_sweep_evaluation(world, ledger, learner),
        Assertion::LearnerJobVisitedState { state } => {
            let jobs: Vec<JobId> = world
                .jobs
                .values()
                .filter(|job| job.spec.user == learner)
                .map(|job| job.id)
                .collect();
            let matched = world.event_log.iter().any(|event| match &event.kind {
                WorldEventKind::JobStarted { job_id, .. } if state == &JobStatus::Running => {
                    jobs.contains(job_id)
                }
                WorldEventKind::JobFinished { job_id, status } if status == state => {
                    jobs.contains(job_id)
                }
                WorldEventKind::JobPending { job_id, .. } if state == &JobStatus::Pending => {
                    jobs.contains(job_id)
                }
                _ => false,
            });
            (matched, format!("learner jobs examined: {jobs:?}"))
        }
        Assertion::LearnerJobNamedVisitedState { name, states } => {
            let jobs: Vec<JobId> = world
                .jobs
                .values()
                .filter(|job| job.spec.user == learner && job.spec.name == *name)
                .map(|job| job.id)
                .collect();
            let matched = world.event_log.iter().any(|event| match &event.kind {
                WorldEventKind::JobStarted { job_id, .. } => {
                    jobs.contains(job_id) && states.contains(&JobStatus::Running)
                }
                WorldEventKind::JobFinished { job_id, status } => {
                    jobs.contains(job_id) && states.contains(status)
                }
                WorldEventKind::JobPending { job_id, .. } => {
                    jobs.contains(job_id) && states.contains(&JobStatus::Pending)
                }
                _ => false,
            });
            (
                matched,
                format!(
                    "learner jobs named {name:?} examined: {jobs:?}; expected states: {states:?}"
                ),
            )
        }
        Assertion::LearnerRecoveryJobCompleted {
            name,
            partition,
            gpus,
            cpus,
            minimum_memory_mib,
            maximum_memory_mib,
            minimum_time_limit_ms,
            maximum_time_limit_ms,
            after_job_name,
        } => {
            let candidates = world
                .jobs
                .values()
                .filter(|job| job.spec.user == learner && job.spec.name == *name)
                .collect::<Vec<_>>();
            let matched = candidates.iter().find_map(|job| {
                let submission_sequence = job_submission_sequence(world, job.id)?;
                let predecessor = world.jobs.values().find(|previous| {
                    previous.spec.user == learner
                        && previous.spec.name == *after_job_name
                        && job_terminal_sequence(world, previous.id)
                            .is_some_and(|sequence| sequence < submission_sequence)
                })?;
                let latest_checkpoint =
                    latest_readable_checkpoint_before(world, learner, name, submission_sequence)?;
                let checkpoint = command_option(&job.spec.command, "--resume-from-checkpoint")?;
                let unresolved_path = if checkpoint.starts_with('/') {
                    checkpoint
                } else {
                    format!("/home/{learner}/{checkpoint}")
                };
                let checkpoint_path = normalize_path(&unresolved_path).ok()?;
                (job.status == JobStatus::Completed
                    && partition.as_ref().is_none_or(|expected| job.spec.partition == *expected)
                    && job.spec.resources.gpus == *gpus
                    && job.spec.resources.cpus == *cpus
                    && job.spec.resources.memory_mib >= *minimum_memory_mib
                    && maximum_memory_mib
                        .is_none_or(|maximum| job.spec.resources.memory_mib <= maximum)
                    && job.spec.time_limit_ms >= *minimum_time_limit_ms
                    && maximum_time_limit_ms
                        .is_none_or(|maximum| job.spec.time_limit_ms <= maximum)
                    && checkpoint_path == latest_checkpoint)
                    .then_some((job.id, predecessor.id, checkpoint_path))
            });
            matched
                .map(|(job_id, predecessor_id, checkpoint)| {
                    (
                        true,
                        format!(
                            "learner recovery job {} completed after {} with the latest pre-submission checkpoint {}",
                            job_id.0, predecessor_id.0, checkpoint
                        ),
                    )
                })
                .unwrap_or_else(|| {
                    (
                        false,
                        format!(
                            "no completed learner job named {name:?} matched the recovery resources, followed {after_job_name:?}, and used the latest readable pre-submission checkpoint; candidates: {:?}",
                            candidates.iter().map(|job| job.id).collect::<Vec<_>>()
                        ),
                    )
                })
        }
        Assertion::LearnerDiagnosis { diagnosis } => {
            let matched = ledger.diagnoses.iter().any(|record| &record.diagnosis == diagnosis);
            (matched, format!("recorded diagnoses: {:?}", ledger.diagnoses))
        }
        Assertion::VirtualFileExists { path } => {
            let exists = world.fs.exists(path);
            (exists, format!("virtual file {path} exists={exists}"))
        }
        Assertion::VirtualFileWritten { path } => {
            let expected = normalize_path(path).ok();
            let written = ledger.events.iter().any(|event| match event {
                EvidenceEvent::FileWritten { path: observed, .. } => {
                    expected.as_ref().is_some_and(|expected| {
                        normalize_path(observed).is_ok_and(|observed| observed == *expected)
                    })
                }
                _ => false,
            });
            (written, format!("learner write recorded for {path}: {written}"))
        }
        Assertion::ActiveAllocationReleased => {
            let released = shell.active_job_id.is_none();
            (released, format!("active_job_id={:?}", shell.active_job_id))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn job_matches(
    job: &JobRecord,
    learner: &str,
    name: Option<&str>,
    gpus: Option<u16>,
    cpus: Option<u32>,
    min_memory_mib: Option<u64>,
    max_memory_mib: Option<u64>,
) -> bool {
    job.spec.user == learner
        && name.is_none_or(|expected| job.spec.name == expected)
        && gpus.is_none_or(|expected| job.spec.resources.gpus == expected)
        && cpus.is_none_or(|expected| job.spec.resources.cpus == expected)
        && min_memory_mib.is_none_or(|minimum| job.spec.resources.memory_mib >= minimum)
        && max_memory_mib.is_none_or(|maximum| job.spec.resources.memory_mib <= maximum)
}

fn job_visited_state(world: &SimulationWorld, job_id: JobId, state: JobStatus) -> bool {
    world.event_log.iter().any(|event| match event.kind {
        WorldEventKind::JobSubmitted { job_id: event_job_id } => {
            event_job_id == job_id && state == JobStatus::Submitted
        }
        WorldEventKind::JobPending { job_id: event_job_id, .. } => {
            event_job_id == job_id && state == JobStatus::Pending
        }
        WorldEventKind::JobStarted { job_id: event_job_id, .. } => {
            event_job_id == job_id && state == JobStatus::Running
        }
        WorldEventKind::JobFinished { job_id: event_job_id, status } => {
            event_job_id == job_id && state == status
        }
        _ => false,
    })
}

fn latest_job_state_event_at(
    world: &SimulationWorld,
    job_id: JobId,
    state: JobStatus,
) -> Option<SimTimeMs> {
    world
        .event_log
        .iter()
        .filter(|event| match event.kind {
            WorldEventKind::JobSubmitted { job_id: event_job_id } => {
                event_job_id == job_id && state == JobStatus::Submitted
            }
            WorldEventKind::JobPending { job_id: event_job_id, .. } => {
                event_job_id == job_id && state == JobStatus::Pending
            }
            WorldEventKind::JobStarted { job_id: event_job_id, .. } => {
                event_job_id == job_id && state == JobStatus::Running
            }
            WorldEventKind::JobFinished { job_id: event_job_id, status } => {
                event_job_id == job_id && state == status
            }
            _ => false,
        })
        .map(|event| event.at)
        .max()
}

fn command_has_prefix(command: &str, prefixes: &[String]) -> bool {
    prefixes.iter().any(|prefix| command.trim_start().starts_with(prefix))
}

fn inspected_job_id(command: &str) -> Option<JobId> {
    let tokens = command.split_whitespace().collect::<Vec<_>>();
    if tokens.first().copied() != Some("scontrol") || tokens.get(1).copied() != Some("show") {
        return None;
    }
    let raw = match tokens.get(2).copied()? {
        "job" | "jobid" => tokens.get(3).copied()?,
        token => token.strip_prefix("job=").or_else(|| token.strip_prefix("jobid="))?,
    };
    raw.parse::<u64>().ok().map(JobId)
}

fn job_family_name(name: &str) -> &str {
    name.split_once('[').map_or(name, |(family, _)| family)
}

fn evaluate_sweep_evaluation(
    world: &SimulationWorld,
    ledger: &EvidenceLedger,
    learner: &str,
) -> (bool, String) {
    let groups = ledger
        .events
        .iter()
        .filter_map(|event| match event {
            EvidenceEvent::Command { created_job_ids, .. } if created_job_ids.len() >= 4 => {
                Some(created_job_ids)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let matched = groups.iter().find_map(|group| {
        let sweep_jobs =
            group.iter().filter_map(|job_id| world.jobs.get(job_id)).collect::<Vec<_>>();
        let distinct_indices =
            sweep_jobs.iter().filter_map(|job| job.spec.array_index).collect::<BTreeSet<_>>();
        if sweep_jobs.len() != group.len()
            || distinct_indices.len() < 4
            || sweep_jobs.iter().any(|job| {
                job.spec.user != learner
                    || job_family_name(&job.spec.name) != "sweep"
                    || job.status != JobStatus::Completed
                    || job.ended_at.is_none()
            })
        {
            return None;
        }
        let latest_completion = sweep_jobs.iter().filter_map(|job| job.ended_at).max()?;
        let sweep_ids = sweep_jobs.iter().map(|job| job.id).collect::<BTreeSet<_>>();
        let evaluation = world.jobs.values().find(|job| {
            job.spec.user == learner
                && job.spec.name == "evaluate"
                && job
                    .spec
                    .dependency_after_ok
                    .is_some_and(|dependency| sweep_ids.contains(&dependency))
                && job.started_at.is_some_and(|started| started >= latest_completion)
        })?;
        Some((group.len(), evaluation.id))
    });
    matched
        .map(|(elements, evaluation)| {
            (
                true,
                format!(
                    "one {elements}-element sweep submission completed before dependent evaluation {} started",
                    evaluation.0
                ),
            )
        })
        .unwrap_or_else(|| {
            (
                false,
                format!(
                    "no coherent sweep submission of at least four completed elements released a dependent evaluation; groups examined={}",
                    groups.len()
                ),
            )
        })
}

fn job_submission_sequence(world: &SimulationWorld, job_id: JobId) -> Option<u64> {
    world.event_log.iter().find_map(|event| match event.kind {
        WorldEventKind::JobSubmitted { job_id: submitted_job_id } if submitted_job_id == job_id => {
            Some(event.sequence)
        }
        _ => None,
    })
}

fn job_terminal_sequence(world: &SimulationWorld, job_id: JobId) -> Option<u64> {
    world.event_log.iter().find_map(|event| match event.kind {
        WorldEventKind::JobFinished { job_id: finished_job_id, .. }
            if finished_job_id == job_id =>
        {
            Some(event.sequence)
        }
        _ => None,
    })
}

fn latest_readable_checkpoint_before(
    world: &SimulationWorld,
    learner: &str,
    recovery_name: &str,
    before_sequence: u64,
) -> Option<String> {
    let checkpoint_root = format!("/home/{learner}/checkpoints/");
    world
        .event_log
        .iter()
        .filter_map(|event| match &event.kind {
            WorldEventKind::ArtifactWritten { job_id, path }
                if event.sequence < before_sequence
                    && world.jobs.get(job_id).is_some_and(|job| job.spec.name != recovery_name) =>
            {
                let normalized = normalize_path(path).ok()?;
                (normalized.starts_with(&checkpoint_root)
                    && normalized.ends_with(".pt")
                    && world.fs.read_file(&normalized).is_ok())
                .then_some((event.at, event.sequence, normalized))
            }
            _ => None,
        })
        .max_by_key(|(at, sequence, _)| (*at, *sequence))
        .map(|(_, _, path)| path)
}

fn command_option(command: &str, option: &str) -> Option<String> {
    let tokens = command.split_whitespace().collect::<Vec<_>>();
    for (index, token) in tokens.iter().enumerate() {
        if *token == option {
            return tokens.get(index + 1).map(|value| (*value).to_string());
        }
        if let Some(value) = token.strip_prefix(&format!("{option}=")) {
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::SimulationWorld;
    use slurm_model::{JobSpec, Tres};

    fn assertion_passes(
        assertion: Assertion,
        world: &SimulationWorld,
        ledger: &EvidenceLedger,
    ) -> bool {
        evaluate_assertion(&assertion, world, &ShellSession::learner(), ledger, "learner").0
    }

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

    #[test]
    fn all_requires_one_job_to_match_resources_and_visit_every_state() {
        let mut world = SimulationWorld::dgx_h200_8(2);
        let spec = JobSpec {
            name: "focused".into(),
            resources: Tres {
                cpus: 8,
                memory_mib: 64 * 1024,
                gpu_type: Some("h200".into()),
                gpus: 1,
            },
            ..JobSpec::default()
        };
        let job_id = world.submit_job(spec).unwrap();
        world.complete_interactive_job(job_id).unwrap();
        let mut ledger = EvidenceLedger::new();
        ledger.record_command(SimTimeMs::ZERO, "sacct");

        let assertion = Assertion::All {
            assertions: vec![
                Assertion::LearnerJobMatches {
                    name: Some("focused".into()),
                    gpus: Some(1),
                    cpus: Some(8),
                    min_memory_mib: Some(64 * 1024),
                    max_memory_mib: Some(64 * 1024),
                    states: vec![JobStatus::Running, JobStatus::Completed],
                },
                Assertion::ExactCommandUsed { command: "sacct".into() },
            ],
        };
        assert!(assertion_passes(assertion, &world, &ledger));

        let split_state_assertion = Assertion::LearnerJobMatches {
            name: None,
            gpus: Some(1),
            cpus: Some(8),
            min_memory_mib: None,
            max_memory_mib: None,
            states: vec![JobStatus::Running, JobStatus::OutOfMemory],
        };
        assert!(!assertion_passes(split_state_assertion, &world, &ledger));
    }

    #[test]
    fn commands_are_linked_to_their_allocation_and_matching_job_state() {
        let mut world = SimulationWorld::dgx_h200_8(3);
        let job_id = world
            .submit_job(JobSpec {
                name: "gpu-shell".into(),
                resources: Tres {
                    cpus: 8,
                    memory_mib: 64 * 1024,
                    gpu_type: Some("h200".into()),
                    gpus: 1,
                },
                ..JobSpec::default()
            })
            .unwrap();
        let mut ledger = EvidenceLedger::new();
        ledger.record_command_with_context(
            SimTimeMs::ZERO,
            "nvidia-smi -L",
            Some(job_id),
            Vec::new(),
        );
        world.advance_by(10).unwrap();
        world.complete_interactive_job(job_id).unwrap();
        ledger.record_command(SimTimeMs(9), "sacct");
        ledger.record_command(SimTimeMs(10), "sacct -j 10000");

        assert!(assertion_passes(
            Assertion::CommandInMatchingAllocation {
                prefixes: vec!["nvidia-smi".into()],
                gpus: Some(1),
                cpus: Some(8),
                min_memory_mib: Some(64 * 1024),
                max_memory_mib: Some(64 * 1024),
                states: vec![JobStatus::Running, JobStatus::Completed],
            },
            &world,
            &ledger,
        ));
        assert!(assertion_passes(
            Assertion::CommandAfterMatchingJobState {
                prefixes: vec!["sacct".into()],
                name: Some("gpu-shell".into()),
                gpus: Some(1),
                cpus: Some(8),
                min_memory_mib: None,
                max_memory_mib: None,
                state: JobStatus::Completed,
            },
            &world,
            &ledger,
        ));

        let only_early =
            EvidenceLedger { events: vec![ledger.events[1].clone()], diagnoses: Vec::new() };
        assert!(!assertion_passes(
            Assertion::CommandAfterMatchingJobState {
                prefixes: vec!["sacct".into()],
                name: Some("gpu-shell".into()),
                gpus: None,
                cpus: None,
                min_memory_mib: None,
                max_memory_mib: None,
                state: JobStatus::Completed,
            },
            &world,
            &only_early,
        ));
    }

    #[test]
    fn pending_inspection_must_name_the_job_during_its_reason_window() {
        let mut world = SimulationWorld::dgx_h200_8(4);
        let blocker = world
            .submit_job(JobSpec {
                name: "blocker".into(),
                user: "other".into(),
                resources: Tres {
                    cpus: 8,
                    memory_mib: 64 * 1024,
                    gpu_type: Some("h200".into()),
                    gpus: 8,
                },
                ..JobSpec::default()
            })
            .unwrap();
        let waiting = world
            .submit_job(JobSpec {
                name: "waiting".into(),
                resources: Tres {
                    cpus: 4,
                    memory_mib: 8 * 1024,
                    gpu_type: Some("h200".into()),
                    gpus: 1,
                },
                ..JobSpec::default()
            })
            .unwrap();
        let mut during = EvidenceLedger::new();
        during.record_command(world.now, format!("scontrol show job {}", waiting.0));
        assert!(assertion_passes(
            Assertion::JobInspectionWhilePending {
                pending_reason: PendingReason::Resources,
                gpus: Some(1),
                cpus: Some(4),
                min_memory_mib: Some(8 * 1024),
                max_memory_mib: Some(8 * 1024),
                states: vec![JobStatus::Pending],
            },
            &world,
            &during,
        ));

        world.advance_by(1).unwrap();
        world.complete_interactive_job(blocker).unwrap();
        assert!(assertion_passes(
            Assertion::JobInspectionWhilePending {
                pending_reason: PendingReason::Resources,
                gpus: Some(1),
                cpus: Some(4),
                min_memory_mib: Some(8 * 1024),
                max_memory_mib: Some(8 * 1024),
                states: vec![JobStatus::Pending, JobStatus::Running],
            },
            &world,
            &during,
        ));
        let mut after = EvidenceLedger::new();
        after.record_command(world.now, format!("scontrol show job {}", waiting.0));
        assert!(!assertion_passes(
            Assertion::JobInspectionWhilePending {
                pending_reason: PendingReason::Resources,
                gpus: Some(1),
                cpus: Some(4),
                min_memory_mib: Some(8 * 1024),
                max_memory_mib: Some(8 * 1024),
                states: vec![JobStatus::Pending, JobStatus::Running],
            },
            &world,
            &after,
        ));
    }

    #[test]
    fn sweep_evaluation_requires_four_completed_elements_and_release_order() {
        let mut world = SimulationWorld::dgx_h200_8(5);
        let mut sweep_ids = Vec::new();
        for index in 0..4 {
            let job_id = world
                .submit_job(JobSpec {
                    name: "sweep".into(),
                    array_index: Some(index),
                    ..JobSpec::default()
                })
                .unwrap();
            world.complete_interactive_job(job_id).unwrap();
            sweep_ids.push(job_id);
        }
        let evaluation = world
            .submit_job(JobSpec {
                name: "evaluate".into(),
                dependency_after_ok: sweep_ids.last().copied(),
                ..JobSpec::default()
            })
            .unwrap();
        let mut ledger = EvidenceLedger::new();
        ledger.record_command_with_context(
            SimTimeMs::ZERO,
            "sbatch sweep.sbatch",
            None,
            sweep_ids.clone(),
        );
        assert!(world.jobs[&evaluation].started_at.is_some());
        assert!(assertion_passes(Assertion::LearnerSweepEvaluation, &world, &ledger,));

        world.jobs.get_mut(&sweep_ids[0]).unwrap().status = JobStatus::Running;
        assert!(!assertion_passes(Assertion::LearnerSweepEvaluation, &world, &ledger,));
    }

    #[test]
    fn learner_writes_are_distinct_from_seeded_files() {
        let world = SimulationWorld::dgx_h200_8(6);
        let mut ledger = EvidenceLedger::new();
        assert!(!assertion_passes(
            Assertion::VirtualFileWritten { path: "/home/learner/train.sbatch".into() },
            &world,
            &ledger,
        ));
        ledger.events.push(EvidenceEvent::FileWritten {
            at: SimTimeMs::ZERO,
            path: "/home/learner/./train.sbatch".into(),
        });
        assert!(assertion_passes(
            Assertion::VirtualFileWritten { path: "/home/learner/train.sbatch".into() },
            &world,
            &ledger,
        ));
    }

    #[test]
    fn recovery_requires_exact_resources_order_and_latest_checkpoint() {
        let mut world = SimulationWorld::dgx_h200_8(7);
        let predecessor = world
            .submit_job(JobSpec {
                name: "failed-train".into(),
                resources: Tres {
                    cpus: 8,
                    memory_mib: 16 * 1024,
                    gpu_type: Some("h200".into()),
                    gpus: 1,
                },
                time_limit_ms: 30 * 60 * 1_000,
                command: "python train.py --epochs 3".into(),
                workload_id: "pytorch-training-v1".into(),
                ..JobSpec::default()
            })
            .unwrap();
        world.advance_by(60_000).unwrap();
        assert_eq!(world.jobs[&predecessor].status, JobStatus::OutOfMemory);
        assert!(world.fs.read_file("/home/learner/checkpoints/epoch-002.pt").is_ok());

        let stale = world
            .submit_job(JobSpec {
                name: "recovery".into(),
                resources: Tres {
                    cpus: 8,
                    memory_mib: 64 * 1024,
                    gpu_type: Some("h200".into()),
                    gpus: 1,
                },
                time_limit_ms: 30 * 60 * 1_000,
                command:
                    "python train.py --epochs 1 --resume-from-checkpoint checkpoints/epoch-001.pt"
                        .into(),
                workload_id: "pytorch-training-v1".into(),
                ..JobSpec::default()
            })
            .unwrap();
        world.advance_by(60_000).unwrap();
        assert_eq!(world.jobs[&stale].status, JobStatus::Completed);

        let assertion = Assertion::LearnerRecoveryJobCompleted {
            name: "recovery".into(),
            partition: Some("gpu".into()),
            gpus: 1,
            cpus: 8,
            minimum_memory_mib: 64 * 1024,
            maximum_memory_mib: Some(64 * 1024),
            minimum_time_limit_ms: 30 * 60 * 1_000,
            maximum_time_limit_ms: Some(30 * 60 * 1_000),
            after_job_name: "failed-train".into(),
        };
        assert!(!assertion_passes(assertion.clone(), &world, &EvidenceLedger::new(),));

        let current = world
            .submit_job(JobSpec {
                name: "recovery".into(),
                resources: Tres {
                    cpus: 8,
                    memory_mib: 64 * 1024,
                    gpu_type: Some("h200".into()),
                    gpus: 1,
                },
                time_limit_ms: 30 * 60 * 1_000,
                command:
                    "python train.py --epochs 1 --resume-from-checkpoint checkpoints/epoch-002.pt"
                        .into(),
                workload_id: "pytorch-training-v1".into(),
                ..JobSpec::default()
            })
            .unwrap();
        world.advance_by(60_000).unwrap();
        assert_eq!(world.jobs[&current].status, JobStatus::Completed);
        assert!(assertion_passes(assertion, &world, &EvidenceLedger::new()));

        let original = world.jobs[&current].spec.clone();
        let mut invalid_specs = Vec::new();
        let mut wrong_partition = original.clone();
        wrong_partition.partition = "cpu".into();
        invalid_specs.push(wrong_partition);
        let mut wrong_gpus = original.clone();
        wrong_gpus.resources.gpus = 2;
        invalid_specs.push(wrong_gpus);
        let mut wrong_cpus = original.clone();
        wrong_cpus.resources.cpus = 9;
        invalid_specs.push(wrong_cpus);
        let mut too_much_memory = original.clone();
        too_much_memory.resources.memory_mib = 96 * 1024;
        invalid_specs.push(too_much_memory);
        let mut too_much_time = original.clone();
        too_much_time.time_limit_ms = 60 * 60 * 1_000;
        invalid_specs.push(too_much_time);

        for invalid in invalid_specs {
            world.jobs.get_mut(&current).unwrap().spec = invalid;
            assert!(!assertion_passes(
                Assertion::LearnerRecoveryJobCompleted {
                    name: "recovery".into(),
                    partition: Some("gpu".into()),
                    gpus: 1,
                    cpus: 8,
                    minimum_memory_mib: 64 * 1024,
                    maximum_memory_mib: Some(64 * 1024),
                    minimum_time_limit_ms: 30 * 60 * 1_000,
                    maximum_time_limit_ms: Some(30 * 60 * 1_000),
                    after_job_name: "failed-train".into(),
                },
                &world,
                &EvidenceLedger::new(),
            ));
        }
        world.jobs.get_mut(&current).unwrap().spec = original;
    }

    #[test]
    fn grades_array_dependency_and_named_recovery_from_world_state() {
        let mut world = SimulationWorld::dgx_h200_8(2);
        let mut array = JobSpec {
            name: "sweep".into(),
            user: "learner".into(),
            array_index: Some(0),
            ..JobSpec::default()
        };
        let parent = world.submit_job(array.clone()).unwrap();
        array.array_index = Some(1);
        world.submit_job(array).unwrap();
        world.complete_interactive_job(parent).unwrap();

        let recovery = JobSpec {
            name: "train-resume".into(),
            user: "learner".into(),
            dependency_after_ok: Some(parent),
            ..JobSpec::default()
        };
        world.submit_job(recovery).unwrap();

        let checks = vec![
            PracticalCheck {
                id: "array".into(),
                points: 25,
                critical: true,
                assertion: Assertion::LearnerArrayTaskCount { minimum: 2 },
            },
            PracticalCheck {
                id: "dependency".into(),
                points: 25,
                critical: true,
                assertion: Assertion::LearnerJobHasDependency,
            },
            PracticalCheck {
                id: "recovery".into(),
                points: 50,
                critical: true,
                assertion: Assertion::LearnerJobNamedVisitedState {
                    name: "train-resume".into(),
                    states: vec![JobStatus::Running, JobStatus::Completed],
                },
            },
        ];
        let score = evaluate_practical(
            &checks,
            &world,
            &ShellSession::learner(),
            &EvidenceLedger::new(),
            "learner",
        );

        assert_eq!(score.earned_points, 100);
        assert!(score.all_critical_passed);
    }
}
