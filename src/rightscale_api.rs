use cookie::CookieJar;
use hyper::client::{Client, RedirectPolicy};
use hyper::header::{Cookie, SetCookie, Location};
use hyper::status::StatusCode;
use rustc_serialize::json;
use std::io::prelude::*;

header! { (XApiVersion, "X-Api-Version") => [String] }
header! { (XAccount, "X-Account") => [i64] }

#[derive(Debug, RustcDecodable)]
struct Server {
    name: String,
    public_ip_addresses: Vec<String>,
}

fn log_in<'a>(url: &str, email: &str, password: &str, account: i64, verbose: bool) -> (String, CookieJar<'a>) {
    let mut client = Client::new();
    let api_15 = XApiVersion("1.5".to_string());
    let params = &format!("email={}&password={}&account_href=/api/accounts/{}", email, password, account);

    if verbose { println!("Logging in to RightScale API at {} with parameters: {:?}", url, params) }

    client.set_redirect_policy(RedirectPolicy::FollowNone);

    let login_response = client.post(url).header(api_15).body(params).send().unwrap();
    let mut cookie_jar = CookieJar::new(b"secret");

    match login_response.status {
        StatusCode::NoContent => {
            login_response.headers.get::<SetCookie>().unwrap().apply_to_cookie_jar(&mut cookie_jar);

            (login_response.url.domain().unwrap().to_string(), cookie_jar)
        },
        StatusCode::Found => match login_response.headers.get::<Location>() {
            Some(location) => log_in(&format!("{}", location), email, password, account, verbose),
            _ => die!("Couldn't find location header for response: {:?}", login_response)
        },
        s => die!("Failed to log in to the RightScale API: {}", s)
    }
}

pub fn find_ips(email: &str, password: &str, account: i64, server: &str, exact_match: bool, verbose: bool) -> Vec<(String, String)> {
    let login_url = "https://my.rightscale.com/api/sessions";
    let client = Client::new();
    let (shard, cookie_jar) = log_in(login_url, email, password, account, verbose);
    let server_name = if exact_match { server.to_string() } else { format!("%25{}%25", server) };
    let api_16 = XApiVersion("1.6".to_string());
    let x_account = XAccount(account);
    let cookie = Cookie::from_cookie_jar(&cookie_jar);
    let url = format!("https://{}/api/instances?filter=name%3D{}%26state%3Doperational", shard, server_name);
    let mut body = String::new();

    if verbose { println!("Finding server: {}", url) }

    let mut response = client.get(&url).header(api_16).header(x_account).header(cookie).send().unwrap();

    let servers: Result<Vec<Server>, _> = match response.status {
        StatusCode::Ok => {
            match response.read_to_string(&mut body) {
                Ok(_) => json::decode(&body),
                Err(e) => {
                    die!("Error reading response from RightScale API: {}", e)
                }
            }
        },
        _ => die!("Unexpected response from RightScale API: {}", response.status)
    };

    if let Ok(servers) = servers {
        servers.iter().
            filter(|s| s.public_ip_addresses.get(0).is_some()).
            map(|s| (s.name.to_string(), s.public_ip_addresses.get(0).unwrap().to_string())).
            collect::<Vec<(String, String)>>()
    } else {
        die!("Error parsing JSON response from RightScale API: {}", servers.err().unwrap());
    }
}
