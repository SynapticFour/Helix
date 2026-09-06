// SPDX-License-Identifier: Apache-2.0
//! Fail closed if VERSIONS.lock checker digest is not the sibling DRS closure
//! Cargo compiles. Same algorithm as HelixTest `checker_identity.rs`.

use sha2::{Digest, Sha256};
use std::env;
use std::path::Path;
use std::process::Command;

const MANIFEST_VERSION: &str = "helix-drs-checker-v2";
const LIST_REL: &str = "crates/framework/checker_source_v2.txt";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

fn parse_listed_paths(list: &str) -> Vec<&str> {
    list.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

fn lock_value(lock: &str, key: &str) -> String {
    let prefix = format!("{key}=");
    for line in lock.lines() {
        if let Some(v) = line.strip_prefix(&prefix) {
            return v.trim().to_string();
        }
    }
    panic!("VERSIONS.lock missing {key}");
}

fn checker_source_sha256(helixtest_root: &Path) -> String {
    let list_path = helixtest_root.join(LIST_REL);
    println!("cargo:rerun-if-changed={}", list_path.display());
    let list_bytes =
        std::fs::read(&list_path).unwrap_or_else(|e| panic!("read {}: {e}", list_path.display()));
    let list_text = String::from_utf8(list_bytes.clone()).expect("checker_source_v2.txt utf-8");
    let mut buf = format!(
        "{MANIFEST_VERSION}\nfile={LIST_REL}\nsha256={}\n",
        sha256_hex(&list_bytes)
    );
    for rel in parse_listed_paths(&list_text) {
        let path = helixtest_root.join(rel);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        buf.push_str(&format!("file={rel}\nsha256={}\n", sha256_hex(&bytes)));
        println!("cargo:rerun-if-changed={}", path.display());
    }
    sha256_hex(buf.as_bytes())
}

fn sibling_git_head(repo: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["-C", repo.to_str()?, "rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() {
    let manifest = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).to_path_buf();
    let lock_path = manifest.join("VERSIONS.lock");
    println!("cargo:rerun-if-changed={}", lock_path.display());
    let lock = std::fs::read_to_string(&lock_path).expect("VERSIONS.lock");
    let expected = lock_value(&lock, "HELIXTEST_CHECKER_SOURCE_SHA256");
    let want_git = lock_value(&lock, "HELIXTEST_SHA");
    let helixtest_repo = manifest.join("../HelixTest");
    let helixtest_root = helixtest_repo.join("helixtest");
    let actual = checker_source_sha256(&helixtest_root);
    if actual != expected {
        panic!(
            "VERSIONS.lock HELIXTEST_CHECKER_SOURCE_SHA256={expected} but compiled HelixTest DRS checker closure hashes to {actual}. Update the lock to the digest of the sources Cargo compiles. Do not report a git SHA as the executed checker."
        );
    }
    if let Some(head) = sibling_git_head(&helixtest_repo) {
        if head != want_git {
            panic!(
                "VERSIONS.lock HELIXTEST_SHA={want_git} but sibling HelixTest HEAD is {head}. Checkout the pinned commit. Git SHA is the checkout pin, not the executed checker identity."
            );
        }
    }
    println!("cargo:rustc-env=HELIX_EXPECTED_CHECKER_SOURCE_SHA256={expected}");
    emit_helix_git_provenance(&manifest);
}

/// Compile-time Helix checkout identity. Not a verification claim. Not HELIOS.
/// Empty SHA / dirty=`unknown` when `.git` is missing — do not fabricate.
fn emit_helix_git_provenance(helix_root: &Path) {
    let git_dir = helix_root.join(".git");
    // HEAD is often a symbolic ref (`ref: refs/heads/...`). A commit updates the
    // branch file, not HEAD itself — watching only HEAD leaves HELIX_GIT_SHA stale.
    println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
    println!("cargo:rerun-if-changed={}", git_dir.join("index").display());
    println!(
        "cargo:rerun-if-changed={}",
        git_dir.join("packed-refs").display()
    );
    if let Ok(head) = std::fs::read_to_string(git_dir.join("HEAD")) {
        if let Some(rel) = head.trim().strip_prefix("ref: ") {
            println!("cargo:rerun-if-changed={}", git_dir.join(rel).display());
        }
    }
    let sha = git_stdout(helix_root, &["rev-parse", "HEAD"])
        .filter(|s| s.len() == 40 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')))
        .unwrap_or_default();
    let dirty = match git_stdout(helix_root, &["status", "--porcelain"]) {
        Some(_) if sha.is_empty() => "unknown".to_string(),
        Some(status) if status.is_empty() => "false".to_string(),
        Some(_) => "true".to_string(),
        None => "unknown".to_string(),
    };
    println!("cargo:rustc-env=HELIX_GIT_SHA={sha}");
    println!("cargo:rustc-env=HELIX_GIT_DIRTY={dirty}");
}

fn git_stdout(repo: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
