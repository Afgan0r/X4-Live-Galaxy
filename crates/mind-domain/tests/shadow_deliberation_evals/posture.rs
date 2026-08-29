use mind_domain::{
    PostureCandidate, PostureEffect, PostureRejection, ShadowPosture, admit_posture,
};

#[test]
fn sd_010_admits_all_closed_shadow_postures_from_frozen_visible_facts() {
    let visible = vec!["ZYA:military:fleet".into(), "XEN:threat:XEN".into()];
    for posture in [
        ShadowPosture::Maintain,
        ShadowPosture::DeEscalate,
        ShadowPosture::Intensify,
        ShadowPosture::LimitedThreatDrivenCoordination,
    ] {
        let fact_ids = if posture == ShadowPosture::LimitedThreatDrivenCoordination {
            vec!["XEN:threat:XEN".into()]
        } else {
            vec!["ZYA:military:fleet".into()]
        };
        let candidate = PostureCandidate {
            posture,
            fact_ids,
            effect: PostureEffect::ShadowOnly,
        };
        assert_eq!(admit_posture(&candidate, &visible), Ok(posture));
    }
}

#[test]
fn sd_010_rejects_hidden_coordination_and_all_external_posture_effects() {
    let visible = vec!["XEN:threat:XEN".into()];
    let mut candidate = PostureCandidate {
        posture: ShadowPosture::LimitedThreatDrivenCoordination,
        fact_ids: vec!["ARG:hidden".into()],
        effect: PostureEffect::ShadowOnly,
    };
    assert_eq!(
        admit_posture(&candidate, &visible),
        Err(PostureRejection::Information)
    );
    candidate.fact_ids = vec!["XEN:threat:XEN".into()];
    for effect in [
        PostureEffect::Negotiation,
        PostureEffect::X4Command,
        PostureEffect::ReportIntent,
        PostureEffect::RelationshipChange,
    ] {
        candidate.effect = effect;
        assert_eq!(
            admit_posture(&candidate, &visible),
            Err(PostureRejection::ExternalEffect)
        );
    }
}

#[test]
fn sd_010_rejects_unsupported_or_noncanonical_posture_evidence_without_effect() {
    let unsupported = serde_json::from_str::<ShadowPosture>("\"negotiate\"");
    assert!(unsupported.is_err());
    let visible = vec!["XEN:threat:XEN".into(), "ZYA:military:fleet".into()];
    let candidate = PostureCandidate {
        posture: ShadowPosture::Maintain,
        fact_ids: vec!["ZYA:military:fleet".into(), "XEN:threat:XEN".into()],
        effect: PostureEffect::ShadowOnly,
    };
    assert_eq!(
        admit_posture(&candidate, &visible),
        Err(PostureRejection::Information)
    );
}
