#![deny(warnings)]

extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate libc;

use std::ffi::CString;
use std::fs;
use std::io::Write;
use std::process::exit;
use std::os::unix::io::AsRawFd;

#[derive(Debug, Deserialize)]
struct Config {
    helpers: Vec<Helper>,
}

#[derive(Debug, Deserialize)]
struct Helper {
    path: String,
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
            return
        }
    };

    unsafe {
        let ret = libc::dup2(file.as_raw_fd(), libc::STDERR_FILENO);
        if ret < 0 {
            libc::perror(CString::new("couldn't dup2 over stderr").expect("constant string").as_ptr());
        }
    }
}

fn main() {
    let _ = std::env::var("HULDUFOLK_DEBUG").map_err(|_| log_to_kmsg());

    let path = DEFAULT_CONFIG_PATH.unwrap_or("/etc/usermode-helper.conf");
    let raw = fs::read_to_string(path).unwrap_or_else(|e| {
        fail!("couldn't read config file {}: {}", path, e)
    });

    let config: Config = toml::from_str(&raw).unwrap_or_else(|e| {
        fail!("couldn't parse config file {}: {}", path, e);
    });

    let name = std::env::args().nth(0).expect("program doesn't have a 0 arg?");

    // Should we support regexes here? Probably not.
    let thing = config.helpers.iter().find(|s| s.path == name).unwrap_or_else(|| {
        fail!("invalid usermode helper {}", name);
    });

    let c_exe = CString::new(thing.path.clone()).unwrap_or_else(|e| {
        fail!("couldn't create exec executable name: {}", e);
    });

    let c_args: Vec<_> = std::env::args().skip(1).map(|a| {
        CString::new(a).unwrap_or_else(|e| {
            fail!("couldn't create exec args array: {}", e);
        })
    }).collect();

    let ptr_args: Vec<_> = std::iter::once(c_exe.as_ptr()).chain(c_args.iter().map(|a| {
        a.as_ptr()
    }).chain(std::iter::once(std::ptr::null()))).collect();

    unsafe {
        libc::execvp(c_exe.as_ptr(), ptr_args.as_ptr());
        libc::perror(CString::new("couldn't execvp").expect("constant string").as_ptr());
    }
    exit(1);
}
