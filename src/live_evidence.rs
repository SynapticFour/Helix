// SPDX-License-Identifier: Apache-2.0
//! Live independent observation vs mock / catalog masquerade.
//!
//! Operator `--implementation-name` is not live evidence. Helix still cannot
//! cryptographically bind a URL to a Docker digest or git commit. This module
//! rejects the cheap forgeries (mock kind, default catalog fixture, default
//! object id). It does not rank implementations. Not HELIOS. Not certification.

use crate::fixture::{FixtureSource, DEFAULT_DRS_OBJECT_ID};
use crate::independence::run_counts_as_independent;
use crate::model::VerificationRun;
use crate::target::TargetKind;

/// B11/B12 pinned DRS 1.4.0 identities. If any computed value differs, B12 is
/// blocked: do not silently regenerate.
pub const PINNED_HELIXTEST_SHA: &str = "1baddfd3d75f01dc7c149074a785616fa014c725";
pub const PINNED_CHECKER_SOURCE_SHA256: &str =
    "18bf4a445ac5cf7ae9a45a331834dc13da3a21528f5b29eb1a72bddfbc42a05a";
pub const PINNED_CHECKER_ID: &str =
    "helixtest-drs:18bf4a445ac5cf7ae9a45a331834dc13da3a21528f5b29eb1a72bddfbc42a05a";
pub const PINNED_RELEASE_COMMIT: &str = "36145d389e0a454428d1dac5c4a30870995fdd7c";
pub const PINNED_PACK_INTEGRITY_SHA256: &str =
    "c3836145e57a62350704e3a67868b80422c54eaca592c33f80fd6b565ac3fc89";
pub const PINNED_SCHEMA_DOCUMENT_SHA256: &str =
    "3d8de69f8ef37e3548b90286b3ae108697ce6afec543e774605dc3f50282c608";
pub const PINNED_SCHEMA_COMPONENT_SHA256: &str =
    "b27ef7640eb43fbd20dd1a4a3b6044a1a7d966f92a252ebcbd88959b1a373003";
pub const PINNED_BINDING_ID: &str =
    "72da037c4ce2383f116bf195507fe1b45c60d6917acf5a87c6e5bba7043c69e2";
pub const PINNED_CATALOG_ID: &str =
    "03ce38f690ed679ff967e636bb037e0ed4ebe42f2cbde62766921ad7eae96ac1";
pub const PINNED_COVERAGE_ID: &str =
    "082f63c9eec7472a66f8121a66e28ffb7f68680791f7bb5ea5ef47441d18f08c";
pub const PINNED_EXECUTION_ID: &str =
    "ee00e1a49e6b3f7d47314bde77738faf4b6cec4d3325dfe56d139809ea97037e";

pub const STARTER_KIT_TARGET_ID: &str = "ga4gh-starter-kit-drs-0.3.2";
pub const BENTO_TARGET_ID: &str = "bento-drs-0.21.5";
pub const STARTER_KIT_OBJECT_ID: &str = "b8cd0667-2c33-4c9f-967b-161b905932c9";
pub const BENTO_OBJECT_ID: &str = "f23b1635-4a65-40fa-8b29-75b4734b602a";
pub const BENTO_OBJECT_SHA256: &str =
    "6896d9ea3f73a4434f5832bc65714e7d066f177373f36f34dc8a6f735daa41b1";
pub const STARTER_KIT_IMAGE: &str = "ga4gh/ga4gh-starter-kit-drs:0.3.2";
pub const STARTER_KIT_DIGEST: &str =
    "sha256:e680096c0f7406f51fceca3812e3e68f5c7c701bc20f2f0fd07bb85fe972b4b1";
pub const BENTO_SOURCE_COMMIT: &str = "1dc55ebea90185b1fec2c78c8c52909dd0ca889e";

/// True only when the run is a reviewed independent implementation observed
/// through an operator-declared fixture that is not the in-process catalog.
///
/// This is **not** a cryptographic bind of the process behind the URL to the
/// reviewed Docker digest or git commit. It rejects mock/catalog masquerade.
pub fn live_independent_observation(run: &VerificationRun) -> bool {
    if !run_counts_as_independent(run) {
        return false;
    }
    let Some(identity) = run.target.identity.as_ref() else {
        return false;
    };
    if identity.target_kind.is_mock_or_fixture_or_synthetic() {
        return false;
    }
    if !matches!(
        identity.target_kind,
        TargetKind::RealIndependentLocalImplementation | TargetKind::RealExternalImplementation
    ) {
        return false;
    }
    if identity.endpoint.is_empty() || identity.endpoint.contains("://user:") {
        return false;
    }
    let Some(fx) = run.drs_fixture.as_ref() else {
        return false;
    };
    if fx.source != FixtureSource::OperatorDeclared {
        return false;
    }
    if fx.object_id == DEFAULT_DRS_OBJECT_ID {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::DrsVerifyFixture;
    use crate::model::Target;
    use crate::target::{DeclaredTarget, TargetIdentity};

    fn run_labeled(
        id: &str,
        kind: TargetKind,
        name: &str,
        fx: DrsVerifyFixture,
    ) -> VerificationRun {
        let mut run = VerificationRun::new(Target::from_identity(TargetIdentity::from_declared(
            "http://127.0.0.1:9",
            &DeclaredTarget {
                target_id: Some(id.into()),
                kind,
                implementation_name: Some(name.into()),
                implementation_version: Some("0.21.5".into()),
                ..DeclaredTarget::default()
            },
        )));
        run.drs_fixture = Some(fx);
        run
    }

    #[test]
    fn mock_named_bento_is_not_live_observation() {
        let run = run_labeled(
            BENTO_TARGET_ID,
            TargetKind::Mock,
            "bento_drs",
            DrsVerifyFixture::default_catalog(),
        );
        assert!(!run_counts_as_independent(&run));
        assert!(!live_independent_observation(&run));
    }

    #[test]
    fn real_kind_default_catalog_is_not_live_observation() {
        let run = run_labeled(
            BENTO_TARGET_ID,
            TargetKind::RealIndependentLocalImplementation,
            "bento_drs",
            DrsVerifyFixture::default_catalog(),
        );
        assert!(run_counts_as_independent(&run));
        assert!(!live_independent_observation(&run));
    }

    #[test]
    fn operator_declared_default_object_id_is_not_live_observation() {
        let fx = DrsVerifyFixture::operator_declared(DEFAULT_DRS_OBJECT_ID.into(), None).unwrap();
        let run = run_labeled(
            BENTO_TARGET_ID,
            TargetKind::RealIndependentLocalImplementation,
            "bento_drs",
            fx,
        );
        assert!(!live_independent_observation(&run));
    }

    #[test]
    fn reviewed_bento_with_operator_fixture_counts() {
        let fx = DrsVerifyFixture::operator_declared(
            BENTO_OBJECT_ID.into(),
            Some(BENTO_OBJECT_SHA256.into()),
        )
        .unwrap();
        let run = run_labeled(
            BENTO_TARGET_ID,
            TargetKind::RealIndependentLocalImplementation,
            "bento_drs",
            fx,
        );
        assert!(run_counts_as_independent(&run));
        assert!(live_independent_observation(&run));
    }
}
