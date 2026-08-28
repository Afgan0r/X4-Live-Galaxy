#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VisibilityPolicy {
    version: &'static str,
}

impl VisibilityPolicy {
    pub const fn v1() -> Self {
        Self {
            version: "visibility-v1",
        }
    }

    #[must_use]
    pub const fn version(self) -> &'static str {
        self.version
    }
}
