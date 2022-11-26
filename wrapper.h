#ifndef LIBGETARGV_H_
#define LIBGETARGV_H_

/*-
 * Copyright: see LICENSE file
 */

#include <sys/types.h>
#if defined(__STDC_VERSION__) && (__STDC_VERSION__ >= 199901L)
  #include <stdbool.h>
#else
typedef enum { false, true } bool;
#endif

struct GetArgvOptions {
  uint  skip;
  pid_t pid;
  bool  nuls;
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

bool print_argv_of_pid(const char* start_pointer, const char* end_pointer);
bool get_argv_of_pid(const struct GetArgvOptions* options, struct ArgvResult* result);
bool get_argv_and_argc_of_pid(pid_t pid, struct ArgvArgcResult* result);

#endif
