#![forbid(unsafe_code)]

//! Pure simulation session used by native tests, the WASM worker, and the UI bridge.

mod course;
mod lab04;
mod lab06;
mod lab07;
mod lab09;
mod pending;

pub mod cert_bank;

use course::{generic_hints, generic_lab_checks, lab_for_scenario, COURSE_LABS};
use dgxlab_contracts::{
    SessionId, SimRequest, SimResponse, SIMULATOR_COMPATIBILITY_VERSION, TerminalLine, UiGpuTile,
    UiJobSummary, UiLabStep, UiWorldView, WORKER_PROTOCOL_VERSION,
};
use grading::{evaluate_practical, EvidenceLedger, PracticalCheck};
use lab04::{lab04_checks, lab04_hints, LAB04_ID};
use lab06::{lab06_checks, lab06_hints, LAB06_ID};
use lab07::{lab07_checks, lab07_hints, LAB07_ID};
use lab09::{lab09_checks, lab09_hints, LAB09_ID};
use pending::explain_pending;
use persistence_codec::SessionBundle;
use scenarios::initialize_scenario;
use serde::{Deserialize, Serialize};
use sim_core::SimulationWorld;
use slurm_model::JobStatus;
use virtual_shell::{execute_line, CommandResult, ShellSession};

pub use course::{LabMeta, COURSE_LABS as BUILTIN_LABS};

/// Authoritative learner session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimSession {
    world: SimulationWorld,
    shell: ShellSession,
    ledger: EvidenceLedger,
    seq: u64,
    scenario_id: String,
    seed: u64,
    hint_level: u8,
    lab_id: String,
}

impl SimSession {
    pub fn new(scenario_id: &str, seed: u64) -> Result<Self, SessionError> {
        let world = initialize_scenario(scenario_id, seed)?;
        let lab_id = lab_for_scenario(scenario_id).unwrap_or("").to_string();
        Ok(Self {
            world,
            shell: ShellSession::learner(),
            ledger: EvidenceLedger::new(),
            seq: 0,
            scenario_id: scenario_id.into(),
            seed,
            hint_level: 0,
            lab_id,
        })
    }

    /// Open a lab by course id (maps to scenario).
    pub fn open_lab(lab_id: &str, seed: u64) -> Result<Self, SessionError> {
        let meta = COURSE_LABS
            .iter()
            .find(|lab| lab.id == lab_id)
            .ok_or_else(|| SessionError::Codec(format!("unknown lab {lab_id}")))?;
        let mut session = Self::new(meta.scenario, seed)?;
        session.lab_id = meta.id.into();
        Ok(session)
    }

    #[must_use]
    pub fn seq(&self) -> u64 {
        self.seq
    }

    #[must_use]
    pub fn state_digest(&self) -> String {
        self.world.state_digest()
    }

    #[must_use]
    pub fn practical_percent(&self) -> u8 {
        self.view().practical_percent
    }

    #[must_use]
    pub fn critical_practical_passed(&self) -> bool {
        let checks = checks_for_lab(&self.lab_id);
        if checks.is_empty() {
            return true;
        }
        evaluate_practical(
            &checks,
            &self.world,
            &self.shell,
            &self.ledger,
            "learner",
        )
        .all_critical_passed
    }

    #[must_use]
    pub fn view(&self) -> UiWorldView {
        build_view(self)
    }

    pub fn export_bundle(&self) -> SessionBundle {
        SessionBundle::new(
            SessionId(format!("local-{}", self.seed)),
            env!("CARGO_PKG_VERSION"),
            "1.0.0",
            self.world.clone(),
            self.shell.clone(),
            self.ledger.clone(),
        )
    }

    pub fn export_json(&self) -> Result<String, SessionError> {
        serde_json::to_string(self).map_err(|error| SessionError::Codec(error.to_string()))
    }

    pub fn import_json(json: &str) -> Result<Self, SessionError> {
        serde_json::from_str(json).map_err(|error| SessionError::Codec(error.to_string()))
    }

