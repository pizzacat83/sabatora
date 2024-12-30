use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    version: String,
    status_code: u32,
    reason: String,
    headers: Vec<Header>,
    body: String,
}

#[derive(Debug, Clone)]
pub struct Header {
    name: String,
    value: String,
}

impl Header {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

impl HttpResponse {
    pub fn new(raw_response: String) -> Result<Self, Error> {
        let preprocessed_response = raw_response.trim_start().replace("\r\n", "\n");

        let (status_line, remaining) =
            preprocessed_response
                .split_once('\n')
                .ok_or(Error::Network(format!(
                    "invalid http response: {}",
                    preprocessed_response
                )))?;

        let (headers, body) = match remaining.split_once("\n\n") {
            Some((h, b)) => {
                let mut headers = Vec::new();
                for header in h.split('\n') {
                    let (name, value) = header
                        .split_once(':')
                        .ok_or(Error::Network(format!("invalid header: {}", header)))?;
                    headers.push(Header::new(
                        name.trim().to_string(),
                        value.trim().to_string(),
                    ));
                }
                (headers, b)
            }
            None => (Vec::new(), remaining),
        };

        let statuses: Vec<&str> = status_line.splitn(3, ' ').collect();
        Ok(Self {
            version: statuses[0].to_string(),
            status_code: statuses[1]
                .parse()
                .map_err(|_| Error::Network(format!("invalid status code: {}", statuses[1])))?,
            reason: statuses[2].to_string(),
            headers,
            body: body.to_string(),
        })
    }

    pub fn version(&self) -> String {
        self.version.clone()
    }

    pub fn status_code(&self) -> u32 {
        self.status_code
    }

    pub fn reason(&self) -> String {
        self.reason.clone()
    }

    pub fn body(&self) -> String {
        self.body.clone()
    }

    pub fn headers(&self) -> Vec<Header> {
        self.headers.clone()
    }

    pub fn header_value(&self, name: &str) -> Result<String, String> {
        self.headers
            .iter()
            .find(|h| h.name == name)
            .map(|h| h.value.clone())
            .ok_or(format!("failed to find {} in headers", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_line_only() {
        let raw = "HTTP/1.1 200 OK\n\n".to_string();

        let res = HttpResponse::new(raw).unwrap();
        assert_eq!(res.version(), "HTTP/1.1");
        assert_eq!(res.status_code(), 200);
        assert_eq!(res.reason(), "OK");
    }

    #[test]
    fn test_one_header() {
        let raw = "HTTP/1.1 200 OK\nDate: xx xx xx\n\n".to_string();
        let res = HttpResponse::new(raw).unwrap();
        assert_eq!(res.version(), "HTTP/1.1");
        assert_eq!(res.status_code(), 200);
        assert_eq!(res.reason(), "OK");

        assert_eq!(res.header_value("Date").unwrap(), "xx xx xx");
    }

    #[test]
    fn test_two_headers_with_whitespace() {
        let raw = "HTTP/1.1 200 OK\nDate: xx xx xx\nContent-Length: 42\n\n".to_string();
        let res = HttpResponse::new(raw).unwrap();
        assert_eq!(res.version(), "HTTP/1.1");
        assert_eq!(res.status_code(), 200);
        assert_eq!(res.reason(), "OK");

        assert_eq!(res.header_value("Date").unwrap(), "xx xx xx");
        assert_eq!(res.header_value("Content-Length").unwrap(), "42");
    }

    #[test]
    fn test_body() {
        let raw = "HTTP/1.1 200 OK\nDate: xx xx xx\n\nbody message".to_string();
        let res = HttpResponse::new(raw).unwrap();
        assert_eq!(res.version(), "HTTP/1.1");
        assert_eq!(res.status_code(), 200);
        assert_eq!(res.reason(), "OK");

        assert_eq!(res.header_value("Date").unwrap(), "xx xx xx");

        assert_eq!(res.body(), "body message");
    }

    #[test]
    fn test_invalid() {
        let raw = "HTTP/1.1 200 OK".to_string();
        assert!(HttpResponse::new(raw).is_err());
    }
}
