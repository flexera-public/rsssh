use cookie::CookieJar;
use hyper::client::Client;
use hyper::header::{Cookie, SetCookie};
use hyper::status::StatusCode;
use rustc_serialize::json::Json;
use std::io::prelude::*;

header! { (XApiVersion, "X-Api-Version") => [String] }
header! { (XAccount, "X-Account") => [i64] }

fn log_in<'a>(email: &str, password: &str, account: i64, verbose: bool) -> CookieJar<'a> {
    let client = Client::new();
    let api_15 = XApiVersion("1.5".to_string());
    let url = "https://my.rightscale.com/api/sessions";
    let params = &format!("email={}&password={}&account_href=/api/accounts/{}", email, password, account);

    if verbose { println!("Logging in to RightScale API at {} with parameters: {}", url, params) }

    let login_request = client.post(url).header(api_15).body(params).send().unwrap();
    let mut cookie_jar = CookieJar::new(b"secret");

    if login_request.status != StatusCode::NoContent {
        die!("Failed to log in to the RightScale API, got response: {}", login_request.status)
    }

    login_request.headers.get::<SetCookie>().unwrap().apply_to_cookie_jar(&mut cookie_jar);
    cookie_jar
}

pub fn find_ip(email: &str, password: &str, account: i64, server: &str, exact_match: bool, verbose: bool) -> String {
    let client = Client::new();
    let cookie_jar = log_in(email, password, account, verbose);
    let server_name = if exact_match { server.to_string() } else { format!("%25{}%25", server) };
    let api_16 = XApiVersion("1.6".to_string());
    let x_account = XAccount(account);
    let cookie = Cookie::from_cookie_jar(&cookie_jar);
    let url = format!("https://my.rightscale.com/api/instances?filter=name%3D{}%26state%3Doperational", server_name);

    if verbose { println!("Finding server: {}", url) }

    let mut find = client.get(&url).header(api_16).header(x_account).header(cookie).send().unwrap();
    let mut body = String::new();

    match find.read_to_string(&mut body) {
        Ok(_) => (),
        Err(e) => die!("Error reading response from RightScale API: {}", e)
    }

    let result = Json::from_str(&body);

    if let Ok(result) = result {
        let first_ip = result.as_array()
            .and_then(|a| a.get(0))
            .and_then(|o| o.as_object())
            .and_then(|o| o.get("public_ip_addresses"))
            .and_then(|a| a.as_array())
            .and_then(|a| a.get(0))
            .and_then(|s| s.as_string());

        match first_ip {
            Some(ip) => ip.to_string(),
            None => die!("Couldn't find server IP. API response: {:?}", result)
        }
    } else {
        die!("Error parsing response from RightScale API: {}", result.err().unwrap());
    }
}
