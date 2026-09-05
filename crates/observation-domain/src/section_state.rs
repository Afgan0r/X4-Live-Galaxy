#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SectionQuality {
    Fresh,
    KnownEmpty,
    Unknown,
    Partial,
    Stale,
    Unsupported,
}
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SectionFreshness {
    Fresh,
    Stale,
}
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SectionCoverage {
    Complete,
    KnownEmpty,
    Unknown,
    Partial,
    Unsupported,
}
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SectionAvailability {
    Available,
    Unavailable,
}
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CaptureWindow {
    start_millis: u64,
    end_millis: u64,
}
impl CaptureWindow {
    #[must_use]
    pub const fn new(start_millis: u64, end_millis: u64) -> Option<Self> {
        if start_millis <= end_millis {
            Some(Self {
                start_millis,
                end_millis,
            })
        } else {
            None
        }
    }
    #[must_use]
    pub const fn start_millis(self) -> u64 {
        self.start_millis
    }
    #[must_use]
    pub const fn end_millis(self) -> u64 {
        self.end_millis
    }
}
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SectionState {
    capture_window: CaptureWindow,
    freshness: SectionFreshness,
    quality: SectionQuality,
    availability: SectionAvailability,
    coverage: SectionCoverage,
}
impl SectionState {
    pub const fn new(freshness: SectionFreshness, coverage: SectionCoverage) -> Self {
        Self {
            capture_window: CaptureWindow {
                start_millis: 0,
                end_millis: 0,
            },
            freshness,
            quality: SectionQuality::Unknown,
            availability: SectionAvailability::Available,
            coverage,
        }
    }
    pub const fn with_evidence(
        capture_window: CaptureWindow,
        freshness: SectionFreshness,
        quality: SectionQuality,
        availability: SectionAvailability,
        coverage: SectionCoverage,
    ) -> Self {
        Self {
            capture_window,
            freshness,
            quality,
            availability,
            coverage,
        }
    }
    pub const fn capture_window(self) -> CaptureWindow {
        self.capture_window
    }
    pub const fn freshness(self) -> SectionFreshness {
        self.freshness
    }
    pub const fn coverage(self) -> SectionCoverage {
        self.coverage
    }
    pub const fn quality(self) -> SectionQuality {
        self.quality
    }
    pub const fn availability(self) -> SectionAvailability {
        self.availability
    }
}
