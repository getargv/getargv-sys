/*-
 * Copyright: see LICENSE file
 */

#![doc(html_logo_url = "https://getargv.narzt.cam/images/logo.svg")]
#![deny(rustdoc::bare_urls)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::invalid_rust_codeblocks)]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

//! Unsafe Rust bindings for the [getargv library](https://getargv.narzt.cam/).
//!
//! This crate provides unsafe FFI bindings for [libgetargv](https://getargv.narzt.cam/),
//! there is a safe wrapper in the
//! [getargv](https://docs.rs/getargv/latest/getargv/) crate.
//!
//! You almost certainly do not want to use this crate directly.
//!
//! You must have [libgetargv](https://getargv.narzt.cam/) installed for
//! this crate to link to, it will not build/install it for you. If
//! `libgetargv.dylib` is not located in one of `clang`'s default search
//! paths, you must set the`LIBGETARGV_LIB_DIR` env var to tell `rustc`
//! where to find it, and you will either need to set the
//! `DYLD_FALLBACK_LIBRARY_PATH` env var at runtime to tell dyld where
//! to load it from, or you will need to use `install_name_tool` on your
//! binary to fixup the library load path.

use std::mem;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl Default for ArgvResult {
    fn default() -> Self{
        let result: Self = unsafe { mem::zeroed() };
        result
    }
}
impl Default for ArgvArgcResult {
    fn default() -> Self{
        let result: Self = unsafe { mem::zeroed() };
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::mem::forget;
    use std::os::raw::c_char;
    use std::process;
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::ffi::CStr;

    #[test]
    fn get_argv_of_pid_works() {
        unsafe {
            let mut result: ArgvResult = mem::zeroed();
            let options: GetArgvOptions = GetArgvOptions {pid: process::id() as pid_t, skip: 0, nuls: false};
            let success = get_argv_of_pid(&options, &mut result);
            assert!(success);
            let result_char: c_char = *result.end_pointer;
            assert_eq!(result_char, b'\0' as i8);
            let expected = env::args_os().collect::<Vec<_>>().join(OsStr::from_bytes(&[b' ']));
            let actual = CStr::from_ptr(result.start_pointer);
            assert_eq!(expected.to_str().unwrap(), actual.to_str().unwrap());
            free_ArgvResult(&mut result);
        }
    }

    #[test]
    fn get_argv_and_argc_of_pid_works() {
        unsafe {
            let mut result: ArgvArgcResult = mem::zeroed();
            assert!(get_argv_and_argc_of_pid(process::id() as pid_t, &mut result));
            assert_eq!(result.argc as usize, env::args_os().len());
            let v = Vec::from_raw_parts(result.argv,result.argc as usize,result.argc as usize);
            for (a,e) in env::args_os().zip(v.iter()) {
                assert_eq!(CStr::from_ptr(*e).to_str().unwrap(), a.to_str().unwrap());
            }
            free_ArgvArgcResult(&mut result);
            forget(v);//would otherwise try to free argv in result
        }
    }

    #[test]
    fn print_argv_of_pid_works() {
        unsafe {
            let mut result: ArgvResult = mem::zeroed();
            let options: GetArgvOptions = GetArgvOptions {pid: process::id() as pid_t, skip: 0, nuls: false};
            assert!(get_argv_of_pid(&options, &mut result));
            assert!(print_argv_of_pid(
                result.start_pointer,
                result.end_pointer
            ));
            free_ArgvResult(&mut result);
        }
    }

    #[test]
    fn argv_result_default_trait_sanity_test() {
        let zeroed: ArgvResult = unsafe { mem::zeroed() };
        let result: ArgvResult = Default::default();
        assert_eq!(result.buffer, zeroed.buffer);
    }

    #[test]
    fn argv_argc_result_default_trait_sanity_test() {
        let zeroed: ArgvArgcResult = unsafe { mem::zeroed() };
        let result: ArgvArgcResult = Default::default();
        assert_eq!(result.buffer, zeroed.buffer);
    }
}
