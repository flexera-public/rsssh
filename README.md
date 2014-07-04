# rsssh #

<b>R</b>eally <b>s</b>imple <b>s</b>tupid <b>s</b>sh <b>h</b>andler

or:

<b>R</b>ight<b>S</b>cale <b>s</b>ecure <b>sh</b>ell

This is a way of managing host aliases for RightScale servers. When an instance
is destroyed and a new one is created for the same server (or server array), it
uses the RightScale API to get the new IP to connect.

There's a [short Asciinema demo][asciinema] if you want to see it in action.

## Installation ##

Ensure that you have the [`right_api_client`][right_api_client] gem installed in
all Ruby versions you will be running this from. This is a simple script,
designed to be run from anywhere, so it doesn't specify a Ruby version of its
own.

* `git clone git@github.com:rightscale/rsssh.git`
* `cd rsssh`
* `ln -s "$PWD/rsssh" /usr/local/bin`
  * (If you really hate typing, trying symlinking it to `/usr/local/bin/r`.
    That's just 20% of the characters!)

## Usage ##

* Create a new host alias: `rsssh moo-ca-core --add` (or just `rsssh
  moo-ca-core`).
* Connect to an existing host: `rsssh moo-ca-core`.
* Update a host's IP using the RightScale API: `rsssh moo-ca-core --update`.
* `rsssh --help` for more.

## More information ##

This script creates a custom configuration file (`~/.ssh/rsssh_config` by
default) which abuses the [SSH `LocalCommand`][ssh_config] configuration
property to store its metadata. It stores the account ID, deployment name, and
server (or server array) name of the instance to connect to, along with an
optional user to switch to and command to run after connecting. It then disables
the local commands from being run by `ssh` by setting the global
`PermitLocalCommand` property to false.

[asciinema]: https://asciinema.org/a/10632
[right_api_client]: https://github.com/rightscale/right_api_client
[ssh_config]: http://www.openbsd.org/cgi-bin/man.cgi?query=ssh_config&sektion=5
