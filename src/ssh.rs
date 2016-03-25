use libc::{c_char, execvp};
use std::io::prelude::*;
use std::ffi::CString;
use std::ptr;

macro_rules! c_ptr {
    ($x:expr) => {{ CString::new($x).unwrap().as_ptr() }};
}

fn exec_ssh(ip: &str, command: &str, verbose: bool) {
    let user_host: &str = &format!("rightscale@{}", ip);
    let ssh_command = format!("ssh -t -o StrictHostKeychecking=no -o UserKnownHostsFile=/dev/null {} \"{}\"", user_host, command);

    if verbose { println!("Running {}", ssh_command) }

    let argv: &[*const c_char] = &[c_ptr!("ssh"),
                                   c_ptr!("-t"),
                                   c_ptr!("-o"),
                                   c_ptr!("StrictHostKeyChecking=no"),
                                   c_ptr!("-o"),
                                   c_ptr!("UserKnownHostsFile=/dev/null"),
                                   c_ptr!(user_host),
                                   c_ptr!(command),
                                   ptr::null()];

    unsafe { execvp(argv[0], &argv[0]); }

    die!("ssh command failed: {}", ssh_command);
}

fn ssh_command_arg(user: Option<String>, command: Option<String>) -> String {
    let user_prefix = user.and_then(|u| Some(format!("sudo -u \"{}\"", u)));
    let escaped_command = command.and_then(|c| Some(c.replace("\"", "\\\"")));

    match user_prefix {
        Some(u) =>
            match escaped_command {
                Some(c) => format!("{} -- sh -cl \"{}\"", u, c),
                None => format!("{} -s", u)
            },
        None => escaped_command.unwrap_or("".to_string())
    }
}

pub fn ssh_connect(ip: String, user: Option<String>, command: Option<String>, verbose: bool) {
    exec_ssh(&ip, &ssh_command_arg(user, command), verbose);
}

#[cfg(test)]
mod tests {
    mod ssh_command_arg {
        use super::super::ssh_command_arg;

        #[test]
        fn no_user() {
            assert_eq!("pwd".to_string(),
                       ssh_command_arg(None, Some("pwd".to_string())));
        }

        #[test]
        fn no_command() {
            assert_eq!("sudo -u \"sean\" -s".to_string(),
                       ssh_command_arg(Some("sean".to_string()), None));
        }

        #[test]
        fn user_and_command() {
            assert_eq!("sudo -u \"sean\" -- sh -cl \"cd / && /bin/bash\"".to_string(),
                       ssh_command_arg(Some("sean".to_string()), Some("cd / && /bin/bash".to_string())));
        }
    }
}
