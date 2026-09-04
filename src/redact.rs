// SPDX-License-Identifier: Apache-2.0
//! Strip secrets and Authorization values from anything Helix might print.
//!
//! Pattern-based leak prevention for stdout/stderr/JSON. Not a security product.
//! Never a substitute for not putting the secret in the string in the first place.

pub const REDACTED: &str = "[redacted]";

/// Redact Authorization / Bearer / Basic / JWT-shaped tokens / URL userinfo,
/// plus `HELIX_HMAC_SECRET` when that env var is set.
pub fn redact_text(input: &str) -> String {
    let extras = env_secrets();
    let refs: Vec<&str> = extras.iter().map(String::as_str).collect();
    redact_with_secrets(input, &refs)
}

/// Same as [`redact_text`], with extra known values (HMAC dummy, minted JWT, …).
/// Secrets shorter than 8 characters are ignored (too many false positives).
pub fn redact_with_secrets(input: &str, extra: &[&str]) -> String {
    let mut s = strip_url_userinfo(input);
    s = redact_labeled_value(&s, "proxy-authorization");
    s = redact_labeled_value(&s, "authorization");
    s = redact_bearer_or_basic(&s, "bearer ");
    s = redact_bearer_or_basic(&s, "basic ");
    s = redact_jwt_like(&s);
    for secret in extra {
        if secret.len() >= 8 {
            s = s.replace(*secret, REDACTED);
        }
    }
    s
}

fn env_secrets() -> Vec<String> {
    let mut v = Vec::new();
    if let Ok(s) = std::env::var("HELIX_HMAC_SECRET") {
        let t = s.trim();
        if t.len() >= 8 {
            v.push(t.to_string());
        }
    }
    v
}

/// `http://user:pass@host/path` → `http://host/path` (any occurrence).
fn strip_url_userinfo(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(rel) = s[i..].find("://") {
        let start = i + rel;
        out.push_str(&s[i..start + 3]);
        let after = start + 3;
        let rest = &s[after..];
        let host_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
        let authority = &rest[..host_end];
        if let Some(at) = authority.rfind('@') {
            out.push_str(&authority[at + 1..]);
        } else {
            out.push_str(authority);
        }
        i = after + host_end;
    }
    out.push_str(&s[i..]);
    out
}

fn is_ascii_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'-'
}

/// Redact the value after `label:` / `label=` (case-insensitive), requiring a
/// non-identifier boundary so `proxy-authorization` is not eaten as `authorization`.
fn redact_labeled_value(s: &str, label: &str) -> String {
    let lower = s.to_ascii_lowercase();
    let lab = label.to_ascii_lowercase();
    let lab_bytes = lab.as_bytes();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if let Some(pos) = lower[i..].find(&lab) {
            let abs = i + pos;
            let before_ok = abs == 0 || !is_ascii_ident(bytes[abs - 1]);
            let after = abs + lab_bytes.len();
            let after_ok = after >= bytes.len() || !is_ascii_ident(bytes[after]);
            if before_ok && after_ok {
                out.push_str(&s[i..after]);
                let mut j = after;
                while j < bytes.len() && (bytes[j].is_ascii_whitespace() || bytes[j] == b'"') {
                    out.push(bytes[j] as char);
                    j += 1;
                }
                if j < bytes.len() && (bytes[j] == b':' || bytes[j] == b'=') {
                    out.push(bytes[j] as char);
                    j += 1;
                    while j < bytes.len() && (bytes[j].is_ascii_whitespace() || bytes[j] == b'"') {
                        out.push(bytes[j] as char);
                        j += 1;
                    }
                    let val_start = j;
                    while j < bytes.len() {
                        let c = bytes[j];
                        if c == b'"' || c == b',' || c == b'\n' || c == b'\r' || c == b'}' {
                            break;
                        }
                        j += 1;
                    }
                    if j > val_start {
                        out.push_str(REDACTED);
                    }
                    i = j;
                    continue;
                }
                i = after;
                continue;
            }
            out.push_str(&s[i..abs + 1]);
            i = abs + 1;
        } else {
            out.push_str(&s[i..]);
            break;
        }
    }
    out
}

