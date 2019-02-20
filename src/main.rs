#![allow(dead_code)]
#[macro_use] extern crate serde_derive;
extern crate getopts;
extern crate serde_json;
extern crate url;
extern crate websocket;
extern crate http;
extern crate base64;
extern crate uuid;
extern crate reqwest;
extern crate libflate;

mod commands;
mod vmservice;
mod devfs;
mod compile;

use commands::*;

fn main() {
    run().unwrap();
}
