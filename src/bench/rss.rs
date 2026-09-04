// SPDX-License-Identifier: Apache-2.0
//! Optional client RSS if the OS exposes it. Linux: VmRSS of this process (not Ferrum).

pub fn rss_source() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "linux_proc_self_vmrss"
    }
    #[cfg(not(target_os = "linux"))]
    {
        "unavailable"
    }
}

pub fn rss_kb() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                return rest.split_whitespace().next()?.parse().ok();
            }
        }
        None
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}
