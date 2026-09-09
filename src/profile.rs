// SPDX-License-Identifier: Apache-2.0
//! Declarative Helix verify profiles (`generic`, `ferrum`).
//!
//! A profile is policy: expected services, enabled/optional checks, fixtures,
//! and HelixTest `Features`. It is not a plugin and not a second engine.
//!
//! The verification engine always uses public GA4GH HTTP and HelixTest
//! `Mode::Generic`. Ferrum is never selected from a service-info `name`.
//! Not HELIOS.

use crate::discover::Ga4ghService;

/// Helix profile id. Distinct from HelixTest `--profile` TOML names
/// (`ga4gh-drs`, `ferrum.toml`) and from `Mode::Generic` on each check row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileId {
    Generic,
    Ferrum,
}

impl ProfileId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Generic => "generic",
            Self::Ferrum => "ferrum",
        }
    }
}

/// HelixTest `Features` bits Helix is allowed to set. Not Ferrum mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    pub strict_drs_checksums: bool,
    pub supports_scatter_gather: bool,
}

/// Static profile. New profiles are new `const`s, not a plugin loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Profile {
    pub id: ProfileId,
    /// Must be DETECTED. Missing → fail (not skip). Empty = none required.
    pub expected_services: &'static [Ga4ghService],
    /// Suites Helix executes when DETECTED + TESTABLE. Subset of engine capability.
    pub enabled_services: &'static [Ga4ghService],
    /// Helix ids that may skip when a capability is off. Skip is never pass.
    pub optional_check_ids: &'static [&'static str],
    pub capabilities: Capabilities,
    /// HelixTest fixture identifiers the target must satisfy when those checks run.
    pub required_fixtures: &'static [&'static str],
}

impl Profile {
    pub fn expects(self, kind: Ga4ghService) -> bool {
        self.expected_services.contains(&kind)
    }

    pub fn enables(self, kind: Ga4ghService) -> bool {
        self.enabled_services.contains(&kind)
    }

    pub fn check_is_optional(self, id: &str) -> bool {
        self.optional_check_ids.contains(&id)
    }
}

const ENABLED_DRS_WES: &[Ga4ghService] = &[Ga4ghService::Drs, Ga4ghService::Wes];

const GENERIC_FIXTURES: &[&str] = &[
    "drs:test-object-1",
    "wes:trs://test-tool/echo/1.0",
    "wes:trs://test-tool/fail/1.0",
    "wes:trs://test-tool/cwl-echo/1.0",
    "wes:trs://nonexistent/invalid/0.0",
];

const FERRUM_FIXTURES: &[&str] = &[
    "drs:test-object-1",
    "wes:trs://test-tool/echo/1.0",
    "wes:trs://test-tool/fail/1.0",
    "wes:trs://test-tool/cwl-echo/1.0",
    "wes:trs://nonexistent/invalid/0.0",
    "wes:trs://test-tool/scatter-gather/1.0",
];

/// Public GA4GH HTTP. No service is required to be present.
/// Checksums on (Helix `ga4gh-drs`-style). Scatter/gather optional (off).
pub const GENERIC: Profile = Profile {
    id: ProfileId::Generic,
    expected_services: &[],
    enabled_services: ENABLED_DRS_WES,
    optional_check_ids: &["wes.run.scatter_gather"],
    capabilities: Capabilities {
        strict_drs_checksums: true,
        supports_scatter_gather: false,
    },
    required_fixtures: GENERIC_FIXTURES,
};

/// Ferrum as a **target** (HelixTest `profiles/ferrum.toml` features we actually run).
/// Same public HTTP as generic. DRS and WES must answer. Scatter/gather enabled.
/// Does not use HelixTest Ferrum mode. TES/TRS/htsget still not executed.
pub const FERRUM: Profile = Profile {
    id: ProfileId::Ferrum,
    expected_services: &[Ga4ghService::Drs, Ga4ghService::Wes],
    enabled_services: ENABLED_DRS_WES,
    optional_check_ids: &[],
    capabilities: Capabilities {
        strict_drs_checksums: true,
        supports_scatter_gather: true,
    },
    required_fixtures: FERRUM_FIXTURES,
};

pub fn definition(id: ProfileId) -> Profile {
    match id {
        ProfileId::Generic => GENERIC,
        ProfileId::Ferrum => FERRUM,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_does_not_require_services() {
        let g = definition(ProfileId::Generic);
        assert!(g.expected_services.is_empty());
        assert!(g.enables(Ga4ghService::Drs));
        assert!(g.enables(Ga4ghService::Wes));
        assert!(!g.enables(Ga4ghService::Tes));
        assert!(g.check_is_optional("wes.run.scatter_gather"));
        assert!(!g.capabilities.supports_scatter_gather);
        assert!(g.capabilities.strict_drs_checksums);
    }

    #[test]
    fn ferrum_expects_drs_and_wes_through_public_apis() {
        let f = definition(ProfileId::Ferrum);
        assert!(f.expects(Ga4ghService::Drs));
        assert!(f.expects(Ga4ghService::Wes));
        assert!(!f.expects(Ga4ghService::Tes));
        assert!(!f.check_is_optional("wes.run.scatter_gather"));
        assert!(f.capabilities.supports_scatter_gather);
        assert!(f
            .required_fixtures
            .contains(&"wes:trs://test-tool/scatter-gather/1.0"));
    }

    #[test]
    fn ids_are_stable_strings() {
        assert_eq!(ProfileId::Generic.as_str(), "generic");
        assert_eq!(ProfileId::Ferrum.as_str(), "ferrum");
        assert_eq!(definition(ProfileId::Generic).id, ProfileId::Generic);
        assert_eq!(definition(ProfileId::Ferrum).id, ProfileId::Ferrum);
    }
}
