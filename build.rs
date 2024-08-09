/*-
 * Copyright: see LICENSE file
 */

extern crate bindgen;

use goblin::error::Error;
use goblin::mach::cputype::{CPU_TYPE_ARM64, CPU_TYPE_X86_64};
use goblin::mach::load_command::CommandVariant;
use goblin::mach::{Mach, MachO, SingleArch};
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::env::{self, VarError};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fmt, fs};

fn building_for_darwin() -> bool {
    // build-target is reflected in env
    env::var("CARGO_CFG_TARGET_VENDOR").is_ok_and(|v| v == "apple")
}

fn target_arch() -> Result<String, VarError> {
    env::var("CARGO_CFG_TARGET_ARCH")
}

fn cross_compiling() -> bool {
    // build-host is reflected in cfg!
    cfg!(target_vendor = "apple") ^ building_for_darwin()
    //  note the operator is xor (^) not a ligature of && (∧)
}

fn homebrew_prefix(path: &str, package: &str) -> PathBuf {
    let arch = target_arch();
    let hb_path = match arch.as_deref() {
        Ok("x86_64") => PathBuf::from("/usr/local"),
        Ok("aarch64") => PathBuf::from("/opt/homebrew"),
        _ => panic!("unknown arch {}", arch.unwrap()),
    };
    if hb_path.join(path).exists() {
        hb_path.join(path)
    } else {
        hb_path.join("opt").join(package).join(path)
    }
}

fn macports_prefix(path: &str) -> PathBuf {
    PathBuf::from("/opt/local/").join(path)
}

fn package_prefix(path: &str, package: &str) -> PathBuf {
    if Path::new("/opt/local/bin/port").exists() {
        macports_prefix(path)
    } else if Path::new("/opt/homebrew/bin/brew").exists()
        || Path::new("/usr/local/bin/brew").exists()
    {
        homebrew_prefix(path, package)
    } else {
        PathBuf::from(env::var_os("PREFIX").unwrap_or_else(|| OsString::from("/"))).join(path)
    }
}

#[derive(Eq)]
struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl fmt::Display for Version {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let mao = self.major.cmp(&other.major);
        let mio = self.minor.cmp(&other.minor);
        let pao = self.patch.cmp(&other.patch);
        if mao == Ordering::Equal && mio == Ordering::Equal {
            pao
        } else if mao == Ordering::Equal {
            mio
        } else {
            mao
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s
            .trim()
            .split('.')
            .map(|p| p.parse::<u32>().unwrap())
            .take(3)
            .collect::<VecDeque<u32>>();

        Ok(Self {
            major: parts.pop_front().unwrap(),
            minor: parts.pop_front().unwrap_or(0),
            patch: parts.pop_front().unwrap_or(0),
        })
    }
}

