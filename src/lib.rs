#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
/*
struct GetArgvOptions {
uint  skip;
pid_t pid;
bool  nuls;
};
struct ArgvArgcResult {
  char*  buffer;
  uint   argc;
  char** argv;
};
struct ArgvResult {
  char* buffer;
  char* start_pointer;
  char* end_pointer;
};

bool print_argv_of_pid(char* start_pointer, char* end_pointer);
bool get_argv_of_pid(const struct GetArgvOptions* options, struct ArgvResult* result);
bool get_argv_and_argc_of_pid(pid_t pid, struct ArgvArgcResult* result);
*/

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
        get_argv_of_pid(&options, &mut result);
        let result_char: c_char = *result.end_pointer;
        assert_eq!(result_char, b'\0' as i8);
        }
    }
}