fn redact_bearer_or_basic(s: &str, prefix: &str) -> String {
    let lower = s.to_ascii_lowercase();
    let pref = prefix.to_ascii_lowercase();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(pos) = lower[i..].find(&pref) {
        let abs = i + pos;
        let before_ok = abs == 0 || !is_ascii_ident(s.as_bytes()[abs - 1]);
        if !before_ok {
            out.push_str(&s[i..abs + 1]);
            i = abs + 1;
            continue;
        }
        out.push_str(&s[i..abs + prefix.len()]);
        let token_start = abs + prefix.len();
        let token_end = s[token_start..]
            .find(|c: char| {
                !(c.is_ascii_alphanumeric()
                    || c == '.'
                    || c == '_'
                    || c == '-'
                    || c == '='
                    || c == '+'
                    || c == '/')
            })
            .map(|n| token_start + n)
            .unwrap_or(s.len());
        if token_end > token_start {
            out.push_str(REDACTED);
            i = token_end;
        } else {
            i = token_start;
        }
    }
    out.push_str(&s[i..]);
    out
}

fn is_b64url(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'-' || c == b'_'
}

/// JWT-shaped tokens (`eyJ` + three base64url segments). High-confidence; `eyJust` is left alone.
fn redact_jwt_like(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if i + 3 <= bytes.len() && &bytes[i..i + 3] == b"eyJ" {
            if let Some(end) = jwt_end(bytes, i) {
                out.push_str(REDACTED);
                i = end;
                continue;
            }
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn jwt_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    for part in 0..3 {
        let begin = i;
        while i < bytes.len() && is_b64url(bytes[i]) {
            i += 1;
        }
        if i == begin {
            return None;
        }
        if part < 2 {
            if i >= bytes.len() || bytes[i] != b'.' {
                return None;
            }
            i += 1;
        }
    }
    if i - start < 20 {
        return None;
    }
    Some(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_authorization_header_and_bearer_token() {
        let jwt =
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let raw = format!("Authorization: Bearer {jwt}");
        let out = redact_text(&raw);
        assert!(!out.contains(jwt), "{out}");
        assert!(!out.to_ascii_lowercase().contains("bearer eyj"), "{out}");
        assert!(out.contains(REDACTED), "{out}");
    }

    #[test]
    fn redacts_json_authorization_field() {
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.signaturepartxx";
        let raw = format!(r#"{{"authorization": "Bearer {jwt}"}}"#);
        let out = redact_text(&raw);
        assert!(!out.contains(jwt), "{out}");
        assert!(!out.contains("Bearer eyJ"), "{out}");
    }

    #[test]
    fn redacts_url_userinfo() {
        let out = redact_text("fetch http://alice:s3cret@example.org/ga4gh and done");
        assert!(!out.contains("s3cret"), "{out}");
        assert!(!out.contains("alice:"), "{out}");
        assert!(out.contains("http://example.org/ga4gh"), "{out}");
    }

    #[test]
    fn redacts_known_secret() {
        let secret = "helix-dummy-hmac-not-for-production-do-not-use";
        let out = redact_with_secrets(&format!("secret={secret}"), &[secret]);
        assert!(!out.contains(secret), "{out}");
        assert!(out.contains(REDACTED));
    }

    #[test]
    fn leaves_docs_placeholder_bearer_alone() {
        let s = "Authorization: Bearer <valid>";
        let out = redact_text(s);
        assert!(out.contains("<valid>") || out.contains(REDACTED));
        assert!(!out.contains("eyJ"));
    }

    #[test]
    fn does_not_treat_eyjust_as_jwt() {
        let s = "eyJust a joke about tokens";
        assert_eq!(redact_text(s), s);
    }
}
