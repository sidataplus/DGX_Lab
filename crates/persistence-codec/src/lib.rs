#![forbid(unsafe_code)]

//! Canonical session serialization and integrity checks.

use dgxlab_contracts::{SESSION_FORMAT_VERSION, SessionId};
use grading::EvidenceLedger;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sim_core::SimulationWorld;
use virtual_shell::ShellSession;

const MAGIC: &[u8] = b"DGXLAB-SESSION\n";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionBundle {
    pub schema: String,
    pub format_version: String,
    pub session_id: SessionId,
    pub app_version: String,
    pub scenario_id: String,
    pub scenario_revision: String,
    pub seed: u64,
    pub world: SimulationWorld,
    pub shell: ShellSession,
    pub evidence: EvidenceLedger,
    pub content_digest: String,
}

impl SessionBundle {
    #[must_use]
    pub fn new(
        session_id: SessionId,
        app_version: impl Into<String>,
        scenario_revision: impl Into<String>,
        world: SimulationWorld,
        shell: ShellSession,
        evidence: EvidenceLedger,
    ) -> Self {
        let mut bundle = Self {
            schema: "dgxlab.session/v1".into(),
            format_version: SESSION_FORMAT_VERSION.into(),
            session_id,
            app_version: app_version.into(),
            scenario_id: world.scenario_id.clone(),
            scenario_revision: scenario_revision.into(),
            seed: world.seed,
            world,
            shell,
            evidence,
            content_digest: String::new(),
        };
        bundle.content_digest = bundle.payload_digest();
        bundle
    }

    #[must_use]
    pub fn payload_digest(&self) -> String {
        let mut clone = self.clone();
        clone.content_digest.clear();
        let bytes = serde_json::to_vec(&clone).expect("session structure is serializable");
        hex::encode(Sha256::digest(bytes))
    }

    #[must_use]
    pub fn integrity_ok(&self) -> bool {
        self.content_digest == self.payload_digest()
    }
}

pub fn encode_session(bundle: &SessionBundle) -> Result<Vec<u8>, CodecError> {
    if !bundle.integrity_ok() {
        return Err(CodecError::DigestMismatch);
    }
    let json = serde_json::to_vec(bundle)?;
    let mut bytes = Vec::with_capacity(MAGIC.len() + json.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&json);
    Ok(bytes)
}

pub fn decode_session(bytes: &[u8]) -> Result<SessionBundle, CodecError> {
    let payload = bytes.strip_prefix(MAGIC).ok_or(CodecError::InvalidMagic)?;
    let bundle: SessionBundle = serde_json::from_slice(payload)?;
    if bundle.schema != "dgxlab.session/v1" {
        return Err(CodecError::UnsupportedSchema(bundle.schema));
    }
    if !bundle.integrity_ok() {
        return Err(CodecError::DigestMismatch);
    }
    Ok(bundle)
}

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("invalid DGX Lab session magic bytes")]
    InvalidMagic,
    #[error("unsupported session schema: {0}")]
    UnsupportedSchema(String),
    #[error("session digest mismatch")]
    DigestMismatch,
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use dgxlab_contracts::SessionId;

    #[test]
    fn round_trip_preserves_world_digest() {
        let world = SimulationWorld::dgx_h200_8(42);
        let expected = world.state_digest();
        let bundle = SessionBundle::new(
            SessionId("test".into()),
            "0.1.0",
            "1.0.0",
            world,
            ShellSession::learner(),
            EvidenceLedger::new(),
        );
        let bytes = encode_session(&bundle).unwrap();
        let decoded = decode_session(&bytes).unwrap();
        assert_eq!(decoded.world.state_digest(), expected);
    }

    #[test]
    fn tampering_is_rejected() {
        let bundle = SessionBundle::new(
            SessionId("test".into()),
            "0.1.0",
            "1.0.0",
            SimulationWorld::dgx_h200_8(42),
            ShellSession::learner(),
            EvidenceLedger::new(),
        );
        let mut bytes = encode_session(&bundle).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        assert!(decode_session(&bytes).is_err());
    }
}
