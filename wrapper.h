#ifndef LIBGETARGV_H_
#define LIBGETARGV_H_

/*-
 * Copyright: see LICENSE file
 */

#ifdef __cplusplus
extern "C" {
#endif

#include <stdio.h>
#include <sys/types.h>

#ifndef __cplusplus
#if defined(__STDC_VERSION__) && (__STDC_VERSION__ >= 199901L)
  #include <stdbool.h>
#else
typedef enum { false, true } bool;
#endif
#endif

struct GetArgvOptions {
  uint      skip;
  pid_t     pid;
  bool      nuls;
};
struct ArgvArgcResult {
  char*  buffer;
  char** argv;
  uint   argc;
};
struct ArgvResult {
  char* buffer;
  char* start_pointer;
  char* end_pointer;
};

int32_t get_arg_exact(pid_t pid);
bool print_argv_of_pid_to(const char* start_pointer, const char* end_pointer, FILE* outstream);
bool get_argv_of_pid_no_malloc(const struct GetArgvOptions* options, struct ArgvResult* retVal, size_t argsize);

bool print_argv_of_pid(const char* start_pointer, const char* end_pointer);
bool get_argv_of_pid(const struct GetArgvOptions* options, struct ArgvResult* result);
bool get_argv_and_argc_of_pid(pid_t pid, struct ArgvArgcResult* result);
void free_ArgvArgcResult(struct ArgvArgcResult* result);
void free_ArgvResult(struct ArgvResult* result);

#ifdef __cplusplus
}
#endif

#endif
