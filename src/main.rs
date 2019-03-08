#![deny(warnings)]

extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate libc;

use std::ffi::CString;
use std::fs;
use std::process::exit;

#[derive(Debug, Deserialize)]
struct Config {
    helpers: Vec<Helper>,
}

#[derive(Debug, Deserialize)]
struct Helper {
    path: String,
}

const DEFAULT_CONFIG_PATH: Option<&'static str> = option_env!("DEFAULT_CONFIG_PATH");

fn main() {
    let path = DEFAULT_CONFIG_PATH.unwrap_or("/etc/usermode-helper.conf");
    let raw = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("couldn't read config file {}: {}", path, e);
        exit(1)
    });

    let config: Config = toml::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("couldn't parse config file {}: {}", path, e);
        exit(1)
    });

    let name = std::env::args().nth(0).expect("program doesn't have a 0 arg?");

    // Should we support regexes here? Probably not.
    let thing = config.helpers.iter().find(|s| s.path == name).unwrap_or_else(|| {
        eprintln!("invalid usermode helper {}", name);
        exit(2)
    });

    let c_exe = CString::new(thing.path.clone()).unwrap_or_else(|e| {
        eprintln!("couldn't create exec executable name: {}", e);
        exit(1)
    });

    let c_args: Vec<_> = std::env::args().skip(1).map(|a| {
        CString::new(a).unwrap_or_else(|e| {
            eprintln!("couldn't create exec args array: {}", e);
            exit(1)
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
