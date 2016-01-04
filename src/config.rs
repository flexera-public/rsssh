use std::collections::BTreeMap;
use std::env;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;

use netrc;
use toml;

#[derive(Debug)]
pub struct Credentials {
    pub email: Option<String>,
    pub password: Option<String>,
}

fn expand_home_directory(path: &str) -> String {
    if path == "" { return path.to_string(); }

    let (prefix, rest) = path.split_at(1);
    let home_dir = env::home_dir();

    if prefix == "~" {
        let expanded_prefix = home_dir.as_ref().and_then(|s| s.to_str()).unwrap_or("~");

        [expanded_prefix, rest].concat()
    } else {
        path.to_string()
    }
}

pub fn read_config(path: &str) -> toml::Table {
    fn config_file_to_string(path: &str) -> Result<String, io::Error> {
        let mut file = try!(File::open(expand_home_directory(path)));
        let mut buffer = String::new();

        try!(file.read_to_string(&mut buffer));

        Ok(buffer)
    }

    let config_string = config_file_to_string(path);
    let empty_config: toml::Table = BTreeMap::new();

    match config_string {
        Ok(r) => toml::Parser::new(&*r).parse().unwrap_or(empty_config),
        Err(e) => {
            println!("Error reading config file {}: {:?}", path, e);
            empty_config
        }
    }
}

pub fn write_config(path: &str, config: toml::Table) {
    let file = File::create(expand_home_directory(path));
    let result = file.map(|mut file| file.write_all(&toml::encode_str(&config).into_bytes()));

    match result {
        Ok(_) => (),
        Err(e) => die!("Error writing config {}: {:?}", path, e)
    }
}

pub fn read_netrc(path: &str) -> Credentials {
    let option_string = |s: String| if s.is_empty() { None } else { Some(s) };
    let file = File::open(expand_home_directory(path));
    let no_credentials = Credentials { email: None, password: None };
    let result = file
        .map(|file| BufReader::new(file))
        .map(|buffer| netrc::Netrc::parse(buffer).map(|netrc| netrc.hosts).unwrap_or(Vec::new()))
        .map(|hosts| {
            hosts.into_iter().find(|host| host.0 == "rsssh").map(|host| {
                Credentials { email: option_string(host.1.login), password: host.1.password }
            })
        });

    match result {
        Ok(r) => r.unwrap_or(no_credentials),
        Err(e) => {
            println!("Error reading netrc file {}: {:?}", path, e);
            no_credentials
        }
    }
}

#[cfg(test)]
mod tests {
    mod expand_home_directory {
        use super::super::expand_home_directory;
        use std::env;

        fn home_dir() -> String {
            env::home_dir().unwrap().to_str().unwrap().to_string()
        }

        #[test]
        fn expand_no_path() {
            assert_eq!("".to_string(), expand_home_directory(""))
        }

        #[test]
        fn expand_tilde_prefix() {
            assert_eq!(home_dir() + "/.ssh/rsssh_config.toml",
                       expand_home_directory("~/.ssh/rsssh_config.toml"))
        }

        #[test]
        fn expand_tilde_suffix() {
            assert_eq!("/.ssh/rsssh_config.toml~".to_string(),
                       expand_home_directory("/.ssh/rsssh_config.toml~"))
        }

        #[test]
        fn expand_tilde_tilde() {
            assert_eq!(home_dir() + "~", expand_home_directory("~~"))
        }

        #[test]
        fn expand_no_tilde() {
            assert_eq!("/home/sean/.ssh/rsssh_config.toml".to_string(),
                       expand_home_directory("/home/sean/.ssh/rsssh_config.toml"))
        }
    }
}
