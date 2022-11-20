#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

/*-
 * Copyright: see LICENSE file
 */

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;
    use std::mem;
    use std::env;
    use std::os::raw::c_char;
    use std::io;
    use std::fs::File;
    use std::os::unix::io::{FromRawFd, AsRawFd};
    use std::io::prelude::*;
    use std::ffi::OsString;

    #[test]
    fn get_argv_of_pid_works() {
        unsafe {
            let mut result: ArgvResult = mem::zeroed();
            let options: GetArgvOptions = GetArgvOptions {pid: process::id() as pid_t, skip: 0, nuls: false};
            let success = get_argv_of_pid(&options, &mut result);
            let result_char: c_char = *result.end_pointer;
            assert!(success);
            assert_eq!(result_char, b'\0' as i8);
        }
    }

    #[test]
    fn get_argv_and_argc_of_pid_works() {
        unsafe {
            let mut result: ArgvArgcResult = mem::zeroed();
            assert!(get_argv_and_argc_of_pid(process::id() as pid_t, &mut result));
            assert_eq!(result.argc as usize, env::args_os().count());
        }
    }

    #[ignore]
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

            let mut f = File::from_raw_fd(io::stdout().as_raw_fd());
            let mut output = String::new();
            //f.read_to_string(&mut output); // hangs forever

            assert_eq!(env::args_os().reduce(|mut a, s| { a.push(" "); a.push(s); a }).unwrap(),Into::<OsString>::into(output));
        }
    }
}
