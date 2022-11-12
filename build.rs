extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let lib_path = option_env!("LIB_PATH").unwrap_or("/usr/local/lib");
    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}",lib_path);
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}",lib_path);

    // Tell cargo to tell rustc to link the system getargv
    // shared library.
    println!("cargo:rustc-link-lib=getargv");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // Tell rust/cargo/bindgen where llvm-config is
    let key = "LLVM_CONFIG_PATH";
    env::set_var(key, env::var(key).unwrap_or_else(|_|"/usr/local/opt/llvm/bin/llvm-config".to_string()));

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
    // The input header we would like to generate
    // bindings for.
        .header("wrapper.h")
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
