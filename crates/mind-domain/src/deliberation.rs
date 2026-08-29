use serde::{Deserialize, Serialize};
use strategic_state::{Capability, Faction, StrategicPacket};

const MAX_TEXT: usize = 128;
const MAX_FACTS: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliberationRequest {
    pub(crate) packet: StrategicPacket,
    pub(crate) snapshot_identity: String,
    pub(crate) visible_fact_ids: Vec<String>,
    pub(crate) allowed_capabilities: Vec<Capability>,
    pub(crate) policy_version: String,
    pub(crate) prompt_package_hash: String,
    pub(crate) max_candidate_bytes: usize,
    pub(crate) max_trade_offs: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestError {
    Invalid,
    UnsupportedFaction,
}

impl DeliberationRequest {
    #[expect(
        clippy::too_many_arguments,
        reason = "the frozen request constructor mirrors the versioned boundary"
    )]
    pub fn from_packet<I, F, J, C>(
        packet: StrategicPacket,
        snapshot_identity: &str,
        fact_ids: I,
        capabilities: J,
        policy_version: &str,
        prompt_package_hash: &str,
        max_candidate_bytes: usize,
        max_trade_offs: usize,
    ) -> Result<Self, RequestError>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<str>,
        J: IntoIterator<Item = C>,
        C: std::borrow::Borrow<Capability>,
    {
        if !matches!(packet.faction(), Faction::Zya | Faction::Arg)
            || !valid(snapshot_identity)
            || !valid(policy_version)
            || !valid(prompt_package_hash)
            || max_candidate_bytes == 0
            || max_trade_offs == 0
        {
            return Err(RequestError::UnsupportedFaction);
        }
        let mut visible_fact_ids: Vec<String> = fact_ids
            .into_iter()
            .map(|fact| fact.as_ref().into())
            .collect();
        visible_fact_ids.sort_unstable();
        visible_fact_ids.dedup();
        let allowed_capabilities: Vec<Capability> = capabilities
            .into_iter()
            .map(|capability| *capability.borrow())
            .collect();
        if visible_fact_ids.is_empty()
            || visible_fact_ids.len() > MAX_FACTS
            || visible_fact_ids.iter().any(|fact| !valid(fact))
            || allowed_capabilities.is_empty()
        {
            return Err(RequestError::Invalid);
        }
        Ok(Self {
            packet,
            snapshot_identity: snapshot_identity.into(),
            visible_fact_ids,
            allowed_capabilities,
            policy_version: policy_version.into(),
            prompt_package_hash: prompt_package_hash.into(),
            max_candidate_bytes,
            max_trade_offs,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShadowProposal {
    pub(crate) schema_version: String,
    pub(crate) capability: Capability,
    pub(crate) priority: u8,
    pub(crate) horizon: String,
    pub(crate) supporting_fact_ids: Vec<String>,
    pub(crate) trade_offs: Vec<String>,
    pub(crate) explanation: String,
    pub(crate) command_id: String,
}

impl ShadowProposal {
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "the strict fixture constructor mirrors the proposal schema"
    )]
    pub fn new<I, F, J, T>(
        schema_version: &str,
        capability: Capability,
        priority: u8,
        horizon: &str,
        facts: I,
        trade_offs: J,
        explanation: &str,
        command_id: &str,
    ) -> Self
    where
        I: IntoIterator<Item = F>,
        F: AsRef<str>,
        J: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        Self {
            schema_version: schema_version.into(),
            capability,
            priority,
            horizon: horizon.into(),
            supporting_fact_ids: facts.into_iter().map(|fact| fact.as_ref().into()).collect(),
            trade_offs: trade_offs
                .into_iter()
                .map(|item| item.as_ref().into())
                .collect(),
            explanation: explanation.into(),
            command_id: command_id.into(),
        }
    }
}

pub const fn valid(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_TEXT
}
