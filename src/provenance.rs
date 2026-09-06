// SPDX-License-Identifier: Apache-2.0
//! Helix git checkout provenance. Not HELIOS. Not a verification identity.
//!
//! `helix_git_sha` is the commit this binary was built from (`git rev-parse HEAD`
//! at compile time). `helix_git_dirty` is whether that checkout had uncommitted
//! changes. Neither enters `execution_id`, `coverage_id`, `binding_id`, or
//! `catalog_id`. A missing `.git` directory yields `None` — Helix does not
//! fabricate a SHA.

/// Compile-time `git rev-parse HEAD`. Empty when `.git` was unavailable.
const EMBEDDED_SHA: &str = env!("HELIX_GIT_SHA");
/// `true` / `false` / empty (`unknown` when git was unavailable).
const EMBEDDED_DIRTY: &str = env!("HELIX_GIT_DIRTY");

/// Accept only a 40-char lowercase git SHA. Anything else is not a commit id.
pub fn parse_git_sha(raw: &str) -> Option<&str> {
    let s = raw.trim();
    if s.len() == 40 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        Some(s)
    } else {
        None
    }
}

pub fn parse_dirty(raw: &str) -> Option<bool> {
    match raw.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Helix commit this verifier binary was compiled from. `None` if the build
/// had no git metadata (do not invent one).
pub fn helix_git_sha() -> Option<&'static str> {
    parse_git_sha(EMBEDDED_SHA)
}

/// Whether the Helix checkout was dirty at compile time. `None` if unknown.
pub fn helix_git_dirty() -> Option<bool> {
    parse_dirty(EMBEDDED_DIRTY)
}

/// True only when the run records this binary's commit and dirty flag.
/// Absence of `helix_git_sha` means the artifact cannot be attributed to
/// this build (including pre-B12.1 live JSON).
pub fn cites_this_verifier_build(run: &crate::model::VerificationRun) -> bool {
    match (
        run.helix_git_sha.as_deref(),
        helix_git_sha(),
        run.helix_git_dirty,
        helix_git_dirty(),
    ) {
        (Some(recorded), Some(built), Some(rd), Some(bd)) => recorded == built && rd == bd,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_unknown_is_not_a_fabricated_sha() {
        assert_eq!(parse_git_sha(""), None);
        assert_eq!(parse_git_sha("unknown"), None);
        assert_eq!(parse_git_sha("NOTAHEX"), None);
        assert_eq!(parse_git_sha(&"0".repeat(39)), None);
        assert_eq!(parse_git_sha(&"z".repeat(40)), None);
        assert_eq!(
            parse_git_sha("8E0A1A725BEA163C8C7DEE136BAE34171335A2C1"),
            None,
            "uppercase is not a git SHA as embedded"
        );
    }

    #[test]
    fn lowercase_40_hex_is_accepted() {
        let sha = "8e0a1a725bea163c8c7dee136bae34171335a2c1";
        assert_eq!(parse_git_sha(sha), Some(sha));
    }

    #[test]
    fn different_commits_parse_to_different_values() {
        let a = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let b = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        assert_ne!(parse_git_sha(a), parse_git_sha(b));
    }

    #[test]
    fn dirty_parse_is_tri_state() {
        assert_eq!(parse_dirty("true"), Some(true));
        assert_eq!(parse_dirty("false"), Some(false));
        assert_eq!(parse_dirty(""), None);
        assert_eq!(parse_dirty("unknown"), None);
        assert_eq!(parse_dirty("TRUE"), None);
    }

    #[test]
    fn embedded_sha_contains_no_path_host_or_user() {
        if let Some(sha) = helix_git_sha() {
            assert!(!sha.contains('/'));
            assert!(!sha.contains('\\'));
            assert!(!sha.contains('@'));
            assert!(!sha.contains(':'));
        }
        assert!(!EMBEDDED_SHA.contains("/Users/"));
        assert!(!EMBEDDED_DIRTY.contains("/Users/"));
    }
}
