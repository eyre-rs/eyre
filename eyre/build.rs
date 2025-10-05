use std::{
    env, fs,
    path::Path,
    process::{Command, ExitStatus},
};

fn main() {
    println!("cargo:rustc-check-cfg=cfg(nightly)");
    println!("cargo:rustc-check-cfg=cfg(track_caller)");
    println!("cargo:rustc-check-cfg=cfg(generic_member_access)");
    println!("cargo:rustc-check-cfg=cfg(eyre_no_fmt_args_capture)");
    println!("cargo:rustc-check-cfg=cfg(backtrace)");
    println!("cargo:rustc-check-cfg=cfg(eyre_no_fmt_arguments_as_str)");
    println!("cargo:rustc-check-cfg=cfg(doc_cfg)");
    let ac = autocfg::new();

    // https://github.com/rust-lang/rust/issues/99301 [nightly]
    //
    // Autocfg does currently not support custom probes, or `nightly` only features
    match compile_probe(GENERIC_MEMBER_ACCESS_PROBE) {
        Some(status) if status.success() => autocfg::emit("generic_member_access"),
        _ => {}
    }

    // https://github.com/rust-lang/rust/issues/47809 [rustc-1.46]
    if ac.probe_rustc_version(1, 46) {
        autocfg::emit("track_caller");
    }

    if ac.probe_rustc_version(1, 52) {
        autocfg::emit("eyre_no_fmt_arguments_as_str");
    }

    if ac.probe_rustc_version(1, 58) {
        autocfg::emit("eyre_no_fmt_args_capture");
    }

    #[cfg(feature = "std")]
    if ac.probe_rustc_version(1, 65) {
        autocfg::emit("backtrace")
    }
}

// This code exercises the surface area or the generic member access feature for the `std::error::Error` trait.
//
// This is use to detect and supply backtrace information through different errors types.
const GENERIC_MEMBER_ACCESS_PROBE: &str = r#"
    #![feature(error_generic_member_access)]
    #![allow(dead_code)]

    use std::error::{Error, Request};
    use std::fmt::{self, Display};

    #[derive(Debug)]
    struct E { 
        backtrace: MyBacktrace,
    }

    #[derive(Debug)]
    struct MyBacktrace;

    impl Display for E {
        fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }

    impl Error for E {
        fn provide<'a>(&'a self, request: &mut Request<'a>) {
            request
                .provide_ref::<MyBacktrace>(&self.backtrace);
        }
    }
"#;

fn compile_probe(probe: &str) -> Option<ExitStatus> {
    let rustc = env::var_os("RUSTC")?;
    let out_dir = env::var_os("OUT_DIR")?;
    let probefile = Path::new(&out_dir).join("probe.rs");
    fs::write(&probefile, probe).ok()?;

    let rustc_wrapper = env::var_os("RUSTC_WRAPPER").filter(|wrapper| !wrapper.is_empty());
    let rustc_workspace_wrapper =
        env::var_os("RUSTC_WORKSPACE_WRAPPER").filter(|wrapper| !wrapper.is_empty());
    let mut rustc = rustc_wrapper
        .into_iter()
        .chain(rustc_workspace_wrapper)
        .chain(std::iter::once(rustc));

    let mut cmd = Command::new(rustc.next().unwrap());
    cmd.args(rustc);

    if let Some(target) = env::var_os("TARGET") {
        cmd.arg("--target").arg(target);
    }

    // If Cargo wants to set RUSTFLAGS, use that.
    if let Ok(rustflags) = env::var("CARGO_ENCODED_RUSTFLAGS") {
        if !rustflags.is_empty() {
            for arg in rustflags.split('\x1f') {
                cmd.arg(arg);
            }
        }
    }

    cmd.arg("--edition=2018")
        .arg("--crate-name=eyre_build")
        .arg("--crate-type=lib")
        .arg("--emit=metadata")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(probefile)
        .status()
        .ok()
}
