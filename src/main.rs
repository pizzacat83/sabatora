#![no_std]
#![cfg_attr(not(target_ps = "linux"), no_main)]

extern crate alloc;
use alloc::string::ToString;
use net_wasabi::http::HttpClient;
use noli::prelude::*;

fn main() -> u64 {
    let client = HttpClient::new();
    match client.get("host.test".to_string(), 8000, "/simple.html".to_string()) {
        Ok(res) => {
            print!("response:\n{:#?}", res);
        }
        Err(e) => {
            print!("error:\n{:#?}", e);
        }
    }
    0
}

entry_point!(main);
