use cookie::CookieJar;
use hyper::client::Client;
use hyper::header::{Cookie, SetCookie};
use hyper::status::StatusCode;
use rustc_serialize::json::Json;
use std::io::prelude::*;

header! { (XApiVersion, "X-Api-Version") => [String] }
header! { (XAccount, "X-Account") => [i64] }

fn log_in<'a>(email: &str, password: &str, account: i64) -> CookieJar<'a> {
    let client = Client::new();
    let mut cookie_jar = CookieJar::new(b"secret");

    let login_request = client
        .post("https://my.rightscale.com/api/sessions")
        .header(XApiVersion("1.5".to_string()))
        .body(&format!("email={}&password={}&account_href=/api/accounts/{}",
                       email, password, account))
        .send()
        .unwrap();

    if login_request.status != StatusCode::NoContent {
        die!("Failed to log in to the RightScale API, got response: {}", login_request.status)
    }

    login_request.headers.get::<SetCookie>().unwrap().apply_to_cookie_jar(&mut cookie_jar);
    cookie_jar
}

pub fn find_ip(email: &str, password: &str, account: i64, server: &str, exact_match: bool) -> String {
    let client = Client::new();
    let cookie_jar = log_in(email, password, account);
    let server_name = if exact_match { server.to_string() } else { format!("%25{}%25", server) };
    let mut body = String::new();

    let mut find = client
        .get(&format!("https://my.rightscale.com/api/instances?filter=name%3D{}%26state%3Doperational",
                      server_name))
        .header(XApiVersion("1.6".to_string()))
        .header(XAccount(account))
        .header(Cookie::from_cookie_jar(&cookie_jar))
        .send()
        .unwrap();

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
