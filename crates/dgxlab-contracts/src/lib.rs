#![forbid(unsafe_code)]

//! Shared versioned contracts used across the DGX Lab workspace.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Wire protocol version between UI and simulation worker.
pub const WORKER_PROTOCOL_VERSION: &str = "dgxlab.worker/v1";
/// Compatibility version for deterministic replay across builds.
pub const SIMULATOR_COMPATIBILITY_VERSION: &str = "0.1.0";
pub const SESSION_FORMAT_VERSION: &str = "1.0.0";
pub const SCENARIO_SCHEMA: &str = "dgxlab.scenario/v1";
pub const QUESTION_SCHEMA: &str = "dgxlab.question/v1";

macro_rules! numeric_id {
    ($name:ident, $inner:ty) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub $inner);

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        // Serialize as a string so JSON map keys (e.g. jobs: { "10000": ... }) are valid.
        impl Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let raw = String::deserialize(deserializer)?;
                let value = raw.parse::<$inner>().map_err(serde::de::Error::custom)?;
                Ok(Self(value))
            }
        }
    };
}

numeric_id!(JobId, u64);
numeric_id!(EventId, u64);
numeric_id!(StepId, u32);
numeric_id!(SnapshotId, u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SimTimeMs(pub u64);

impl SimTimeMs {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub fn saturating_add(self, delta_ms: u64) -> Self {
        Self(self.0.saturating_add(delta_ms))
    }
}

impl Display for SimTimeMs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let total_seconds = self.0 / 1_000;
        let hours = total_seconds / 3_600;
        let minutes = (total_seconds % 3_600) / 60;
        let seconds = total_seconds % 60;
        write!(f, "{hours:02}:{minutes:02}:{seconds:02}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActorId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScenarioId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalKind {
    Input,
    Stdout,
    Stderr,
    System,
    Success,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalLine {
    pub kind: TerminalKind,
    pub text: String,
}

impl TerminalLine {
    #[must_use]
    pub fn stdout(text: impl Into<String>) -> Self {
        Self { kind: TerminalKind::Stdout, text: text.into() }
    }

    #[must_use]
    pub fn stderr(text: impl Into<String>) -> Self {
        Self { kind: TerminalKind::Stderr, text: text.into() }
    }

    #[must_use]
    pub fn input(text: impl Into<String>) -> Self {
        Self { kind: TerminalKind::Input, text: text.into() }
    }

    #[must_use]
    pub fn system(text: impl Into<String>) -> Self {
        Self { kind: TerminalKind::System, text: text.into() }
    }
}

/// Requests from the UI (or harness) to the authoritative simulation session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SimRequest {
    Initialize { scenario_id: String, seed: u64 },
    ExecuteCommand { command: String },
    AdvanceClock { delta_ms: u64 },
    SetClockSpeed { multiplier: u32 },
    Pause,
    Resume,
    Reset { scenario_id: String, seed: u64 },
    Snapshot,
    CancelJob { job_id: u64 },
    /// Reveal the next progressive lab hint (recorded separately from correctness).
    UseHint,
    /// Read a virtual-filesystem text file (editor / log inspection).
    ReadVfs { path: String },
    /// Write a virtual-filesystem text file (never host paths).
    WriteVfs { path: String, content: String },
}

/// Responses from the simulation session to the UI.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SimResponse {
    Ready {
        protocol_version: String,
        compatibility_version: String,
        seq: u64,
        state: UiWorldView,
    },
    CommandResult {
        seq: u64,
        prompt: String,
        lines: Vec<TerminalLine>,
        state: UiWorldView,
    },
    State {
        seq: u64,
        state: UiWorldView,
    },
    Error {
        code: String,
        message: String,
        seq: u64,
    },
    FileContent {
        seq: u64,
        path: String,
        content: String,
    },
}

/// UI-facing authoritative snapshot. Derived only from the simulation worker.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiWorldView {
    pub scenario_id: String,
    pub seed: u64,
    pub now_ms: u64,
    pub paused: bool,
    pub clock_multiplier: u32,
    pub state_digest: String,
    pub prompt: String,
    pub gpus: Vec<UiGpuTile>,
    pub jobs: Vec<UiJobSummary>,
    pub node_status: String,
    pub lab_steps: Vec<UiLabStep>,
    pub hint_level: u8,
    pub hint_text: Option<String>,
    pub lab_complete: bool,
    pub practical_percent: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiLabStep {
    pub id: String,
    pub label: String,
    pub complete: bool,
    pub critical: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiGpuTile {
    pub index: u16,
    pub model: String,
    pub status: String,
    pub owner_job_id: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiJobSummary {
    pub id: u64,
    pub name: String,
    pub user: String,
    pub status: String,
    pub pending_reason: Option<String>,
    /// Curriculum-safe explanation when the job is pending.
    pub pending_explanation: Option<String>,
    pub gpus: u16,
    pub cpus: u32,
    pub memory_mib: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("unsupported protocol version: {0}")]
    UnsupportedProtocol(String),
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulated_time_formats_without_wall_clock() {
        assert_eq!(SimTimeMs(3_661_000).to_string(), "01:01:01");
    }

    #[test]
    fn worker_message_round_trips() {
        let request = SimRequest::ExecuteCommand { command: "sinfo".into() };
        let json = serde_json::to_string(&request).expect("serialize");
        let decoded: SimRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(request, decoded);
    }

    #[test]
    fn response_round_trips_with_state() {
        let response = SimResponse::State {
            seq: 3,
            state: UiWorldView {
                scenario_id: "dgx-h200-8".into(),
                seed: 1,
                now_ms: 0,
                paused: false,
                clock_multiplier: 1,
                state_digest: "abc".into(),
                prompt: "learner@dgx-login-01:~$".into(),
                gpus: vec![UiGpuTile {
                    index: 0,
                    model: "H200".into(),
                    status: "Idle".into(),
                    owner_job_id: None,
                }],
                jobs: vec![],
                node_status: "idle".into(),
                lab_steps: vec![],
                hint_level: 0,
                hint_text: None,
                lab_complete: false,
                practical_percent: 0,
            },
        };
        let json = serde_json::to_string(&response).expect("serialize");
        let decoded: SimResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(response, decoded);
    }
}
