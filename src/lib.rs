#[macro_use]
extern crate lazy_static;

pub mod wapp;

use headless_chrome::protocol::cdp::Network::{GetCookies, GetResponseBodyReturnObject};
use headless_chrome::{Browser, Tab};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use url::Url;
use wapp::{RawData, Tech};

const DEFAULT_BROWSER_WS_ENDPOINT: &str = "ws://190.102.43.107:9222";

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub url: String,
    pub result: Result<HashSet<Tech>, String>,
    pub scan_time: Option<Duration>,
}

/// Possible Errors in the domain_info lib
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WappError {
    Fetch(String),
    Analyze(String),
    Other(String),
}

impl fmt::Display for WappError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WappError::Fetch(err) => format!("Fetch/{}", err),
                WappError::Analyze(err) => format!("Analyze/{}", err),
                WappError::Other(err) => format!("Other/{}", err),
            }
        )
    }
}

impl std::convert::From<std::io::Error> for WappError {
    fn from(err: std::io::Error) -> Self {
        WappError::Other(err.to_string())
    }
}
impl From<&dyn std::error::Error> for WappError {
    fn from(err: &dyn std::error::Error) -> Self {
        WappError::Other(err.to_string())
    }
}

// the trait `std::convert::From<std::str::Utf8Error>` is not implemented for `WappError`
impl From<std::str::Utf8Error> for WappError {
    fn from(err: std::str::Utf8Error) -> Self {
        WappError::Other(err.to_string())
    }
}

use std::time::{Duration, Instant};

pub async fn scan(url: Url, with_timing: Option<bool>) -> Analysis {
    let url_str = url.as_str().to_string();

    let start = match with_timing {
        Some(true) => Some(Instant::now()),
        _ => None,
    };

    let analysis = match fetch(url).await {
        Some(raw_data) => {
            let result: HashSet<Tech> =
                wapp::check(raw_data).await.into_iter().collect();

            Analysis {
                url: url_str,
                result: Ok(result),
                scan_time: start.map(|s| s.elapsed()),
            }
        }
        None => Analysis {
            url: url_str,
            result: Err("Error".to_string()),
            scan_time: start.map(|s| s.elapsed()),
        },
    };

    analysis
}


fn get_html(tab: &Tab) -> Option<String> {
    let remote_object = tab
        .evaluate("document.documentElement.outerHTML", false)
        .ok()?;

    let json = remote_object.value?;
    let str = json.as_str()?;

    Some(str.to_owned())
}

async fn fetch(url: Url) -> Option<Arc<wapp::RawData>> {
    let browser_ws_endpoint = std::env::var("BROWSER_WS_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_BROWSER_WS_ENDPOINT.to_string());
    ensure_remote_tab(&browser_ws_endpoint);
    let browser = Browser::connect(browser_ws_endpoint).ok()?;

    let tab = browser.wait_for_initial_tab().ok()?;

    let responses = Arc::new(Mutex::new(Vec::new()));
    let responses2 = responses.clone();

    tab.enable_response_handling(Box::new(move |response, fetch_body| {
        let body = fetch_body().unwrap_or(GetResponseBodyReturnObject {
            body: "".to_string(),
            base_64_encoded: false,
        });
        responses2.lock().unwrap().push((response, body));
    }))
        .unwrap();
    tab.navigate_to(url.as_str()).ok()?;

    let rendered_tab = tab.wait_until_navigated().ok()?;

    let html = get_html(rendered_tab).unwrap();

    let final_responses: Vec<_> = responses.lock().unwrap().clone();

    let headers: HashMap<String, String> = final_responses
        .into_iter()
        .nth(0)
        .unwrap()
        .0
        .response
        .headers
        .0
        .unwrap()
        .as_object()
        .unwrap()
        .clone()
        .into_iter()
        .map(|(a, b)| (a.to_lowercase(), b.to_string().replace("\"", "")))
        .collect();
    // Revisiting since cookies aren't always detected on first tab.
    let cookies: Vec<wapp::Cookie> = tab
        .navigate_to(url.as_str())
        .ok()?
        .get_cookies()
        .ok()?
        .into_iter()
        .map(|c| wapp::Cookie {
            name: c.name,
            value: c.value,
        })
        .collect();
    //let cookies: Vec<wapp::Cookie> = vec![wapp::Cookie {name: "a".to_string(), value: "value".to_string()}];

    let parsed_html = Html::parse_fragment(&html);
    let selector = Selector::parse("meta").unwrap();
    let mut script_tags = vec![];
    for js in parsed_html.select(&Selector::parse("script").ok()?) {
        script_tags.push(js.html());
    }

    // Note: using a hashmap will not support two meta tags with the same name and different values,
    // though I'm not sure if that's legal html.
    let mut meta_tags = HashMap::new();
    for meta in parsed_html.select(&selector) {
        if let (Some(name), Some(content)) =
        (meta.value().attr("name"), meta.value().attr("content"))
        {
            // eprintln!("META {} -> {}", name, content);
            meta_tags.insert(String::from(name), String::from(content));
        }
    }
    let raw_data = Arc::new(RawData {
        headers,
        cookies,
        meta_tags,
        script_tags,
        html,
    });

    Some(raw_data)
}

fn ensure_remote_tab(browser_ws_endpoint: &str) {
    let url = match Url::parse(browser_ws_endpoint) {
        Ok(url) => url,
        Err(_) => return,
    };
    let host = match url.host_str() {
        Some(host) => host,
        None => return,
    };
    let port = match url.port_or_known_default() {
        Some(port) => port,
        None => return,
    };
    let request = format!(
        "GET /json/new HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    );
    let mut stream = match TcpStream::connect((host, port)) {
        Ok(stream) => stream,
        Err(_) => return,
    };
    if stream.write_all(request.as_bytes()).is_err() {
        return;
    }
    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response);
}