    pub fn handle(&mut self, request: SimRequest) -> SimResponse {
        match request {
            SimRequest::Initialize { scenario_id, seed } => match Self::new(&scenario_id, seed) {
                Ok(session) => {
                    *self = session;
                    self.seq = self.seq.saturating_add(1);
                    SimResponse::Ready {
                        protocol_version: WORKER_PROTOCOL_VERSION.into(),
                        compatibility_version: SIMULATOR_COMPATIBILITY_VERSION.into(),
                        seq: self.seq,
                        state: self.view(),
                    }
                }
                Err(error) => self.error("init_failed", error.to_string()),
            },
            SimRequest::ExecuteCommand { command } => {
                self.ledger
                    .record_command(self.world.now, command.clone());
                let result = execute_line(&mut self.world, &mut self.shell, &command);
                self.seq = self.seq.saturating_add(1);
                SimResponse::CommandResult {
                    seq: self.seq,
                    prompt: self.shell.prompt(),
                    lines: decorate_command_lines(&command, result),
                    state: self.view(),
                }
            }
            SimRequest::AdvanceClock { delta_ms } => {
                if let Err(error) = self.world.advance_by(delta_ms) {
                    return self.error("advance_failed", error.to_string());
                }
                self.bump_state()
            }
            SimRequest::SetClockSpeed { multiplier } => {
                self.world.clock_multiplier = multiplier.max(1);
                self.bump_state()
            }
            SimRequest::Pause => {
                self.world.paused = true;
                self.bump_state()
            }
            SimRequest::Resume => {
                self.world.paused = false;
                self.bump_state()
            }
            SimRequest::Reset { scenario_id, seed } => match Self::new(&scenario_id, seed) {
                Ok(mut session) => {
                    session.seq = self.seq.saturating_add(1);
                    let seq = session.seq;
                    let state = session.view();
                    *self = session;
                    SimResponse::State { seq, state }
                }
                Err(error) => self.error("reset_failed", error.to_string()),
            },
            SimRequest::Snapshot => self.bump_state(),
            SimRequest::CancelJob { job_id } => {
                if let Err(error) = self.world.cancel_job(dgxlab_contracts::JobId(job_id)) {
                    return self.error("cancel_failed", error.to_string());
                }
                self.bump_state()
            }
            SimRequest::UseHint => {
                let hints = hints_for_lab(&self.lab_id);
                if self.hint_level < hints.len() as u8 {
                    self.hint_level = self.hint_level.saturating_add(1);
                    self.ledger.events.push(grading::EvidenceEvent::HintUsed {
                        at: self.world.now,
                        hint_id: format!("{}-hint-{}", self.lab_id, self.hint_level),
                        level: self.hint_level,
                    });
                }
                self.bump_state()
            }
            SimRequest::ReadVfs { path } => {
                let resolved = match virtual_shell::resolve_path_for_session(&self.shell, &path) {
                    Ok(path) => path,
                    Err(message) => return self.error("vfs_path", message),
                };
                match self.world.fs.read_text(&resolved) {
                    Ok(content) => {
                        self.seq = self.seq.saturating_add(1);
                        SimResponse::FileContent {
                            seq: self.seq,
                            path: resolved,
                            content,
                        }
                    }
                    Err(error) => self.error("vfs_read", error.to_string()),
                }
            }
            SimRequest::WriteVfs { path, content } => {
                let resolved = match virtual_shell::resolve_path_for_session(&self.shell, &path) {
                    Ok(path) => path,
                    Err(message) => return self.error("vfs_path", message),
                };
                match self.world.fs.write_file(&resolved, content.as_bytes()) {
                    Ok(()) => {
                        self.ledger.events.push(grading::EvidenceEvent::FileWritten {
                            at: self.world.now,
                            path: resolved,
                        });
                        self.bump_state()
                    }
                    Err(error) => self.error("vfs_write", error.to_string()),
                }
            }
        }
    }

    fn bump_state(&mut self) -> SimResponse {
        self.seq = self.seq.saturating_add(1);
        SimResponse::State {
            seq: self.seq,
            state: self.view(),
        }
    }

