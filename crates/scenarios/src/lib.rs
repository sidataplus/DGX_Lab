#![forbid(unsafe_code)]

//! Scenario contracts and built-in scenario initializers.

use actors::ActorAction;
use dgxlab_contracts::SimTimeMs;
use serde::{Deserialize, Serialize};
use sim_core::{SimError, SimulationWorld};
use slurm_model::{JobSpec, Tres};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioDefinition {
    pub schema: String,
    pub id: String,
    pub revision: String,
    pub title: String,
    pub cluster_profile: String,
    pub learner: LearnerDefinition,
    #[serde(default)]
    pub initial_files: Vec<InitialFile>,
    #[serde(default)]
    pub actors: Vec<ScenarioActor>,
    #[serde(default)]
    pub objectives: Vec<ObjectiveDefinition>,
    #[serde(default)]
    pub hints: Vec<HintDefinition>,
    #[serde(default)]
    pub checks: Vec<CheckDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnerDefinition {
    pub username: String,
    pub account: String,
    pub qos: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitialFile {
    pub path: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioActor {
    pub id: String,
    pub kind: String,
    pub username: String,
    #[serde(default)]
    pub actions: Vec<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectiveDefinition {
    pub id: String,
    pub competency: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HintDefinition {
    pub level: u8,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckDefinition {
    pub id: String,
    #[serde(default)]
    pub critical: bool,
    pub points: u32,
    pub assert: serde_json::Value,
}

pub fn initialize_scenario(id: &str, seed: u64) -> Result<SimulationWorld, ScenarioError> {
    match id {
        "dgx-h200-8" | "guided-one-gpu" => Ok(SimulationWorld::dgx_h200_8(seed)),
        "dgx-contended" | "pending-gpu-contention-01" => contended(seed),
        "dgx-degraded" | "failure-resume-01" => degraded(seed),
        "dgx-shared" => shared(seed),
        other => Err(ScenarioError::UnknownScenario(other.into())),
    }
}

fn contended(seed: u64) -> Result<SimulationWorld, ScenarioError> {
    let mut world = SimulationWorld::dgx_h200_8(seed);
    world.scenario_id = "dgx-contended".into();
    world.apply_actor_action(ActorAction::SubmitJob {
        spec: Box::new(background_job("vision-train", "alice", 4, 256, 8)),
    })?;
    world.apply_actor_action(ActorAction::SubmitJob {
        spec: Box::new(background_job("language-train", "bob", 4, 512, 5)),
    })?;
    Ok(world)
}

fn degraded(seed: u64) -> Result<SimulationWorld, ScenarioError> {
    let mut world = SimulationWorld::dgx_h200_8(seed);
    world.scenario_id = "dgx-degraded".into();
    world.apply_actor_action(ActorAction::InjectGpuWarning {
        node_id: "dgx-h200-01".into(),
        gpu_index: 2,
    })?;
    let failed = JobSpec {
        name: "train-llm".into(),
        resources: Tres { cpus: 16, memory_mib: 32 * 1024, gpu_type: Some("h200".into()), gpus: 4 },
        command: "python train.py --batch-size 64 --epochs 5".into(),
        workload_id: "checkpoint-resume-v1".into(),
        time_limit_ms: 2 * 60 * 60 * 1_000,
        ..JobSpec::default()
    };
    world.apply_actor_action(ActorAction::SubmitJob { spec: Box::new(failed) })?;
    world.advance_to(SimTimeMs(90_000))?;
    Ok(world)
}

fn shared(seed: u64) -> Result<SimulationWorld, ScenarioError> {
    let mut world = SimulationWorld::dgx_h200_8(seed);
    world.scenario_id = "dgx-shared".into();
    world.apply_actor_action(ActorAction::SubmitJob {
        spec: Box::new(background_job("learner-history", "learner", 1, 96, 2)),
    })?;
    world.advance_to(SimTimeMs(60_000))?;

    let normal = world
        .cluster
        .qos
        .get_mut("normal")
        .expect("the built-in DGX profile defines the normal QOS");
    normal.max_running_jobs_per_user = Some(1);
    normal.max_gpus_per_user = Some(4);

    world.apply_actor_action(ActorAction::SubmitJob {
        spec: Box::new(background_job("learner-baseline", "learner", 1, 64, 4)),
    })?;
    world.apply_actor_action(ActorAction::SubmitJob {
        spec: Box::new(background_job("learner-followup", "learner", 1, 64, 4)),
    })?;
    Ok(world)
}

fn background_job(name: &str, user: &str, gpus: u16, memory_gib: u64, epochs: u32) -> JobSpec {
    JobSpec {
        name: name.into(),
        user: user.into(),
        resources: Tres {
            cpus: 8 * u32::from(gpus),
            memory_mib: memory_gib * 1024,
            gpu_type: Some("h200".into()),
            gpus,
        },
        command: format!("python train.py --batch-size 64 --epochs {epochs}"),
        workload_id: "pytorch-training-v1".into(),
        time_limit_ms: 4 * 60 * 60 * 1_000,
        ..JobSpec::default()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ScenarioError {
    #[error("unknown scenario: {0}")]
    UnknownScenario(String),
    #[error(transparent)]
    Simulation(#[from] SimError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use slurm_model::{GpuHealth, JobStatus, PendingReason};

    #[test]
    fn contended_profile_occupies_all_gpus() {
        let world = initialize_scenario("dgx-contended", 1).unwrap();
        assert_eq!(world.jobs.values().filter(|job| job.status == JobStatus::Running).count(), 2);
        assert_eq!(world.cluster.nodes["dgx-h200-01"].allocated.gpus, 8);
    }

    #[test]
    fn degraded_profile_contains_warning_and_failure_history() {
        let world = initialize_scenario("dgx-degraded", 1).unwrap();
        assert_eq!(world.cluster.nodes["dgx-h200-01"].gpus[2].health, GpuHealth::Warning);
        assert!(world.jobs.values().any(|job| job.status == JobStatus::OutOfMemory));
    }

    #[test]
    fn shared_profile_exposes_a_qos_limited_learner_job() {
        let world = initialize_scenario("dgx-shared", 1).unwrap();
        let learner_jobs =
            world.jobs.values().filter(|job| job.spec.user == "learner").collect::<Vec<_>>();

        assert_eq!(learner_jobs.len(), 3);
        assert!(learner_jobs.iter().any(|job| job.status == JobStatus::Running));
        assert!(learner_jobs.iter().any(|job| {
            job.status == JobStatus::Pending
                && job.pending_reason == PendingReason::QosMaxJobsPerUserLimit
        }));
        assert!(
            world
                .accounting
                .values()
                .any(|record| { record.user == "learner" && record.state == JobStatus::Completed })
        );
    }
}
