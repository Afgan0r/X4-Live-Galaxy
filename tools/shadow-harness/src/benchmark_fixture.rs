use mind_domain::DeliberationRequest;
use mind_orchestration::{ProviderMetadata, ProviderRequest};
use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Capability, Faction, PacketLimits, derive_packets};

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenchmarkFixture {
    pub(crate) id: String,
    track: String,
    faction: String,
    frozen_snapshot_identity: String,
    current_snapshot_identity: String,
    visible_fact_ids: Vec<String>,
    allowed_capabilities: Vec<String>,
    policy_version: String,
    prompt_package_hash: String,
    provider_id: String,
    model_id: String,
    generation_settings: String,
    prompt_payload: String,
    expected_trajectory: String,
    expected_disposition: String,
    observation_identity: u64,
    max_candidate_bytes: usize,
    max_trade_offs: usize,
    frames: Vec<String>,
}

impl BenchmarkFixture {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, ()> {
        let fixture = serde_json::from_slice::<Self>(bytes).map_err(|_| ())?;
        fixture.validate()?;
        Ok(fixture)
    }

    pub(crate) fn request(&self) -> Result<ProviderRequest, ()> {
        let faction = faction(&self.faction)?;
        let frames: Vec<_> = self.frames.iter().map(String::as_str).collect();
        let snapshot = admit_batch(AcceptedProjection::empty(), &frames)
            .into_projection()
            .snapshot;
        let packet = derive_packets(&snapshot, PacketLimits::tracer())
            .map_err(|_| ())?
            .packet(faction)
            .clone();
        let capabilities: Result<Vec<_>, _> = self
            .allowed_capabilities
            .iter()
            .map(|value| capability(value))
            .collect();
        let request = DeliberationRequest::from_packet(
            packet,
            &self.frozen_snapshot_identity,
            &self.visible_fact_ids,
            capabilities?,
            &self.policy_version,
            &self.prompt_package_hash,
            self.max_candidate_bytes,
            self.max_trade_offs,
        )
        .map_err(|_| ())?;
        let metadata = ProviderMetadata::new(&self.provider_id, &self.model_id).map_err(|_| ())?;
        ProviderRequest::new(&self.id, self.observation_identity, request, metadata).map_err(|_| ())
    }

    pub(crate) fn faction(&self) -> Result<Faction, ()> {
        faction(&self.faction)
    }

    pub(crate) fn current_snapshot_identity(&self) -> &str {
        &self.current_snapshot_identity
    }

    pub(crate) fn canonical_payload(&self) -> String {
        serde_json::json!({
            "allowed_capabilities": self.allowed_capabilities,
            "current_snapshot_identity": self.current_snapshot_identity,
            "expected_disposition": self.expected_disposition,
            "expected_trajectory": self.expected_trajectory,
            "faction": self.faction,
            "frozen_snapshot_identity": self.frozen_snapshot_identity,
            "generation_settings": self.generation_settings,
            "id": self.id,
            "model_id": self.model_id,
            "observation_identity": self.observation_identity,
            "policy_version": self.policy_version,
            "prompt_package_hash": self.prompt_package_hash,
            "prompt_payload": self.prompt_payload,
            "provider_id": self.provider_id,
            "visible_fact_ids": self.visible_fact_ids,
        })
        .to_string()
    }

    fn validate(&self) -> Result<(), ()> {
        let fields = [
            &self.id,
            &self.track,
            &self.faction,
            &self.frozen_snapshot_identity,
            &self.current_snapshot_identity,
            &self.policy_version,
            &self.prompt_package_hash,
            &self.provider_id,
            &self.model_id,
            &self.generation_settings,
            &self.prompt_payload,
            &self.expected_trajectory,
            &self.expected_disposition,
        ];
        if self.track != "benchmark"
            || self.frozen_snapshot_identity != self.current_snapshot_identity
            || self.observation_identity == 0
            || self.max_candidate_bytes == 0
            || self.max_trade_offs == 0
            || self.frames.is_empty()
            || self.visible_fact_ids.is_empty()
            || self.allowed_capabilities.is_empty()
            || fields
                .iter()
                .any(|value| value.is_empty() || value.len() > 128)
            || self
                .visible_fact_ids
                .iter()
                .any(|value| value.is_empty() || value.len() > 128)
            || self
                .allowed_capabilities
                .iter()
                .any(|value| capability(value).is_err())
            || faction(&self.faction).is_err()
            || !matches!(self.expected_disposition.as_str(), "accept" | "reject")
            || !matches!(
                self.expected_trajectory.as_str(),
                "zero-cycle" | "direct-agreement"
            )
        {
            return Err(());
        }
        Ok(())
    }
}

fn faction(value: &str) -> Result<Faction, ()> {
    match value {
        "ZYA" => Ok(Faction::Zya),
        "ARG" => Ok(Faction::Arg),
        _ => Err(()),
    }
}

fn capability(value: &str) -> Result<Capability, ()> {
    match value {
        "DefenseAndMilitaryStrategy" => Ok(Capability::DefenseAndMilitaryStrategy),
        "EconomyAndLogistics" => Ok(Capability::EconomyAndLogistics),
        "TerritorialDevelopmentAndInfrastructure" => {
            Ok(Capability::TerritorialDevelopmentAndInfrastructure)
        }
        _ => Err(()),
    }
}
