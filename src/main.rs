extern crate cookie;
extern crate docopt;
#[macro_use] extern crate hyper;
extern crate libc;
extern crate netrc;
extern crate rustc_serialize;

use std::env;
use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;

use docopt::Docopt;
use netrc::Netrc;

#[macro_use] mod die;
mod rightscale_api;
mod ssh;

const USAGE: &'static str = "
Usage: rsssh [connect] <host> [options]
       rsssh list [--config=<config-file>]
       rsssh delete <host> [--config=<config-file>]
       rsssh (-h | --help)
       rsssh

Options:
    -h, --help                  show this help
    -c, --config=<config-file>  use alternative config file [default: ~/.ssh/rsssh_config.toml]
    --account=<account-id>      the account ID to use when searching for a host
    --server=<server-name>      name of server or array; can use as a wildcard
    --user=<user>               user to switch to after connect
    --command=<command>         command to run after connect (must include a shell; try suffixing `&& /bin/bash`)

To pass credentials for connecting to the RightScale API, either:
1. Create a ~/.netrc file with an entry for the machine 'rsssh'. (Recommended.)
2. Set the environment variables RSSSH_EMAIL and RSSSH_PASSWORD.

The environment variables will override any values set in the netrc file.
";

const ERROR_MISSING_CREDENTIALS: &'static str = "
Email and password are both required to connect to the RightScale API. Either set the
environment variables RSSSH_EMAIL and RSSSH_PASSWORD, or create a ~/.netrc file with an
entry for 'rsssh', like so:
    machine rsssh
      login email@example.com
      password MyPassword

For more information, see:
  http://www.gnu.org/software/inetutils/manual/html_node/The-_002enetrc-file.html
";

const ERROR_ACCOUNT_SERVER_REQUIRED: &'static str = "
Account ID and server name are both required to connect to a host.
";


#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_connect: bool,
    cmd_delete: bool,
    cmd_list: bool,
    arg_host: String,
    flag_help: bool,
    flag_config: String,
    flag_account: Option<i32>,
    flag_server: Option<String>,
    flag_user: Option<String>,
    flag_command: Option<String>,
}

#[derive(Debug)]
struct Credentials {
    email: Option<String>,
    password: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.help(false).decode())
        .unwrap_or_else(|e| {println!("{:?}", e); e.exit()});

    let cmd_list = args.cmd_list || (args.arg_host == "list");

    if args.cmd_delete {
        delete(&args.arg_host, &args.flag_config)
    } else if cmd_list {
        list(&args.flag_config)
    } else if args.arg_host != "" {
        connect(args);
    } else {
        println!("{}", USAGE.trim());
    }
}

fn list(config: &str) {
    println!("list: {}", config);
}

fn delete(host: &str, config: &str) {
    println!("delete: {}, {}", host, config);
}

fn connect(args: Args) {
    let netrc = read_netrc("~/.netrc");
    let email = override_from_env(netrc.email, "RSSSH_EMAIL");
    let password = override_from_env(netrc.password, "RSSSH_PASSWORD");
    let account = args.flag_account;
    let server = args.flag_server;
    let user = args.flag_user;
    let command = args.flag_command;

    if let (Some(email), Some(password)) = (email, password) {
        if let (Some(account), Some(server)) = (account, server) {
            let ip = rightscale_api::find_ip(&email, &password, account, &server);

            ssh::ssh_connect(ip, user, command);
        } else {
            die!(ERROR_ACCOUNT_SERVER_REQUIRED);
        }
    } else {
        die!(ERROR_MISSING_CREDENTIALS);
    }
}

fn override_from_env(value: Option<String>, env_var: &str) -> Option<String> {
    env::var_os(env_var).and_then(|s| Some(s.to_str().unwrap().to_string())).or(value)
}

fn option_string(string: String) -> Option<String> {
    if string.is_empty() {
        None
    } else {
        Some(string)
    }
}

fn find_rsssh_host(hosts: Vec<(String, netrc::Machine)>) -> Option<Credentials> {
    hosts
        .into_iter()
        .find(|host| host.0 == "rsssh")
        .map(|host| Credentials { email: option_string(host.1.login), password: host.1.password })
}

fn read_netrc(path: &str) -> Credentials {
    let file = File::open(expand_home_directory(path));
    let no_credentials = Credentials { email: None, password: None };
    let result = file
        .map(|file| BufReader::new(file))
        .map(|buffer| Netrc::parse(buffer).map(|netrc| netrc.hosts).unwrap_or(Vec::new()))
        .map(find_rsssh_host);

    match result {
        Ok(r) => r.unwrap_or(no_credentials),
        Err(e) => {
            println!("Error finding config: {:?}", e);
            no_credentials
        }
    }
}

fn expand_home_directory(path: &str) -> String {
    match env::var_os("HOME") {
        Some(s) => path.to_string().replace("~", s.to_str().unwrap()),
        None => path.to_string()
    }
}

#[cfg(test)]
mod tests {
    mod override_from_env {
        use super::super::override_from_env;

        #[test]
        fn both_exist() {
            let value = Some("test@example.com".to_string());

            assert_eq!(Some(env!("CARGO_MANIFEST_DIR").to_string()), override_from_env(value, "CARGO_MANIFEST_DIR"));
        }

        #[test]
        fn value_exists() {
            let value = Some("test@example.com".to_string());

            assert_eq!(value.clone(), override_from_env(value, "ENV_VAR_THAT_DOES_NOT_EXIST"));
        }

        #[test]
        fn env_exists() {
            assert_eq!(Some(env!("CARGO_MANIFEST_DIR").to_string()), override_from_env(None, "CARGO_MANIFEST_DIR"));
        }

        #[test]
        fn neither_exists() {
            assert_eq!(None, override_from_env(None, "ENV_VAR_THAT_DOES_NOT_EXIST"));
        }
    }
}
