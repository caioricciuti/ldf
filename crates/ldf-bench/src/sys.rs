//! Process resource usage and machine description, without extra crates.

use std::process::Command;

use serde::{Deserialize, Serialize};

#[cfg(target_os = "macos")]
mod ffi {
    #[repr(C)]
    pub struct Timeval {
        pub tv_sec: i64,
        pub tv_usec: i32,
    }
    #[repr(C)]
    pub struct Rusage {
        pub ru_utime: Timeval,
        pub ru_stime: Timeval,
        pub ru_maxrss: i64,
        pub rest: [i64; 13],
    }
    pub const MAXRSS_UNIT_BYTES: i64 = 1;
}

#[cfg(target_os = "linux")]
mod ffi {
    #[repr(C)]
    pub struct Timeval {
        pub tv_sec: i64,
        pub tv_usec: i64,
    }
    #[repr(C)]
    pub struct Rusage {
        pub ru_utime: Timeval,
        pub ru_stime: Timeval,
        pub ru_maxrss: i64,
        pub rest: [i64; 13],
    }
    pub const MAXRSS_UNIT_BYTES: i64 = 1024;
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
unsafe extern "C" {
    fn getrusage(who: i32, usage: *mut ffi::Rusage) -> i32;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Usage {
    /// Peak resident set size of this process so far, in bytes.
    pub max_rss_bytes: u64,
    pub user_cpu_secs: f64,
    pub system_cpu_secs: f64,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn usage() -> Usage {
    let mut ru = ffi::Rusage {
        ru_utime: ffi::Timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        ru_stime: ffi::Timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        ru_maxrss: 0,
        rest: [0; 13],
    };
    // SAFETY: RUSAGE_SELF (0) with a correctly laid out, writable struct.
    let rc = unsafe { getrusage(0, &mut ru) };
    if rc != 0 {
        return Usage::default();
    }
    Usage {
        max_rss_bytes: (ru.ru_maxrss * ffi::MAXRSS_UNIT_BYTES) as u64,
        user_cpu_secs: ru.ru_utime.tv_sec as f64 + ru.ru_utime.tv_usec as f64 * 1e-6,
        system_cpu_secs: ru.ru_stime.tv_sec as f64 + ru.ru_stime.tv_usec as f64 * 1e-6,
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn usage() -> Usage {
    Usage::default()
}

#[derive(Debug, Clone, Serialize)]
pub struct SysInfo {
    pub os: String,
    pub arch: String,
    pub cpu: String,
    pub logical_cpus: usize,
    pub memory_bytes: u64,
    pub rustc: String,
    pub cargo_profile: &'static str,
    pub git_commit: String,
    pub crate_versions: Vec<(String, String)>,
}

fn sh(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

pub fn sysinfo() -> SysInfo {
    let (cpu, mem) = if cfg!(target_os = "macos") {
        (
            sh("sysctl", &["-n", "machdep.cpu.brand_string"]),
            sh("sysctl", &["-n", "hw.memsize"]).parse().unwrap_or(0),
        )
    } else {
        let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let model = cpuinfo
            .lines()
            .find(|l| l.starts_with("model name"))
            .and_then(|l| l.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".into());
        let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
        let kb: u64 = meminfo
            .lines()
            .find(|l| l.starts_with("MemTotal"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        (model, kb * 1024)
    };
    let lock = include_str!("../../../Cargo.lock");
    let interesting = [
        "serde_json",
        "simd-json",
        "csv",
        "arrow",
        "parquet",
        "tiktoken-rs",
        "criterion",
    ];
    let mut crate_versions = Vec::new();
    let mut lines = lock.lines().peekable();
    while let Some(l) = lines.next() {
        if let Some(name) = l
            .strip_prefix("name = \"")
            .and_then(|s| s.strip_suffix('"'))
            && interesting.contains(&name)
            && let Some(v) = lines
                .peek()
                .and_then(|n| n.strip_prefix("version = \""))
                .and_then(|s| s.strip_suffix('"'))
        {
            crate_versions.push((name.to_string(), v.to_string()));
        }
    }
    SysInfo {
        os: format!("{} {}", sh("uname", &["-s"]), sh("uname", &["-r"])),
        arch: std::env::consts::ARCH.to_string(),
        cpu,
        logical_cpus: std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(0),
        memory_bytes: mem,
        rustc: sh("rustc", &["--version"]),
        cargo_profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        git_commit: sh("git", &["rev-parse", "--short", "HEAD"]),
        crate_versions,
    }
}
