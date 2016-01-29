# rsssh #

[![Build status](https://travis-ci.org/rightscale/rsssh.svg)](https://travis-ci.org/rightscale/rsssh)

<b>R</b>eally <b>s</b>imple <b>s</b>tupid <b>s</b>sh <b>h</b>andler

or:

<b>R</b>ight<b>S</b>cale <b>s</b>ecure <b>sh</b>ell

```shell
$ rsssh myserver --account=1234 --server=myserver
Warning: Permanently added '1.2.3.4' (ECDSA) to the list of known hosts.
Creating your user profile (myuser) on this machine.
Welcome to Ubuntu 14.04.3 LTS (GNU/Linux 3.13.0-46-generic x86_64)

myuser@myserver:~$ exit
logout
Connection to 1.2.3.4 closed.

# Relaunch that server so that it gets a different IP, and run again:
$ rsssh myserver
Warning: Permanently added '5.6.7.8' (ECDSA) to the list of known hosts.
Creating your user profile (myuser) on this machine.
Welcome to Ubuntu 14.04.3 LTS (GNU/Linux 3.13.0-46-generic x86_64)

myuser@myserver:~$ exit
logout
Connection to 5.6.7.8 closed.
```

This is a way of managing host aliases for [RightScale servers][rs]. Every time you run
`rsssh $host`, it finds the host using the [RightScale API][rsapi] and connects to it via
`ssh`.

## Installation ##

Binaries for OS X and Linux are available from the [GitHub release][release]. Download,
rename to just `rsssh`, and place somewhere on the `PATH`.

To build from source, see [building](#building).

## Usage ##

* Create a new host alias, or update an existing alias: `rsssh myserver --server=myserver
  --account=1234`
* Connect to an existing host: `rsssh myserver`
* List all aliases stored in the config file: `rsssh list`.
* `rsssh --help` for more.

### Finding a server ###

The [Instances#index][ii] action is used to find all servers matching the name given. By
default, `rsssh` adds wildcards to the beginning and end of the server name, and picks the
first server in the response (the order of which is not guaranteeed).

To only match the server name as specified, without the leading and trailing wildcards,
use the `--exact-match` option.

To add a wildcard to a server name, use `%25` (the URL-encoded form of the percent sign).

### User switching ###

Passing the `--user=web` flag will automatically switch to the user 'web' after logging
in, and launch their shell (in this example, it will run `sudo -u "web" -s`).

The default user used for connecting to hosts via `ssh` is 'rightscale'.

### Running a command on launch ###

The `--command='cd /var/log && /bin/bash` flag runs a command after logging in. If this
command terminates immediately, the `ssh` session will end immediately. The command passed
will be run as the user passed in the [`--user`](#user-switching) flag, if both are
present.

### `ssh` options used ###

`rsssh` sets the following `ssh` options:
- `-t` - to create a TTY, when running a command (that may launch a shell).
- `-o StrictHostKeychecking=no` - because instances are relaunched frequently with
  different host keys.
- `-o UserKnownHostsFile=/dev/null` - for the same reason.

*Bonus option*: if you really hate typing, trying creating an alias to use this with `r`
(assuming you don't already use [R][r]). That's just 20% of the characters!

## Building ##

There's a [`Makefile`](Makefile)! Here are the targets:
- `make build` - create a release build.
- `make run` - create a debug build and run it. To pass arguments, use `cargo run --
  $args` instead.
- `make test` - run the unit tests. (There aren't many of these at the moment, because
  pretty much everything this does is interface with remote services.)
- `make install [--prefix=~/bin]` installs to the prefix (defaults to `/usr/local/bin`).
- `make uninstall [--prefix=~/bin]` remove from the prefix (defaults to `/usr/local/bin`).

## Maintained by ##

[Sean McGivern](https://github.com/smcgivern)

[rs]: http://docs.rightscale.com/cm/rs101/story_of_a_rightscale_server.html
[array]: http://docs.rightscale.com/cm/dashboard/manage/arrays/arrays.html
[rsapi]: http://reference.rightscale.com/api1.6/
[r]: https://www.r-project.org/
[ii]: http://reference.rightscale.com/api1.6/#/1.6/controller/V1_6-Instances
[rust]: https://www.rust-lang.org/downloads.html
[release]: https://github.com/rightscale/rsssh/releases/latest