impl From<u32> for Version {
    fn from(packed: u32) -> Self {
        // X.Y.Z is encoded in nibbles xxxx.yy.zz
        // 12.6 = 0b0000_0000_0000_1100_0000_0110_0000_0000
        let major = (packed & 0b1111_1111_1111_1111_0000_0000_0000_0000u32) >> 16;
        let minor = (packed & 0b0000_0000_0000_0000_1111_1111_0000_0000u32) >> 8;
        let patch = (packed & 0b0000_0000_0000_0000_0000_0000_1111_1111u32) >> 0;
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl From<MachO<'_>> for Version {
    fn from(b: MachO) -> Self {
        let packed = b
            .load_commands
            .iter()
            .find_map(|c| match c.command {
                CommandVariant::VersionMinMacosx(v) => Some(v.version),
                CommandVariant::BuildVersion(v) => Some(v.minos),
                _ => None,
            })
            .unwrap();
        Self::from(packed)
    }
}

fn find_version(lib: &Path) -> Version {
    match Mach::parse(&fs::read(lib).map_err(goblin::error::Error::IO).unwrap()).unwrap() {
        Mach::Binary(b) => Version::from(b),
        Mach::Fat(f) => {
            match f
                .find(|r| {
                    r.unwrap().cputype
                        == match target_arch().as_deref() {
                            Ok("x86_64") => CPU_TYPE_X86_64,
                            Ok("aarch64") => CPU_TYPE_ARM64,
                            _ => panic!("unknown arch"),
                        }
                })
                .unwrap()
                .ok()
                .unwrap()
            {
                SingleArch::MachO(b) => Version::from(b),
                SingleArch::Archive(_) => panic!("lib is an archive?"),
            }
        }
    }
}

fn ensure_apple() {
    if !building_for_darwin() {
        panic!("The KERN_PROCARGS2 sysctl only exists in xnu kernels, BSD or Linux users should just read /proc/$PID/cmdline which is much easier and faster, Solaris users should use pargs.\nIf you are writing a cross platform program, you can depend on this crate only on macOS by specifying the dependency as:\n[target.'cfg(target_vendor = \"apple\")'.dependencies]\n{} = \"{}\"",env!("CARGO_PKG_NAME"),env!("CARGO_PKG_VERSION"))
    }
}

fn debug_env() {
    env::vars().for_each(|(key, value)| println!("cargo::warning={}={}", key, value));
}

fn locate_llvm_config() {
    let key = "LLVM_CONFIG_PATH";
    env::set_var(
        key,
        env::var_os(key)
            .unwrap_or_else(|| package_prefix("bin/llvm-config", "llvm").into_os_string()),
    );
}

fn main() {
    ensure_apple();
    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    if env::var_os("DEBUG_CARGO_ENV").is_some() {
        debug_env();
    }
    let header = "wrapper.h";
    let building_docs = env::var("DOCS_RS").unwrap_or_else(|_| "0".to_string()) == "1";
    if !building_docs {
        let lib_env = "LIBGETARGV_LIB_DIR";
        let lib_path = env::var(lib_env)
            .map(PathBuf::from)
            .unwrap_or_else(|_| package_prefix("lib", "getargv"));
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
            "cargo::rustc-link-search={}",
            lib_path
            .canonicalize()
            .expect("cannot canonicalize path")
            .display()
        );
        // println!("cargo::rustc-link-arg=-Wl,-rpath,{}", lib_path); // this isn't the one that should set rpath, that's the c lib

        // Tell cargo to tell rustc to link the system getargv shared library.
        println!("cargo::rustc-link-lib=getargv");

        // Tell cargo to invalidate the built crate whenever the wrapper changes
        println!("cargo::rerun-if-env-changed={}", lib_env);
        println!("cargo::rerun-if-changed={}", header);

        // Tell rust/cargo/bindgen what macOS this is
        let key = "MACOSX_DEPLOYMENT_TARGET";
        let version = env::var(key)
            .map(|s| s.parse::<Version>().unwrap())
            .unwrap_or_else(|_| find_version(&lib));
        println!("cargo::rerun-if-env-changed={}", key);
        println!("cargo::rustc-env={}={}", key, version);
        println!("cargo::metadata={}={}", key, version);

        // pidmax probably not neccesary, i don't think rust really works on 10.5
        println!(
            "cargo::metadata=PID_MAX={}",
            if version
            >= (Version {
                major: 10,
                minor: 6,
                patch: 0
            })
            {
                99_999
            } else {
                30_000
            }
        );
    }
    // Tell rust/cargo/bindgen where llvm-config is
    locate_llvm_config();

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let builder = if building_docs {
        bindgen::Builder::default().clang_args(["-I", "docs_shim"])
    } else {
        bindgen::Builder::default()
    }
        // The input header we would like to generate
        // bindings for.
        .header(header)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Allow emitting desired fns by name
        .allowlist_function(".*_of_pid|free_Argv.*")
        // Allow emitting desired types by name
        .allowlist_type(".*Argv.*")
        // Don't allow copying structs with pointers, leads to calling free multiple times
        .no_copy(".*Result");

    // Finish the builder and generate the bindings.
    match builder.generate() {
        Ok(bindings) => bindings,
        Err(e) => match e {
            bindgen::BindgenError::ClangDiagnostic(s) if cross_compiling() && s.contains("file not found") && s.split('\'').nth(1).is_some() => {
                panic!("Clang could not find '{}', perhaps you need to set the 'BINDGEN_EXTRA_CLANG_ARGS' env var to something like: '--sysroot=/path/to/macos/sysroot'",
                    s.split('\'').nth(1).unwrap()
                )
            }
            _ => {
                // All other errors, or if parsing fails
                panic!("Unable to generate bindings: {}", e)
            }
        },
    }
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
