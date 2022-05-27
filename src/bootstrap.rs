use crate::error::UpstreamError::{self, Bootstrap, Build};
use crate::utils::build_request_message;
use http::header::{ACCEPT, CONTENT_TYPE};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use std::{net::SocketAddr, time::Duration};
use trust_dns_proto::{
    op::message::Message,
    rr::{Name, RData, RecordType},
};

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
        headers.insert(
            ACCEPT,
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
        let request_name = match host.parse::<Name>() {
            Ok(request_name) => request_name,
            Err(error) => return Err(Bootstrap(host.to_string(), error.to_string())),
        };
        let request_message = build_request_message(request_name, RecordType::A);

        let raw_request_message = match request_message.to_vec() {
            Ok(raw_request_message) => raw_request_message,
            Err(error) => return Err(Bootstrap(host.to_string(), error.to_string())),
        };

        let url = "https://1.1.1.1/dns-query";
        let request = self.https_client.post(url).body(raw_request_message);
        let response = match request.send().await {
            Ok(response) => response,
            Err(error) => return Err(Bootstrap(host.to_string(), error.to_string())),
        };

        let raw_response_message = match response.bytes().await {
            Ok(response_bytes) => response_bytes,
            Err(error) => return Err(Bootstrap(host.to_string(), error.to_string())),
        };

        let response_message = match Message::from_vec(&raw_response_message) {
            Ok(response_message) => response_message,
            Err(error) => return Err(Bootstrap(host.to_string(), error.to_string())),
        };

        if response_message.answers().is_empty() {
            return Err(Bootstrap(
                host.to_string(),
                String::from("the response doesn't contain the answer"),
            ));
        }
        let record = &response_message.answers()[0];
        let record_data = match record.data() {
            Some(record_data) => record_data,
            None => {
                return Err(Bootstrap(
                    host.to_string(),
                    String::from("the response doesn't contain the answer"),
                ))
            }
        };

        match record_data {
            RData::A(ipv4_address) => Ok(SocketAddr::new((*ipv4_address).into(), 0)),
            RData::AAAA(ipv6_address) => Ok(SocketAddr::new((*ipv6_address).into(), 0)),
            _ => Err(Bootstrap(
                host.to_string(),
                String::from("unknown record type"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BootstrapClient;
    use std::{
        collections::HashMap,
        net::{Ipv4Addr, SocketAddr},
    };

    #[tokio::test]
    async fn test_bootstrap() {
        let bootstrap_client = BootstrapClient::new().unwrap();
        let bootstrap_result_map = HashMap::from([
            (
                "dns.google",
                vec![
                    SocketAddr::new(Ipv4Addr::new(8, 8, 8, 8).into(), 0),
                    SocketAddr::new(Ipv4Addr::new(8, 8, 4, 4).into(), 0),
                ],
            ),
            (
                "one.one.one.one",
                vec![
                    SocketAddr::new(Ipv4Addr::new(1, 1, 1, 1).into(), 0),
                    SocketAddr::new(Ipv4Addr::new(1, 0, 0, 1).into(), 0),
                ],
            ),
            (
                "dns.quad9.net",
                vec![
                    SocketAddr::new(Ipv4Addr::new(9, 9, 9, 9).into(), 0),
                    SocketAddr::new(Ipv4Addr::new(149, 112, 112, 112).into(), 0),
                ],
            ),
            (
                "dns.adguard.com",
                vec![
                    SocketAddr::new(Ipv4Addr::new(94, 140, 14, 14).into(), 0),
                    SocketAddr::new(Ipv4Addr::new(94, 140, 15, 15).into(), 0),
                ],
            ),
        ]);

        for (host, socket_addr_list) in bootstrap_result_map {
            let result = bootstrap_client.bootstrap(host).await.unwrap();
            assert!(socket_addr_list.contains(&result));
        }
    }
}