    fn error(&mut self, code: &str, message: String) -> SimResponse {
        self.seq = self.seq.saturating_add(1);
        SimResponse::Error {
            code: code.into(),
            message,
            seq: self.seq,
        }
    }
}

fn hints_for_lab(lab_id: &str) -> Vec<&'static str> {
    match lab_id {
        LAB04_ID => lab04_hints().to_vec(),
        LAB06_ID => lab06_hints().to_vec(),
        LAB07_ID => lab07_hints().to_vec(),
        LAB09_ID => lab09_hints().to_vec(),
        other => generic_hints(other),
    }
}

fn checks_for_lab(lab_id: &str) -> Vec<PracticalCheck> {
    match lab_id {
        LAB04_ID => lab04_checks(),
        LAB06_ID => lab06_checks(),
        LAB07_ID => lab07_checks(),
        LAB09_ID => lab09_checks(),
        other if !other.is_empty() => generic_lab_checks(other),
        _ => Vec::new(),
    }
}

fn decorate_command_lines(command: &str, result: CommandResult) -> Vec<TerminalLine> {
    let mut lines = vec![TerminalLine::input(command.to_string())];
    lines.extend(result.lines);
    lines
}

fn build_view(session: &SimSession) -> UiWorldView {
    let world = &session.world;
    let shell = &session.shell;
    let compute = world
        .cluster
        .nodes
        .values()
        .find(|node| node.id != "dgx-login-01")
        .or_else(|| world.cluster.nodes.values().next());

    let gpus = compute
        .map(|node| {
            node.gpus
                .iter()
                .map(|gpu| {
                    let status = if gpu.allocated_to.is_some() {
                        "Allocated"
                    } else {
                        match gpu.health {
                            slurm_model::GpuHealth::Ok => "Idle",
                            slurm_model::GpuHealth::Warning => "Warning",
                            slurm_model::GpuHealth::Failed => "Failed",
                        }
                    };
                    UiGpuTile {
                        index: gpu.index,
                        model: gpu.model.clone(),
                        status: status.into(),
                        owner_job_id: gpu.allocated_to.map(|id| id.0),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let jobs = world
        .jobs
        .values()
        .map(|job| {
            let pending = job.status == JobStatus::Pending;
            UiJobSummary {
                id: job.id.0,
                name: job.spec.name.clone(),
                user: job.spec.user.clone(),
                status: format!("{:?}", job.status).to_uppercase(),
                pending_reason: pending.then(|| job.pending_reason.display_name().into()),
                pending_explanation: pending
                    .then(|| explain_pending(job.pending_reason).to_string()),
                gpus: job.spec.resources.gpus,
                cpus: job.spec.resources.cpus,
                memory_mib: job.spec.resources.memory_mib,
            }
        })
        .collect();

    let (lab_steps, practical_percent, lab_complete) = lab_progress(session);
    let hints = hints_for_lab(&session.lab_id);
    let hint_text = if session.hint_level == 0 {
        None
    } else {
        hints
            .get(session.hint_level as usize - 1)
            .map(|text| (*text).into())
    };

    UiWorldView {
        scenario_id: session.scenario_id.clone(),
        seed: session.seed,
        now_ms: world.now.0,
        paused: world.paused,
        clock_multiplier: world.clock_multiplier,
        state_digest: world.state_digest(),
        prompt: shell.prompt(),
        gpus,
        jobs,
        node_status: compute
            .map(|node| format!("{:?}", node.status).to_lowercase())
            .unwrap_or_else(|| "unknown".into()),
        lab_steps,
        hint_level: session.hint_level,
        hint_text,
        lab_complete,
        practical_percent,
    }
}

fn lab_progress(session: &SimSession) -> (Vec<UiLabStep>, u8, bool) {
    let checks = checks_for_lab(&session.lab_id);
    if checks.is_empty() {
        return (Vec::new(), 0, false);
    }
    let score = evaluate_practical(
        &checks,
        &session.world,
        &session.shell,
        &session.ledger,
        "learner",
    );
    let steps = score
        .results
        .iter()
        .map(|result| UiLabStep {
            id: result.id.clone(),
            label: result.id.clone(),
            complete: result.passed,
            critical: result.critical,
        })
        .collect::<Vec<_>>();
    let percent = if score.possible_points == 0 {
        0
    } else {
        ((score.earned_points * 100) / score.possible_points) as u8
    };
    let complete = score.all_critical_passed && percent >= 80;
    (steps, percent, complete)
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error(transparent)]
    Scenario(#[from] scenarios::ScenarioError),
    #[error("session codec error: {0}")]
    Codec(String),
}

pub fn digest_for_transcript(
    scenario_id: &str,
    seed: u64,
    commands: &[&str],
) -> Result<String, SessionError> {
    let mut session = SimSession::new(scenario_id, seed)?;
    for command in commands {
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: (*command).into(),
        });
    }
    Ok(session.state_digest())
}

pub fn lab04_completes(seed: u64) -> Result<(bool, u8, String), SessionError> {
    let commands = [
        "sinfo",
        "srun --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:30:00 --pty bash",
        "echo $SLURM_JOB_ID",
        "echo $CUDA_VISIBLE_DEVICES",
        "nvidia-smi -L",
        "exit",
        "sacct",
    ];
    let mut session = SimSession::new("guided-one-gpu", seed)?;
    for command in commands {
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: command.into(),
        });
    }
    let view = session.view();
    Ok((
        view.lab_complete,
        view.practical_percent,
        session.state_digest(),
    ))
}

pub fn lab06_batch_path(seed: u64) -> Result<(bool, String), SessionError> {
    let mut session = SimSession::new("dgx-h200-8", seed)?;
    let _ = session.handle(SimRequest::ExecuteCommand {
        command: "sbatch train.sbatch".into(),
    });
    let _ = session.handle(SimRequest::AdvanceClock {
        delta_ms: 30 * 60 * 1_000,
    });
    let _ = session.handle(SimRequest::ExecuteCommand {
        command: "sacct".into(),
    });
    let _ = session.handle(SimRequest::ExecuteCommand {
        command: "ls logs".into(),
    });
    let view = session.view();
    Ok((
        view.lab_complete || view.practical_percent >= 80,
        session.state_digest(),
    ))
}

pub fn lab07_contention_path(seed: u64) -> Result<(bool, String, String), SessionError> {
    let mut session = SimSession::new("dgx-contended", seed)?;
    let _ = session.handle(SimRequest::ExecuteCommand {
        command: "sbatch train.sbatch".into(),
    });
    let pending_reason = session
        .view()
        .jobs
        .iter()
        .find(|job| job.user == "learner")
        .and_then(|job| job.pending_reason.clone())
        .unwrap_or_default();
    let _ = session.handle(SimRequest::ExecuteCommand {
        command: "squeue".into(),
    });
    if let Some(job) = session.view().jobs.iter().find(|job| job.user == "learner") {
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: format!("scontrol show job {}", job.id),
        });
    }
    let _ = session.handle(SimRequest::AdvanceClock {
        delta_ms: 4 * 60 * 60 * 1_000,
    });
    let view = session.view();
    let started = view.jobs.iter().any(|job| {
        job.user == "learner" && (job.status == "RUNNING" || job.status == "COMPLETED")
    });
    Ok((started, pending_reason, session.state_digest()))
}

