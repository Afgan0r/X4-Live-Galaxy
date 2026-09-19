use observation_domain::{SectionAvailability, SectionCoverage, SectionFreshness, SectionQuality};

const FORMAT_VERSION: u64 = 2;

pub fn parse_completion_counts(
    format: u64,
    fields: &mut std::str::Split<'_, char>,
) -> Option<(Option<usize>, Option<usize>)> {
    let counts = if format == 1 {
        (next_u64(fields)? == 0).then_some((None, None))?
    } else if format == FORMAT_VERSION {
        (
            Some(usize::try_from(next_u64(fields)?).ok()?),
            Some(usize::try_from(next_u64(fields)?).ok()?),
        )
    } else {
        return None;
    };
    fields.next().is_none().then_some(counts)
}

pub fn next_u64(fields: &mut std::str::Split<'_, char>) -> Option<u64> {
    fields.next()?.parse().ok()
}

pub const fn freshness_code(value: SectionFreshness) -> u8 {
    match value {
        SectionFreshness::Fresh => 1,
        SectionFreshness::Stale => 2,
    }
}
pub const fn parse_freshness(value: u64) -> Option<SectionFreshness> {
    match value {
        1 => Some(SectionFreshness::Fresh),
        2 => Some(SectionFreshness::Stale),
        _ => None,
    }
}
pub const fn availability_code(value: SectionAvailability) -> u8 {
    match value {
        SectionAvailability::Available => 1,
        SectionAvailability::Unavailable => 2,
    }
}
pub const fn parse_availability(value: u64) -> Option<SectionAvailability> {
    match value {
        1 => Some(SectionAvailability::Available),
        2 => Some(SectionAvailability::Unavailable),
        _ => None,
    }
}
pub const fn quality_code(value: SectionQuality) -> u8 {
    match value {
        SectionQuality::Fresh => 1,
        SectionQuality::KnownEmpty => 2,
        SectionQuality::Unknown => 3,
        SectionQuality::Partial => 4,
        SectionQuality::Stale => 5,
        SectionQuality::Unsupported => 6,
    }
}
pub const fn parse_quality(value: u64) -> Option<SectionQuality> {
    match value {
        1 => Some(SectionQuality::Fresh),
        2 => Some(SectionQuality::KnownEmpty),
        3 => Some(SectionQuality::Unknown),
        4 => Some(SectionQuality::Partial),
        5 => Some(SectionQuality::Stale),
        6 => Some(SectionQuality::Unsupported),
        _ => None,
    }
}
pub const fn coverage_code(value: SectionCoverage) -> u8 {
    match value {
        SectionCoverage::Complete => 1,
        SectionCoverage::KnownEmpty => 2,
        SectionCoverage::Unknown => 3,
        SectionCoverage::Partial => 4,
        SectionCoverage::Unsupported => 5,
        SectionCoverage::PointMeasurement => 6,
    }
}
pub const fn parse_coverage(value: u64) -> Option<SectionCoverage> {
    match value {
        1 => Some(SectionCoverage::Complete),
        2 => Some(SectionCoverage::KnownEmpty),
        3 => Some(SectionCoverage::Unknown),
        4 => Some(SectionCoverage::Partial),
        5 => Some(SectionCoverage::Unsupported),
        6 => Some(SectionCoverage::PointMeasurement),
        _ => None,
    }
}
