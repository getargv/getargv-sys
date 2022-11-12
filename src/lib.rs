#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;
    use std::mem;
    use std::os::raw::c_char;
    #[test]
    fn it_works() {
        unsafe {
            let mut result: ArgvResult = mem::zeroed();
            let options: GetArgvOptions = GetArgvOptions {pid: process::id() as pid_t, skip: 0, nuls: false};
            let success = get_argv_of_pid(&options, &mut result);
            let result_char: c_char = *result.end_pointer;
            assert!(success);
            assert_eq!(result_char, b'\0' as i8);
        }
    }
}
