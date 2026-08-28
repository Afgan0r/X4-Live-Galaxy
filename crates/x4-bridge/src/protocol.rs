const PROTOCOL_CAPABILITY: &str = "live-galaxy-observation-v1";
const GAME_FACING_BUILD: &str = "live-galaxy-x4-build-1";

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityDecision {
    Compatible,
    RestartRequired,
}

impl CapabilityDecision {
    pub fn negotiate(capability: &str) -> Self {
        if capability == PROTOCOL_CAPABILITY {
            Self::Compatible
        } else {
            Self::RestartRequired
        }
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionHello {
    protocol_major: u16,
    game_build: String,
    capabilities: Vec<String>,
}

impl SessionHello {
    pub fn new<I, S>(protocol_major: u16, game_build: impl Into<String>, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            protocol_major,
            game_build: game_build.into(),
            capabilities: capabilities.into_iter().map(Into::into).collect(),
        }
    }

    pub(crate) fn restart_requirement(
        self,
        expected_protocol_major: u16,
    ) -> Option<RestartRequirement> {
        if self.protocol_major != expected_protocol_major {
            return Some(RestartRequirement::ProtocolMajorMismatch);
        }
        if self.game_build != GAME_FACING_BUILD {
            return Some(RestartRequirement::GameBuildMismatch);
        }
        if !self
            .capabilities
            .iter()
            .any(|capability| capability == PROTOCOL_CAPABILITY)
        {
            return Some(RestartRequirement::MissingRequiredCapability);
        }
        None
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestartRequirement {
    ProtocolMajorMismatch,
    MissingRequiredCapability,
    GameBuildMismatch,
}
