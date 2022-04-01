use crate::bootstrap::BootstrapHttpsClient;
use crate::cache::Cache;
use crate::error::UpstreamError;
use reqwest::{header, Client};
use std::{net::IpAddr, time::Duration};
use trust_dns_proto::op::message::Message;

#[derive(Clone)]
pub struct UpstreamHttpsClient {
    host: String,
    port: u16,
    https_client: Client,
    cache: Cache,
}

impl UpstreamHttpsClient {
    pub async fn new(host: String, port: u16) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/dns-message"),
        );

        let mut client_builder = Client::builder()
            .default_headers(headers)
            .https_only(true)
            .gzip(true)
            .brotli(true)
            .timeout(Duration::from_secs(10));

        if host.as_str().parse::<IpAddr>().is_err() {
            let bootstrap_https_client = BootstrapHttpsClient::new();
            let ip_addr = match bootstrap_https_client.bootstrap(host.clone()).await {
                Ok(ip_addr) => ip_addr,
                Err(_) => panic!("[upstream] failed to bootstrap the DNS-over-HTTPS client"),
            };
            client_builder = client_builder.resolve(host.as_str(), ip_addr);
        }

        let https_client = match client_builder.build() {
            Ok(https_client) => {
                println!("[upstream] connected to https://{}:{}", host, port);
                https_client
            }
            Err(_) => panic!("[upstream] failed to build the HTTPS client"),
        };

        UpstreamHttpsClient {
            host,
            port,
            https_client,
            cache: Cache::new(),
        }
    }

    pub async fn process(&mut self, request_message: Message) -> Result<Message, UpstreamError> {
        if let Some(response_message) = self.cache.get(&request_message) {
            return Ok(response_message);
        }

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

        self.cache.put(message.clone());
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::BootstrapHttpsClient;
    use std::net::{Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn test_bootstrap() {
        let bootstrap_https_client = BootstrapHttpsClient::new();
        let host = String::from("dns.google");
        let ip_addr = match bootstrap_https_client.bootstrap(host).await {
            Ok(ip_addr) => ip_addr,
            Err(_) => panic!("[test] failed to bootstrap the DNS-over-HTTPS service"),
        };
        let expected_ip_addr = [
            SocketAddr::new(Ipv4Addr::new(8, 8, 8, 8).into(), 0),
            SocketAddr::new(Ipv4Addr::new(8, 8, 4, 4).into(), 0),
        ];
        assert!(expected_ip_addr.contains(&ip_addr));
    }
}
