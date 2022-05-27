use crate::error::UpstreamError::{self, Bootstrap, Build};
use http::header::CONTENT_TYPE;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use std::{net::SocketAddr, time::Duration};

use trust_dns_proto::{
    op::{message::Message, Query},
    rr::{Name, RData},
};

#[derive(Clone, Debug)]
pub struct BootstrapClient {
    https_client: Client,
}

impl BootstrapClient {
    pub fn new() -> Result<Self, UpstreamError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/dns-message").unwrap(),
        );

        let client_builder = Client::builder()
            .default_headers(headers)
            .https_only(true)
            .gzip(true)
            .brotli(true)
            .timeout(Duration::from_secs(10));

        let https_client = match client_builder.build() {
            Ok(https_client) => https_client,
            Err(_) => return Err(Build),
        };

        Ok(BootstrapClient { https_client })
    }

    pub async fn bootstrap(&self, host: &str) -> Result<SocketAddr, UpstreamError> {
        let mut query = Query::new();
        let query_name = match host.parse::<Name>() {
            Ok(query_name) => query_name,
            Err(_) => panic!("[bootstrap] failed to parse the host {}", host),
        };
        query.set_name(query_name);

        let mut request_message = Message::new();
        request_message.add_query(query);
        let raw_request_message = match request_message.to_vec() {
            Ok(raw_request_message) => raw_request_message,
            Err(_) => return Err(Bootstrap(host.to_string())),
        };

        let url = "https://1.1.1.1/dns-query";
        let request = self.https_client.post(url).body(raw_request_message);
        let response = match request.send().await {
            Ok(response) => response,
            Err(_) => return Err(Bootstrap(host.to_string())),
        };

        let raw_response_message = match response.bytes().await {
            Ok(response_bytes) => response_bytes,
            Err(_) => return Err(Bootstrap(host.to_string())),
        };

        let message = match Message::from_vec(&raw_response_message) {
            Ok(message) => message,
            Err(_) => return Err(Bootstrap(host.to_string())),
        };

        if message.answer_count() == 0 {
            return Err(Bootstrap(host.to_string()));
        }

        let record = &message.answers()[0];
        let record_data = match record.data() {
            Some(record_data) => record_data,
            None => return Err(Bootstrap(host.to_string())),
        };

        match record_data {
            RData::A(ipv4_address) => Ok(SocketAddr::new((*ipv4_address).into(), 0)),
            RData::AAAA(ipv6_address) => Ok(SocketAddr::new((*ipv6_address).into(), 0)),
            _ => Err(Bootstrap(host.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BootstrapClient;
    use std::net::{Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn test_bootstrap() {
        let bootstrap_https_client = BootstrapClient::new().unwrap();
        let host = "dns.google";
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
