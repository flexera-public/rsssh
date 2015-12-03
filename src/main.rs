extern crate cookie;
extern crate docopt;
#[macro_use] extern crate hyper;
extern crate libc;
extern crate rustc_serialize;

use std::env;
use std::io::prelude::*;

use docopt::Docopt;

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
    -h, --help                   show this help
    -c, --config=<config-file>   use alternative config file [default: ~/.ssh/rsssh_config.toml]
    --account=<account-id>       the account ID to use when searching for a host
    --server=<server-name>       name of server or array; can use as a wildcard
    --user=<user>                user to switch to after connect
    --command=<command>          command to run after connect (must include a shell; try suffixing `&& /bin/bash`)
    --email=<email>              email to use to connect to the RightScale API (or set RSSSH_EMAIL)
    --password=<password>        password to use to connect to the RightScale API (or set RSSSH_PASSWORD)
";

const ERROR_MISSING_CREDENTIALS: &'static str = "
Email and password are both required to connect to the RightScale API. Either pass them on
the command line:
    --email=<email> --password=<password>

or set the environment variables RSSSH_EMAIL and RSSSH_PASSWORD.
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
    flag_email: Option<String>,
    flag_password: Option<String>,
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
    let email = default_from_env(args.flag_email, "RSSSH_EMAIL");
    let password = default_from_env(args.flag_password, "RSSSH_PASSWORD");
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

fn default_from_env(value: Option<String>, env_var: &str) -> Option<String> {
    value.or(env::var_os(env_var).and_then(|s| Some(s.to_str().unwrap().to_string())))
}

#[cfg(test)]
mod tests {
    mod default_from_env {
        use super::super::default_from_env;

        #[test]
        fn value_passed() {
            let value = Some("test@example.com".to_string());

            assert_eq!(value.clone(), default_from_env(value, "CARGO_MANIFEST_DIR"));
        }

        #[test]
        fn env_exists() {
            assert_eq!(Some(env!("CARGO_MANIFEST_DIR").to_string()), default_from_env(None, "CARGO_MANIFEST_DIR"));
        }

        #[test]
        fn neither_exists() {
            assert_eq!(None, default_from_env(None, "ENV_VAR_THAT_DOES_NOT_EXIST"));
        }
    }
}
