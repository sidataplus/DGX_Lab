#![forbid(unsafe_code)]

//! Declarative virtual-user and infrastructure actor contracts.

use dgxlab_contracts::{ActorId, JobId, SimTimeMs};
use serde::{Deserialize, Serialize};
use slurm_model::JobSpec;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActorKind {
    Scripted,
    BackgroundLoad { target_gpu_occupancy_percent: u8 },
    PolicyDriven { resubmit_after_failure: bool },
    Infrastructure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorDefinition {
    pub id: ActorId,
    pub username: String,
    pub kind: ActorKind,
    pub actions: Vec<TimedActorAction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedActorAction {
    pub at: SimTimeMs,
    pub action: ActorAction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActorAction {
    SubmitJob { spec: Box<JobSpec> },
    CancelJob { job_id: JobId },
    DrainNode { node_id: String, reason: String },
    ResumeNode { node_id: String },
    InjectGpuWarning { node_id: String, gpu_index: u16 },
    RestoreGpu { node_id: String, gpu_index: u16 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_actions_are_data_not_callbacks() {
        let action =
            ActorAction::DrainNode { node_id: "dgx-h200-01".into(), reason: "maintenance".into() };
        let encoded = serde_json::to_string(&action).unwrap();
        assert!(encoded.contains("drain_node"));
    }
}
