use crate::bootstrap::BootstrapClient;
use crate::error::UpstreamError::{self, Build, Resolve};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client,
};
use std::{net::IpAddr, time::Duration};
use tracing::info;
use trust_dns_proto::op::message::Message;

#[derive(Clone, Debug)]
pub struct HttpsClient {
    host: String,
    port: u16,
    https_client: Client,
}

impl HttpsClient {
    pub async fn new(host: String, port: u16) -> Result<Self, UpstreamError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/dns-message").unwrap(),
        );

        let mut client_builder = Client::builder()
            .default_headers(headers)
            .https_only(true)
            .gzip(true)
            .brotli(true)
            .timeout(Duration::from_secs(10));

        if host.parse::<IpAddr>().is_err() {
            let bootstrap_client = match BootstrapClient::new() {
                Ok(bootstrap_client) => bootstrap_client,
                Err(error) => return Err(error),
            };
            let ip_addr = match bootstrap_client.bootstrap(&host).await {
                Ok(ip_addr) => ip_addr,
                Err(error) => return Err(error),
            };
            client_builder = client_builder.resolve(&host, ip_addr);
        }

        let https_client = match client_builder.build() {
            Ok(https_client) => https_client,
            Err(_) => return Err(Build),
        };
        info!("connected to https://{}:{}", host, port);

        Ok(HttpsClient {
            host,
            port,
            https_client,
        })
    }

    pub async fn process(&mut self, request_message: Message) -> Result<Message, UpstreamError> {

        let raw_request_message = match request_message.to_vec() {
            Ok(raw_request_message) => raw_request_message,
            Err(_) => return Err(Resolve),
        };

        let url = format!("https://{}:{}/dns-query", self.host, self.port);
        let request = self.https_client.post(url).body(raw_request_message);
        let response = match request.send().await {
            Ok(response) => response,
            Err(_) => return Err(Resolve),
        };

        let raw_response_message = match response.bytes().await {
            Ok(response_bytes) => response_bytes,
            Err(_) => return Err(Resolve),
        };

        let message = match Message::from_vec(&raw_response_message) {
            Ok(message) => message,
            Err(_) => return Err(Resolve),
        };

        Ok(message)
    }
}
