// SPDX-License-Identifier: Apache-2.0
//! Fail closed if VERSIONS.lock checker digest is not the sibling sources Cargo compiles.

use sha2::{Digest, Sha256};
use std::env;
use std::path::Path;

const FILES: &[&str] = &[
    "crates/framework/src/drs.rs",
    "crates/common/src/ga4gh_schemas.rs",
    "crates/common/src/spec_source.rs",
];

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

fn checker_source_sha256(helixtest_root: &Path) -> String {
    let mut buf = String::from("helix-drs-checker-v1\n");
    for rel in FILES {
        let path = helixtest_root.join(rel);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        buf.push_str(&format!("file={rel}\nsha256={}\n", sha256_hex(&bytes)));
        println!("cargo:rerun-if-changed={}", path.display());
    }
    sha256_hex(buf.as_bytes())
}

fn lock_checker_digest(lock: &str) -> String {
    for line in lock.lines() {
        if let Some(hex) = line.strip_prefix("HELIXTEST_CHECKER_SOURCE_SHA256=") {
            return hex.trim().to_string();
        }
    }
    panic!("VERSIONS.lock missing HELIXTEST_CHECKER_SOURCE_SHA256");
}

fn main() {
    let manifest = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).to_path_buf();
    let lock_path = manifest.join("VERSIONS.lock");
    println!("cargo:rerun-if-changed={}", lock_path.display());
    let lock = std::fs::read_to_string(&lock_path).expect("VERSIONS.lock");
    let expected = lock_checker_digest(&lock);
    let actual = checker_source_sha256(&manifest.join("../HelixTest/helixtest"));
    if actual != expected {
        panic!(
            "VERSIONS.lock HELIXTEST_CHECKER_SOURCE_SHA256={expected} but compiled HelixTest DRS checker sources hash to {actual}. Update the lock to the digest of the sources Cargo compiles. Do not report a git SHA as the executed checker."
        );
    }
    println!("cargo:rustc-env=HELIX_EXPECTED_CHECKER_SOURCE_SHA256={expected}");
}
