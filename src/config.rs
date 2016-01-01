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
    match env::var_os("HOME") {
        Some(s) => path.to_string().replace("~", s.to_str().unwrap()),
        None => path.to_string()
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

    let file = File::open(expand_home_directory(path));
    let no_credentials = Credentials { email: None, password: None };
    let result = file
        .map(|file| BufReader::new(file))
        .map(|buffer| netrc::Netrc::parse(buffer).map(|netrc| netrc.hosts).unwrap_or(Vec::new()))
        .map(find_rsssh_host);

    match result {
        Ok(r) => r.unwrap_or(no_credentials),
        Err(e) => {
            println!("Error reading netrc file {}: {:?}", path, e);
            no_credentials
        }
    }
}
