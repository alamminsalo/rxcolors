extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate serde_json;

use futures::{Future, Stream};
use hyper::{Method, Request, Client, Error};
use hyper_tls::{HttpsConnector};
use tokio_core::reactor::Core;
use serde_json::{Value};
use std::env;
use std::str;

// Struct for JSON calls
struct GithubResource {
    client: Client<HttpsConnector<hyper::client::HttpConnector>>
}

impl GithubResource {

    fn get_raw(&self, url: &str) -> Box<Future<Item=String, Error=Error>>
    {
        let mut req = Request::new(Method::Get, url.parse().unwrap());
        req.headers_mut().set_raw("Accept", "text/plain".to_owned());
        req.headers_mut().set_raw("User-Agent", "rxres".to_owned());

        Box::new(self.client.request(req)
                 .and_then(|res| {
                     res.body().concat2().and_then(|data| {
                         // Here final result as json value
                         Ok(String::new(data))
                     })
                 })) 
    }


    fn get(&self, url: &str) -> Box<Future<Item=Value, Error=Error>>
    {
        let mut req = Request::new(Method::Get, url.parse().unwrap());
        req.headers_mut().set_raw("Accept", "application/vnd.github.v3+json".to_owned());
        req.headers_mut().set_raw("User-Agent", "rxres".to_owned());

        Box::new(self.client.request(req)
                 .and_then(|res| {
                     res.body().concat2().and_then(|data| {
                         // Here final result as json value
                         Ok(serde_json::from_slice(&data).unwrap())
                     })
                 }))
    }
}

fn print_args() {
    println!("Arguments:");
    println!("list\t\t\tPrints all color theme names");
    println!("<themename>\t\tFetches theme and prints to stdout");
}

// Main
fn main() {

    let mut core = Core::new().unwrap();
    let api = GithubResource{client: Client::configure().connector(HttpsConnector::new(4, &core.handle()).unwrap()).build(&core.handle())};

    // Closure operations
    let print_theme = |theme: &Value| println!("{}", theme["name"].as_str().unwrap());

    let xcolors_url = "https://api.github.com/repos/tlatsas/xcolors/contents/themes";
    let base64_url = "https://api.github.com/repos/chriskempson/base16-xresources/contents/xresources";

    // Parse args
    let mut it = env::args().into_iter().peekable();
    it.next();
    while let Some(arg) = it.next() {
        if arg == "list" {
            let work = api.get(xcolors_url)
                .join(api.get(base64_url))
                .and_then(|(a, b)| {
                    //Chain iterators and print
                    for value in a.as_array().unwrap().iter()
                        .chain(b.as_array().unwrap().iter()) {
                            print_theme(value);
                        }
                    Ok(())
                });
            core.run(work).unwrap();
            return;
        }

        else {
            let work = api.get(xcolors_url)
                .join(api.get(base64_url))
                .and_then(|(a,b)| {
                    Ok(a.as_array().unwrap().iter()
                       .chain(b.as_array().unwrap().iter()).collect::<Vec<Value>>())
                })
            .and_then(|values| {
                for value in values {
                    if value["name"].as_str().unwrap().trim() == arg {
                        api.get_raw(value["download_url"].as_str().unwrap())
                            .and_then(|data| {
                                println!("{:?}", data);
                                Ok(data)
                            })
                    }
                }
                Err(())
            });
            core.run(work).unwrap();
            return;
        }
    }

    print_args();
}


