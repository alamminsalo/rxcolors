extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate serde_json;

use futures::{Future, Stream};
use hyper::{Method, Request, Client, Chunk, StatusCode};
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use serde_json::{Value};
use std::env;
use std::str;

fn print_args() {
    println!("Arguments:");
    println!("list\t\t\tPrints all color theme names");
    println!("search <query>\t\tSearches for given string");
    println!("<themename>\t\tFetches theme and prints to stdout");
}

// Fn to request themes
fn get_themes_and(url: &str, action: &Fn(&serde_json::Value)) {
    let mut core = Core::new().unwrap();
    let client = Client::configure().connector(HttpsConnector::new(4, &core.handle()).unwrap()).build(&core.handle());

    let uri = url.parse().unwrap();
    let mut req = Request::new(Method::Get, uri);
    req.headers_mut().set_raw("Accept", "application/vnd.github.v3+json".to_owned());
    req.headers_mut().set_raw("User-Agent", "rxres".to_owned());

    let work = client.request(req)
        .and_then(move |res| {
            res.body().concat2().and_then(move |data: Chunk| {
                let v: Value = serde_json::from_slice(&data).unwrap();
                for o in v.as_array().unwrap() {
                    action(o);
                }
                Ok(())
            })
        });

    core.run(work).unwrap();
}

fn download_theme(url: &str) {
    let mut core = Core::new().unwrap();
    let client = Client::configure().connector(HttpsConnector::new(4, &core.handle()).unwrap()).build(&core.handle());

    let uri = url.parse().unwrap();
    let mut req = Request::new(Method::Get, uri);
    req.headers_mut().set_raw("Accept", "text/plain".to_owned());
    req.headers_mut().set_raw("User-Agent", "rxres".to_owned());

    let work = client.request(req)
        .and_then(move |res| {
            res.body().concat2().and_then(move |data: Chunk| {
                println!("{}", str::from_utf8(&data).unwrap());
                Ok(())
            })
        });

    core.run(work).unwrap();
}

// Main
fn main() {

    // Closure operations
    let print_theme = |theme: &serde_json::Value| println!("{}", theme["name"].as_str().unwrap());

    let xcolors_url = "https://api.github.com/repos/tlatsas/xcolors/contents/themes";
    let base64_url = "https://api.github.com/repos/chriskempson/base16-xresources/contents/xresources";

    // Parse args
    let mut it = env::args().into_iter().peekable();
    it.next();
    while let Some(arg) = it.next() {
        if arg == "list" {
            get_themes_and(xcolors_url, &print_theme);
            get_themes_and(base64_url, &print_theme);
            return;
        }

        else if arg == "search" {
            let q = Some(it.next().unwrap().clone()).unwrap();

            get_themes_and(xcolors_url, &|theme: &serde_json::Value| {
                if theme["name"].as_str().unwrap().contains(&q) {
                    print_theme(theme);
                }
            });
            
            get_themes_and(base64_url, &move |theme: &serde_json::Value| {
                if theme["name"].as_str().unwrap().contains(&q) {
                    print_theme(theme);
                }
            });
            return;
        }

        else {
            get_themes_and(xcolors_url, &|theme: &serde_json::Value|{
                if theme["name"].as_str().unwrap().trim() == arg {
                    download_theme(theme["download_url"].as_str().unwrap());
                }
            });
            get_themes_and(base64_url, &|theme: &serde_json::Value|{
                if theme["name"].as_str().unwrap().trim() == arg {
                    download_theme(theme["download_url"].as_str().unwrap());
                }
            });
            return;
        }
    }

    print_args();
}
