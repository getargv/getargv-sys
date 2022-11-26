/*-
 * Copyright: see LICENSE file
 */

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

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
            let result_char: c_char = *result.end_pointer;
            assert!(success);
            assert_eq!(result_char, b'\0' as i8);
            let expected = env::args_os().collect::<Vec<_>>().join(OsStr::from_bytes(&[b' ']));
            let actual = CStr::from_ptr(result.start_pointer);
            assert_eq!(expected.to_str().unwrap(), actual.to_str().unwrap());
        }
    }

    #[test]
    fn get_argv_and_argc_of_pid_works() {
        unsafe {
            let mut result: ArgvArgcResult = mem::zeroed();
            assert!(get_argv_and_argc_of_pid(process::id() as pid_t, &mut result));
            assert_eq!(result.argc as usize, env::args_os().count());
            let v = Vec::from_raw_parts(result.argv,result.argc as usize,result.argc as usize);
            for (a,e) in env::args_os().zip(v.iter()) {
                assert_eq!(CStr::from_ptr(*e).to_str().unwrap(), a.to_str().unwrap());
            }
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
        }
    }
}
