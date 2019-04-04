#![deny(warnings)]

extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate capabilities;
extern crate libc;

use std::convert::From;
use std::ffi::{CString, OsString};
use std::fs;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::process::exit;

// The "prctl" library depends on nix, which itself depends on some other stuff. We only need these
// few defines, so let's just hard code them here. Plus, it doesn't have the ambient ones.
const PR_SET_SECUREBITS: libc::c_int = 28;
const SECBIT_NOROOT: libc::c_int = 0x01;
const PR_CAP_AMBIENT: libc::c_int = 47;
const PR_CAP_AMBIENT_RAISE: libc::c_int = 2;

#[derive(Deserialize)]
struct Config {
    helpers: Vec<Helper>,
}

#[derive(Deserialize)]
struct Helper {
    path: String,
    argc: Option<usize>,
    #[serde(deserialize_with = "deserialize_caps", default)]
    capabilities: Option<capabilities::Capabilities>,
}

impl Helper {
    fn allowed(&self, args: &Vec<OsString>) -> bool {
        if !args
            .get(0)
            .map_or(false, |a| a == &OsString::from(&self.path))
        {
            return false;
        }

        if self.argc.map_or(false, |argc| args.len() != argc) {
            return false;
        }

        return true;
    }
}

fn deserialize_caps<'de, D>(deserializer: D) -> Result<Option<capabilities::Capabilities>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    if s == "" {
        return Ok(None);
    }
    return s
        .parse::<capabilities::Capabilities>()
        .map_err(|_| serde::de::Error::custom(format!("invalid caps {}", s)))
        .map(|v| Some(v));
}

const DEFAULT_CONFIG_PATH: Option<&'static str> = option_env!("DEFAULT_CONFIG_PATH");

macro_rules! fail {
    ($($arg:tt)*) => ({
        let log = std::fmt::format(format_args!($($arg)*));
	let _ = std::io::stderr().write(log.as_ref());
	std::process::exit(1)
    })
}

/*
 * TODO: should we exit(1) if this fails? It would be nice to know, but also this could stop
 * peoples' systems from booting...
 */
fn log_to_kmsg() {
    let file = match fs::OpenOptions::new().write(true).open("/dev/kmsg") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("couldn't open /dev/kmsg: {}", e);
            return;
        }
    };

    unsafe {
        let ret = libc::dup2(file.as_raw_fd(), libc::STDERR_FILENO);
        if ret < 0 {
            libc::perror(
                CString::new("couldn't dup2 over stderr")
                    .expect("constant string")
                    .as_ptr(),
            );
        }
    }
}

fn main() {
    let _ = std::env::var("HULDUFOLK_DEBUG").map_err(|_| log_to_kmsg());

    let path = DEFAULT_CONFIG_PATH.unwrap_or("/etc/usermode-helper.conf");
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|e| fail!("couldn't read config file {}: {}", path, e));

    let config: Config = toml::from_str(&raw).unwrap_or_else(|e| {
        fail!("couldn't parse config file {}: {}", path, e);
    });

    let name = std::env::args()
        .nth(0)
        .expect("program doesn't have a 0 arg?");

    let args = std::env::args_os().collect();
    let thing = config
        .helpers
        .iter()
        .find(|s| s.allowed(&args))
        .unwrap_or_else(|| {
            fail!("invalid usermode helper {}", name);
        });

    if let Some(capabilities) = &thing.capabilities {
        unsafe {
            let ret = libc::prctl(PR_SET_SECUREBITS, SECBIT_NOROOT, 0, 0, 0);
            if ret < 0 {
                libc::perror(
                    CString::new("couldn't set securebits")
                        .expect("constant string")
                        .as_ptr(),
                );
            }
        }

        if let Err(e) = capabilities.apply() {
            fail!("couldn't apply caps: {}", e)
        }

        for cap in 0..capabilities::Capability::CAP_LAST_CAP as u32 {
            if !capabilities.check(cap.into(), capabilities::Flag::Inheritable) {
                continue;
            }

            unsafe {
                let ret = libc::prctl(PR_CAP_AMBIENT, PR_CAP_AMBIENT_RAISE, cap, 0, 0);
                if ret < 0 {
                    libc::perror(
                        CString::new(format!("couldn't set ambient cap {}", cap))
                            .expect("constant string")
                            .as_ptr(),
                    );
                }
            }
        }
    }

    let c_exe = CString::new(thing.path.clone()).unwrap_or_else(|e| {
        fail!("couldn't create exec executable name: {}", e);
    });

    let c_args: Vec<_> = std::env::args()
        .skip(1)
        .map(|a| {
            CString::new(a).unwrap_or_else(|e| {
                fail!("couldn't create exec args array: {}", e);
            })
        })
        .collect();

    let ptr_args: Vec<_> = std::iter::once(c_exe.as_ptr())
        .chain(
            c_args
                .iter()
                .map(|a| a.as_ptr())
                .chain(std::iter::once(std::ptr::null())),
        )
        .collect();

    unsafe {
        libc::execvp(c_exe.as_ptr(), ptr_args.as_ptr());
        libc::perror(
            CString::new("couldn't execvp")
                .expect("constant string")
                .as_ptr(),
        );
    }
    exit(1);
}
