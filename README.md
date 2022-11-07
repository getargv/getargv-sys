<h1><img src="logo.svg" width="200" alt="getargv"></h1>

[![C/make CI](https://github.com/getargv/getargv/actions/workflows/actions.yml/badge.svg?event=push)](https://github.com/getargv/getargv/actions/workflows/actions.yml)

`getargv` is a tool to get the arguments that were passed to another running process. It is intended to provide roughly the same functionality as `/proc/<pid>/cmdline` on linux. On macOS you only have `ps -o args= <pid>` available to you, which is space separated and therefore not guaranteed to parse the same way as when it was originally passed to the process, as arguments could have contained escaped or quoted spaces, or even empty arguments.

## Permissions

`getargv` can only see your processes by default, which is often sufficient. If you need to see the arguments of a process owned by another user, run `sudo getargv <pid>`.

## Use cases

 - `ssh` does not provide the remote command passed on the command line as a token in the `~/.ssh/config`, `getargs` allows you to create a condition which checks if a remote command was specified on the cli, and if not provide a default command.
 - GitHub forces all users to `ssh` in using the `git` username, so there is normally no way for `ssh` to differentiate which account you are using. `getargv` allows you to check.

## System Requirements

Your system must support `sysctl` and `KERN_PROCARGS2`, which probably means macOS [10.3](https://github.com/CamJN/xnu/blob/b52f6498893f78b034e2e00b86a3e146c3720649/bsd/sys/sysctl.h#L332) or later, though I haven't tested older versions. You'll also need a non-ancient clang (c99 is required) or you'll have to override the compiler flags with `CC`, `EXTRA_CPPFLAGS`, and `EXTRA_CFLAGS`.

## Building

Clone the repo and run `make`.

I've built getargv on macOS 10.7-12.0, using only the CLT package, not the full Xcode install. If you need to override variables, do so after the `make` command, eg: `make EXTRA_CPPFLAGS=-DMACRO EXTRA_CFLAGS=-std=c17`. If you are trying to build on a version of macOS earlier than 10.7, let me know how it goes.

## Installing

Run `make install`, installs to the `/usr/local/` prefix by default, change with the `PREFIX` `make` variable.

## Testing

Run `make && make -C test`.

I've tested getargv on macOS 10.7-12.0, and run CI against 10.15 & 11.0.
