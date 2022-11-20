/*-
 * Copyright: see LICENSE file
 */

extern crate bindgen;

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str;

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
    ).ok().and_then(|out|
                    out.lines()
                    .skip_while(|s| !(s.contains("LC_VERSION_MIN_MACOSX") || s.contains("LC_BUILD_VERSION")))
                    .find(|s| s.contains("minos") || s.contains("version"))
                    .and_then(|s| s.split_ascii_whitespace().last().map(|s| s.to_string()))
    ).or_else(|| {
        str::from_utf8(
            &Command::new("sw_vers")
                .arg("-productVersion")
                .output()
                .unwrap()
                .stdout,
        )
            .map(|s| s.to_string())
            .ok()
    }).unwrap()
}

fn main() {
    if cfg!(not(target_vendor = "apple")) {
        panic!("The KERN_PROCARGS2 sysctl only exists in xnu kernels, linux users should just read /proc/$PID/cmdline which is much easier and faster.")
    }

    let lib_env = "LIBGETARGV_LIB_DIR";
    let lib_path = env::var(lib_env).unwrap_or_else(|_| "/usr/local/lib".to_string());
    let lib_name = "libgetargv.dylib";
    let lib = Path::new(&lib_path).join(lib_name);
    if !lib.exists() {
        panic!("Couldn't locate {1}, try setting the {0} env var to the path to the directory in which {1} is located.", lib_env, lib_name);
    }
    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", lib_path);
    // println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path); // this isn't the one that should set rpath, that's the c lib

    // Tell cargo to tell rustc to link the system getargv
    // shared library.
    println!("cargo:rustc-link-lib=getargv");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-env-changed={}", lib_env);
    let header = "wrapper.h";
    println!("cargo:rerun-if-changed={}", header);

    // Tell rust/cargo/bindgen where llvm-config is
    let key = "LLVM_CONFIG_PATH";
    env::set_var(
        key,
        env::var(key).unwrap_or_else(|_| "/usr/local/opt/llvm/bin/llvm-config".to_string()),
    );

    // Tell rust/cargo/bindgen what macOS this is
    let key = "MACOSX_DEPLOYMENT_TARGET";
    let version = env::var(key).unwrap_or_else(|_|find_version(&lib));
    let version_n = version.parse::<f32>().unwrap();
    env::set_var(key, version);
    println!("cargo:{}={}", key, version_n);

    // pidmax probably not neccesary, i don't think rust really works on 10.5
    println!("cargo:PID_MAX={}", if version_n >= 10.6 { 99_999 } else { 30_000 });

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
