// SPDX-License-Identifier: Apache-2.0
//! Hygiene for target-controlled strings before they reach reports or logs.
//!
//! A malicious origin can fail Helix checks. It must not inject ANSI into a TTY,
//! forge extra report lines, or balloon JSON with multi-megabyte field copies.
//! Not a WAF. Not a security scanner. Secrets: [`crate::redact`].

use crate::redact::redact_text;

/// Cap for HelixTest error text, diagnostics `observed`, and similar.
pub const MAX_UNTRUSTED_CHARS: usize = 8 * 1024;

/// Cap for service-info id / name / version snapshots.
pub const MAX_UNTRUSTED_SHORT_CHARS: usize = 512;

/// Redact secrets, strip terminal/log controls, cap length.
pub fn sanitize_untrusted(input: &str) -> String {
    sanitize_untrusted_n(input, MAX_UNTRUSTED_CHARS)
}

pub fn sanitize_untrusted_n(input: &str, max_chars: usize) -> String {
    let redacted = redact_text(input);
    let stripped = strip_ansi_and_controls(&redacted);
    truncate_chars(&stripped, max_chars)
}

/// CSI / OSC / C0 / C1 → gone or space. Newlines become spaces (log/report injection).
pub fn strip_ansi_and_controls(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b {
            i += 1;
            if i >= bytes.len() {
                break;
            }
            match bytes[i] {
                b'[' => {
                    i += 1;
                    while i < bytes.len() && !(0x40..=0x7e).contains(&bytes[i]) {
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1;
                    }
                }
                b']' => {
                    i += 1;
                    while i < bytes.len() {
                        if bytes[i] == 0x07 {
                            i += 1;
                            break;
                        }
                        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
            continue;
        }
        let Some(ch) = s.get(i..).and_then(|rest| rest.chars().next()) else {
            break;
        };
        let u = ch as u32;
        if u < 0x20 || (0x7f..=0x9f).contains(&u) {
            out.push(' ');
        } else {
            out.push(ch);
        }
        i += ch.len_utf8();
    }
    out
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    let mut it = s.chars();
    let taken: String = it.by_ref().take(max_chars).collect();
    if it.next().is_some() {
        format!("{taken}…")
    } else {
        taken
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_csi_color_and_does_not_keep_esc() {
        let raw = "FAIL \x1b[32mPASS\x1b[0m hidden";
        let out = sanitize_untrusted(raw);
        assert!(!out.contains('\u{1b}'), "{out:?}");
        assert!(out.contains("PASS"), "{out}");
        assert!(out.contains("FAIL"), "{out}");
    }

    #[test]
    fn newlines_cannot_forge_report_lines() {
        let raw = "got 200\nHELIX VERIFICATION\n  5 PASS";
        let out = sanitize_untrusted(raw);
        assert!(!out.contains('\n'), "{out:?}");
        assert!(out.contains("HELIX VERIFICATION"), "{out}");
    }

    #[test]
    fn caps_long_fields() {
        let raw = "A".repeat(MAX_UNTRUSTED_CHARS + 50);
        let out = sanitize_untrusted(&raw);
        assert!(
            out.chars().count() <= MAX_UNTRUSTED_CHARS + 1,
            "{}",
            out.len()
        );
        assert!(out.ends_with('…'), "{out}");
    }

    #[test]
    fn still_redacts_jwt() {
        let jwt =
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let out = sanitize_untrusted(&format!("Authorization: Bearer {jwt}"));
        assert!(!out.contains(jwt), "{out}");
    }

    #[test]
    fn short_cap_used_for_service_info() {
        let raw = "n".repeat(MAX_UNTRUSTED_SHORT_CHARS + 10);
        let out = sanitize_untrusted_n(&raw, MAX_UNTRUSTED_SHORT_CHARS);
        assert!(out.ends_with('…'));
        assert!(out.chars().count() <= MAX_UNTRUSTED_SHORT_CHARS + 1);
    }
}
