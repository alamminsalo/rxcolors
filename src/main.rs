extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate serde_json;

use futures::{Future, Stream};
use hyper::{Method, Request, Client, Chunk};
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use serde_json::{Value};
use std::env;

fn print_args() {
  println!("Arguments:");
}

// Fn to request themes
fn get_themes_and(action: &Fn(&serde_json::Value)) {
  let mut core = Core::new().unwrap();
  let client = Client::configure().connector(HttpsConnector::new(4, &core.handle()).unwrap()).build(&core.handle());

  let uri = "https://api.github.com/repos/tlatsas/xcolors/contents/themes".parse().unwrap();
  let mut req = Request::new(Method::Get, uri);
  req.headers_mut().set_raw("Accept", "application/vnd.github.v3+json".to_owned());
  req.headers_mut().set_raw("User-Agent", "rxres".to_owned());

  let work = client.request(req)
    .and_then(move |res| {
	println!("Response: {}", res.status());
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

// Main
fn main() {

  // Closure operations
  let print_theme = |theme: &serde_json::Value| println!("{}", theme["name"].as_str().unwrap());
  let download_theme = |theme: &serde_json::Value| println!("{}", theme["download_url"].as_str().unwrap());

  // Parse args
  let mut it = env::args().into_iter().peekable();
  while let Some(arg) = it.next() {
      if arg == "list" {
	get_themes_and(&print_theme);
	break;
      }

      else if arg == "search" {
	let q = Some(it.next().unwrap().clone()).unwrap();
	get_themes_and(&move |theme: &serde_json::Value| {
	    if theme["name"].as_str().unwrap().contains(&q) {
	      print_theme(theme);
	      }
	    });
	break;
      }

      else if it.peek() == None {
	  print_args();
      }
  }
}
