extern crate cookie;
extern crate docopt;
#[macro_use] extern crate hyper;
extern crate libc;
extern crate netrc;
extern crate rustc_serialize;
extern crate toml;

use std::env;
use std::io;
use std::io::prelude::*;

use docopt::Docopt;
use rustc_serialize::Encodable;

#[macro_use] mod die;
mod config;
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
    -e, --exact-match           match the server name exactly, rather than using wildcards at the start and end
    -p, --pick                  when more than one server or array matches, pick one interactively
    --account=<account-id>      the account ID to use when searching for a host
    --server=<server-name>      name of server or array; can use %25 as a wildcard
    --user=<user>               user to switch to after connect
    --command=<command>         command to run after connect (must include a shell; try suffixing `&& /bin/bash`)
    --verbose                   log extra information to standard output

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
    flag_account: Option<i64>,
    flag_server: Option<String>,
    flag_user: Option<String>,
    flag_command: Option<String>,
    flag_exact_match: Option<bool>,
    flag_pick: bool,
    flag_verbose: bool,
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct HostConfig {
    account: Option<i64>,
    server: Option<String>,
    user: Option<String>,
    command: Option<String>,
    exact_match: Option<bool>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.help(false).decode())
        .unwrap_or_else(|e| {println!("{:?}", e); e.exit()});

    let cmd_list = args.cmd_list || (args.arg_host == "list");

    if args.cmd_delete {
        delete(&args.arg_host, &args.flag_config, args.flag_verbose)
    } else if cmd_list {
        list(&args.flag_config, args.flag_verbose)
    } else if args.arg_host != "" {
        connect(args);
    } else {
        println!("{}", USAGE.trim());
    }
}

fn list(config: &str, verbose: bool) {
    let hosts = config::read_config(config, verbose);

    for (key, value) in hosts {
        match toml::decode::<HostConfig>(value.clone()) {
            Some(c) => print_host(key, c),
            None => println!("{} has an invalid config (missing account or server?)", key)
        }
    }
}

fn print_host(key: String, config: HostConfig) {
    fn s(n: &str, x: Option<String>) -> String { x.map(|x| format!(", {}: {}", n, x)).unwrap_or("".to_string()) }

    println!("{}: {{ account: {}{}{}{} }}",
             key, config.account.map(|x| format!("{}", x)).unwrap_or("".to_string()),
             s("server", config.server), s("user", config.user), s("command", config.command))
}

fn delete(host: &str, config_file: &str, verbose: bool) {
    let mut config = config::read_config(config_file, verbose);
    let result = config.remove(host);

    config::write_config(config_file, config, verbose);

    match result.and_then(|result| toml::decode::<HostConfig>(result.clone())) {
        Some(c) => print_host(format!("Host {} removed", host), c),
        None => println!("Host {} removed", host)
    }
}

fn connect(args: Args) {
    let verbose = args.flag_verbose;
    let netrc = config::read_netrc("~/.netrc", verbose);
    let email = override_from_env(netrc.email, "RSSSH_EMAIL");
    let password = override_from_env(netrc.password, "RSSSH_PASSWORD");
    let mut config = config::read_config(&args.flag_config, verbose);
    let read_only_config = config.clone();

    let host_config = read_only_config
        .get(&args.arg_host)
        .and_then(|host| toml::decode::<HostConfig>(host.clone()))
        .unwrap_or(HostConfig { account: None, server: None, user: None, command: None, exact_match: None });

    let account = args.flag_account.or(host_config.account);
    let server = args.flag_server.or(host_config.server);
    let exact_match = args.flag_exact_match.or(host_config.exact_match).unwrap_or(false);

    if let (Some(email), Some(password)) = (email, password) {
        if let (Some(account), Some(server)) = (account, server) {
            let ips = rightscale_api::find_ips(&email, &password, account, &server, exact_match, verbose);
            let ip = pick_ip(ips, args.flag_pick);

            let new_host_config = HostConfig {
                account: Some(account),
                server: Some(server),
                user: args.flag_user.or(host_config.user),
                command: args.flag_command.or(host_config.command),
                exact_match: args.flag_exact_match.or(host_config.exact_match),
            };

            config.remove(&args.arg_host);
            config.insert(args.arg_host, toml::encode(&new_host_config));

            config::write_config(&args.flag_config, config, verbose);

            ssh::ssh_connect(ip, new_host_config.user, new_host_config.command, verbose);
        } else {
            die!(ERROR_ACCOUNT_SERVER_REQUIRED);
        }
    } else {
        die!(ERROR_MISSING_CREDENTIALS);
    }
}

fn pick_ip(ips: Vec<(String, String)>, pick: bool) -> String {
    let index = if pick && ips.len() > 1 {
        for (i, item) in ips.iter().enumerate() {
            println!("{}. {} ({})", i + 1, item.0, item.1);
        }

        println!("");

        get_index(ips.len())
    } else {
        0
    };

    ips.get(index).unwrap().1.clone()
}

fn get_index(max: usize) -> usize {
    let mut input = String::new();

    print!("Enter a number from the list above: ");
    io::stdout().flush().unwrap();

    match io::stdin().read_line(&mut input) {
        Ok(_) => match input.trim().parse::<usize>() {
            Ok(i) => if i > 0 && i <= max { i - 1 } else { get_index(max) },
            Err(_) => get_index(max)
        },
        Err(e) => die!("Error reading input: {}", e)
    }
}

fn override_from_env(value: Option<String>, env_var: &str) -> Option<String> {
    env::var_os(env_var).and_then(|s| Some(s.to_str().unwrap().to_string())).or(value)
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
