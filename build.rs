/*-
 * Copyright: see LICENSE file
 */

extern crate bindgen;

use std::ffi::OsString;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str;

fn homebrew_prefix(path: &str, package: &str)-> PathBuf {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let hb_path = match arch.as_str() {
        "x86_64" => PathBuf::from("/usr/local"),
        "aarch64" => PathBuf::from("/opt/homebrew"),
        _ => panic!("unknown arch {}",arch)
    };
    if hb_path.join(path).exists() {
        hb_path.join(path)
    } else {
        hb_path.join("opt")
            .join(package)
            .join(path)
    }
}

fn macports_prefix(path:&str) -> PathBuf {
    PathBuf::from("/opt/local/").join(path)
}

fn package_prefix(path: &str, package: &str) -> PathBuf {
    if Path::new("/opt/local/bin/port").exists() { macports_prefix(path) }
    else if Path::new("/opt/homebrew/bin/brew").exists() || Path::new("/usr/local/bin/brew").exists() { homebrew_prefix(path, package) }
    else { PathBuf::from(env::var_os("PREFIX").unwrap_or_else(|| OsString::from("/"))).join(path) }
}

fn find_version(lib: &Path) -> String {
    str::from_utf8(
        // &Command::new("vtool")
        // .arg("-show-build")
        &Command::new("otool")
            .arg("-l")
            .arg(lib)
            .output()
            .unwrap()
            .stdout,
    )
        .ok()
        .and_then(|out| {
            out.lines()
                .skip_while(|s| {
                    !(s.contains("LC_VERSION_MIN_MACOSX") || s.contains("LC_BUILD_VERSION"))
                })
                .find(|s| s.contains("minos") || s.contains("version"))
                .and_then(|s| s.split_ascii_whitespace().last().map(|s| s.to_string()))
        })
        .or_else(|| {
            str::from_utf8(
                &Command::new("sw_vers")
                    .arg("-productVersion")
                    .output()
                    .unwrap()
                    .stdout,
            )
                .map(|s| s.to_string())
                .ok()
        })
        .unwrap()
}

fn ensure_apple(){
    if env::var("CARGO_CFG_TARGET_VENDOR").unwrap() != "apple" {
        panic!("The KERN_PROCARGS2 sysctl only exists in xnu kernels, BSD or Linux users should just read /proc/$PID/cmdline which is much easier and faster, Solaris users should use pargs.\nIf you are writing a cross platform program, you can depend on this crate only on macOS by specifying the dependency as:\n[target.'cfg(target_vendor = \"apple\")'.dependencies]\ngetargv = \"{}\"",env!("CARGO_PKG_VERSION"))
    }
}

fn debug_env(){
    env::vars().for_each(|(key, value)| println!("cargo:warning={}={}", key, value));
}

fn locate_llvm_config(){
    let key = "LLVM_CONFIG_PATH";
    env::set_var(
        key,
        env::var_os(key).unwrap_or_else(|| package_prefix("bin/llvm-config","llvm").into_os_string()),
    );
}

fn main() {
    ensure_apple();
    if env::var_os("DEBUG_CARGO_ENV").is_some() {debug_env();}
    let header = "wrapper.h";
    let building_docs = env::var("DOCS_RS").unwrap_or_else(|_| "0".to_string()) == "1";
    if !building_docs {
        let lib_env = "LIBGETARGV_LIB_DIR";
        let lib_path = env::var(lib_env).map(PathBuf::from).unwrap_or_else(|_| package_prefix("lib","getargv"));
        let lib_name = "libgetargv.dylib";
        let lib = lib_path.join(lib_name);
        if !lib.exists() && env::var_os(lib_env).is_some() {
            panic!(
                "Couldn't locate {1} in {0}, check if version is in file name.",
                env::var(lib_env).unwrap(),
                lib_name
            );
        } else if !lib.exists() {
            panic!("Couldn't locate {1}, try setting the {0} env var to the path to the directory in which {1} is located.", lib_env, lib_name);
        }
        // Tell cargo to look for shared libraries in the specified directory
        println!(
            "cargo:rustc-link-search={}",
            lib_path
                .canonicalize()
                .expect("cannot canonicalize path")
                .display()
        );
        // println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path); // this isn't the one that should set rpath, that's the c lib

        // Tell cargo to tell rustc to link the system getargv shared library.
        println!("cargo:rustc-link-lib=getargv");

        // Tell cargo to invalidate the built crate whenever the wrapper changes
        println!("cargo:rerun-if-env-changed={}", lib_env);
        println!("cargo:rerun-if-changed={}", header);

        // Tell rust/cargo/bindgen what macOS this is
        let key = "MACOSX_DEPLOYMENT_TARGET";
        let version = env::var(key).unwrap_or_else(|_| find_version(&lib));
        let version_n = version.parse::<f32>().unwrap();
        env::set_var(key, version);
        println!("cargo:{}={}", key, version_n);

        // pidmax probably not neccesary, i don't think rust really works on 10.5
        println!(
            "cargo:PID_MAX={}",
            if version_n >= 10.6 { 99_999 } else { 30_000 }
        );
    }
    // Tell rust/cargo/bindgen where llvm-config is
    locate_llvm_config();

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
    // The input header we would like to generate
    // bindings for.
        .header(header)
    // Tell cargo to invalidate the built crate whenever any of the
    // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    // Allow emitting desired fns by name
        .allowlist_function(".*_of_pid")
    // Allow emitting desired types by name
        .allowlist_type(".*Argv.*")
    // Don't allow copying structs with pointers, leads to calling free multiple times
        .no_copy(".*Result");

    let bindings = if building_docs {
        bindings.clang_args(["-I", "docs_shim"])
    } else {
        bindings
    }
    // Finish the builder and generate the bindings.
    .generate()
    // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