pub fn lab09_failure_path(seed: u64) -> Result<(bool, String), SessionError> {
    let mut session = SimSession::new("dgx-degraded", seed)?;
    let _ = session.handle(SimRequest::ExecuteCommand {
        command: "sacct".into(),
    });
    let _ = session.handle(SimRequest::ExecuteCommand {
        command: "ls checkpoints".into(),
    });
    let _ = session.handle(SimRequest::ExecuteCommand {
        command: "ls logs".into(),
    });
    // Resubmit a safer job as recovery practice.
    let _ = session.handle(SimRequest::ExecuteCommand {
        command: "sbatch train.sbatch".into(),
    });
    let view = session.view();
    Ok((view.lab_complete || view.practical_percent >= 50, session.state_digest()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sinfo_returns_authoritative_snapshot() {
        let mut session = SimSession::new("dgx-h200-8", 42).expect("session");
        let response = session.handle(SimRequest::ExecuteCommand {
            command: "sinfo".into(),
        });
        assert!(matches!(response, SimResponse::CommandResult { .. }));
    }

    #[test]
    fn equal_transcripts_share_digest() {
        let commands = ["sinfo", "squeue"];
        let first = digest_for_transcript("dgx-h200-8", 7, &commands).unwrap();
        let second = digest_for_transcript("dgx-h200-8", 7, &commands).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn lab04_transcript_completes_practical() {
        let (complete, percent, digest) = lab04_completes(42).expect("lab04");
        assert!(complete, "lab should complete");
        assert!(percent >= 80, "percent={percent}");
        let again = lab04_completes(42).expect("lab04 again");
        assert_eq!(digest, again.2);
    }

    #[test]
    fn session_json_round_trip_preserves_digest() {
        let mut session = SimSession::new("guided-one-gpu", 9).unwrap();
        let _ = session.handle(SimRequest::ExecuteCommand {
            command: "sinfo".into(),
        });
        let json = session.export_json().unwrap();
        let restored = SimSession::import_json(&json).unwrap();
        assert_eq!(session.state_digest(), restored.state_digest());
    }

    #[test]
    fn vfs_write_and_read_round_trip() {
        let mut session = SimSession::new("dgx-h200-8", 3).unwrap();
        let write = session.handle(SimRequest::WriteVfs {
            path: "train.sbatch".into(),
            content: "#!/bin/bash\n#SBATCH --gres=gpu:h200:1\npython train.py\n".into(),
        });
        assert!(matches!(write, SimResponse::State { .. }));
        let read = session.handle(SimRequest::ReadVfs {
            path: "train.sbatch".into(),
        });
        assert!(matches!(read, SimResponse::FileContent { .. }));
    }

    #[test]
    fn batch_submission_path_works() {
        let (ok, digest) = lab06_batch_path(11).expect("lab06");
        assert!(ok, "batch path should score well");
        let again = lab06_batch_path(11).expect("lab06 again");
        assert_eq!(digest, again.1);
    }

    #[test]
    fn contended_job_pending_then_starts() {
        let (started, reason, digest) = lab07_contention_path(5).expect("lab07");
        assert_eq!(reason, "Resources");
        assert!(started);
        let again = lab07_contention_path(5).expect("lab07 again");
        assert_eq!(digest, again.2);
    }

    #[test]
    fn degraded_failure_lab_path() {
        let (ok, digest) = lab09_failure_path(2).expect("lab09");
        assert!(ok);
        let again = lab09_failure_path(2).expect("lab09 again");
        assert_eq!(digest, again.1);
    }

    #[test]
    fn array_script_submits_multiple_tasks() {
        let mut session = SimSession::new("dgx-h200-8", 4).unwrap();
        let script = "#!/bin/bash\n#SBATCH --job-name=arr\n#SBATCH --array=1-3\n#SBATCH --gres=gpu:h200:1\n#SBATCH --cpus-per-task=4\n#SBATCH --mem=16G\npython train.py\n";
        let _ = session.handle(SimRequest::WriteVfs {
            path: "array.sbatch".into(),
            content: script.into(),
        });
        let response = session.handle(SimRequest::ExecuteCommand {
            command: "sbatch array.sbatch".into(),
        });
        match response {
            SimResponse::CommandResult { lines, state, .. } => {
                assert!(lines.iter().any(|line| line.text.contains("array") || line.text.contains("Submitted")));
                let learner_jobs = state.jobs.iter().filter(|job| job.user == "learner").count();
                assert!(learner_jobs >= 3, "expected array expansion, got {learner_jobs}");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn course_has_twelve_labs() {
        assert_eq!(COURSE_LABS.len(), 12);
    }
}
