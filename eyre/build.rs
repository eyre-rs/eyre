use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus};
use std::str;

// This code exercises the surface area that we expect of the Error generic
// member access API. If the current toolchain is able to compile it, then
// anyhow is able to provide backtrace support.
const BACKTRACE_PROBE: &str = r#"
    #![feature(error_generic_member_access)]

    use std::backtrace::Backtrace;
    use std::error::{self, Error, Request};
    use std::fmt::{self, Debug, Display};

    struct MyError(Thing);
    struct Thing;

    impl Debug for MyError {
        fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }

    impl Display for MyError {
        fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }

    impl Error for MyError {
        fn provide<'a>(&'a self, request: &mut Request<'a>) {
            request.provide_ref(&self.0);
        }
    }

    const _: fn(&dyn Error) -> Option<&Backtrace> = |err| error::request_ref::<Backtrace>(err);
"#;

const TRACK_CALLER_PROBE: &str = r#"
    #![allow(dead_code)]

    #[track_caller]
    fn foo() {
        let _location = std::panic::Location::caller();
    }
"#;

fn main() {
    match compile_probe(BACKTRACE_PROBE) {
        Some(status) if status.success() => println!("cargo:rustc-cfg=backtrace"),
        _ => {}
    }

    match compile_probe(TRACK_CALLER_PROBE) {
        Some(status) if status.success() => println!("cargo:rustc-cfg=track_caller"),
        _ => {}
    }

    let version = match rustc_version_info() {
        Some(version) => version,
        None => return,
    };

    version.toolchain.set_feature();

    if version.minor < 52 {
        println!("cargo:rustc-cfg=eyre_no_fmt_arguments_as_str");
    }

    if version.minor < 58 {
        println!("cargo:rustc-cfg=eyre_no_fmt_args_capture");
    }
}

fn compile_probe(probe: &str) -> Option<ExitStatus> {
    let rustc = env::var_os("RUSTC")?;
    let out_dir = env::var_os("OUT_DIR")?;
    let probefile = Path::new(&out_dir).join("probe.rs");
    fs::write(&probefile, probe).ok()?;
    Command::new(rustc)
        .arg("--edition=2018")
        .arg("--crate-name=eyre_build")
        .arg("--crate-type=lib")
        .arg("--emit=metadata")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(probefile)
        .status()
        .ok()
}

// TODO factor this toolchain parsing and related tests into its own file
#[derive(PartialEq)]
enum Toolchain {
    Stable,
    Beta,
    Nightly,
}
impl Toolchain {
    fn set_feature(self) {
        match self {
            Toolchain::Nightly => println!("cargo:rustc-cfg=nightly"),
            Toolchain::Beta => println!("cargo:rustc-cfg=beta"),
            Toolchain::Stable => println!("cargo:rustc-cfg=stable"),
        }
    }
}

struct VersionInfo {
    minor: u32,
    toolchain: Toolchain,
}

fn rustc_version_info() -> Option<VersionInfo> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = str::from_utf8(&output.stdout).ok()?;
    let mut pieces = version.split(['.', ' ', '-']);
    if pieces.next() != Some("rustc") {
        return None;
    }
    let _major: u32 = pieces.next()?.parse().ok()?;
    let minor = pieces.next()?.parse().ok()?;
    let _patch: u32 = pieces.next()?.parse().ok()?;
    let toolchain = match pieces.next() {
        Some("beta") => Toolchain::Beta,
        Some("nightly") => Toolchain::Nightly,
        _ => Toolchain::Stable,
    };
    let version = VersionInfo { minor, toolchain };
    Some(version)
}
