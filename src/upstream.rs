use crate::error::UpstreamError;
use reqwest::{header, Client};
use trust_dns_client::op::Message;

#[derive(Debug, Clone)]
pub struct UpstreamHttpsClient {
    host: String,
    port: u16,
    https_client: Client,
}

impl UpstreamHttpsClient {
    pub fn new(host: String, port: u16) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/dns-message"),
        );

        let https_client = match Client::builder()
            .default_headers(headers)
            .https_only(true)
            .gzip(true)
            .brotli(true)
            .build()
        {
            Ok(https_client) => https_client,
            Err(_) => {
                panic!("[upstream] failed to build the HTTPS client");
            }
        };

        UpstreamHttpsClient {
            host,
            port,
            https_client,
        }
    }

    pub async fn process(&self, request_message: Message) -> Result<Message, UpstreamError> {
        let raw_request_message = match request_message.to_vec() {
            Ok(raw_request_message) => raw_request_message,
            Err(error) => {
                return Err(error.into());
            }
        };

        let url = format!("https://{}:{}/dns-query", self.host, self.port);
        let request = self.https_client.post(url).body(raw_request_message);
        let response = match request.send().await {
            Ok(response) => response,
            Err(error) => {
                return Err(error.into());
            }
        };

        let raw_response_message = match response.bytes().await {
            Ok(response_bytes) => response_bytes,
            Err(error) => {
                return Err(error.into());
            }
        };

        let message = match Message::from_vec(&raw_response_message) {
            Ok(message) => message,
            Err(error) => {
                return Err(error.into());
            }
        };

        Ok(message)
    }
}
