extern crate alloc;
use core::f32::consts::LN_10;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use noli::net::{lookup_host, SocketAddr, TcpStream};
use saba_core::error::Error;
use saba_core::http::HttpResponse;

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, host: String, port: u16, path: String) -> Result<HttpResponse, Error> {
        let ips = lookup_host(&host)
            .map_err(|err| Error::Network(format!("Failed to find IP addresses: {:#?}", err)))?;
        if ips.is_empty() {
            return Err(Error::Network("Failed to find IP addresses".to_string()));
        }

        let socket_addr: SocketAddr = (ips[0], port).into();

        let mut stream = TcpStream::connect(socket_addr)
            .map_err(|err| Error::Network("Failed to connect to TCP stream".to_string()))?;

         
        let mut request = String::from("GET /");
        request.push_str(&path);
        request.push_str(" HTTP/1.1\n");

        // add headers
        request.push_str("Host: ");
        request.push_str(&host);
        request.push_str("\n");
        request.push_str("Accept: text/html\n");
        request.push_str("Connection: close\n");
        request.push_str("\n");

        let _bytes_written = stream
            .write(request.as_bytes())
            .map_err(|err| Error::Network("Failed to send a reqyestr to TCP stream".to_string()))?;

        let mut received = Vec::new();
        loop {
            let mut buf = [0u8; 4096];
            let bytes_read = stream.read(&mut buf).map_err(|err| {
                Error::Network("Failed to receive a request from TCP stream".to_string())
            })?;
            if bytes_read == 0 {
                break;
            }
            received.extend_from_slice(&buf[..bytes_read]);
        }
        let response = core::str::from_utf8(&received)
            .map_err(|err| Error::Network(format!("Invalid received response: {}", err)))?;

        HttpResponse::new(response.to_string())
    }
}
