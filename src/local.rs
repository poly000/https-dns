use crate::upstream::UpstreamHttpsClient;
use std::{io, net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;
use trust_dns_proto::op::message::Message;

pub struct LocalUdpSocket {
    udp_socket: Arc<UdpSocket>,
    upstream_https_client: UpstreamHttpsClient,
}

impl LocalUdpSocket {
    pub async fn new(host: String, port: u16, upstream_https_client: UpstreamHttpsClient) -> Self {
        let socket_addr = match format!("{}:{}", host, port).parse::<SocketAddr>() {
            Ok(socket_addr) => socket_addr,
            Err(_) => {
                panic!("[local] failed to parse the address {}:{}", host, port,);
            }
        };

        let udp_socket = match UdpSocket::bind(socket_addr).await {
            Ok(udp_socket) => Arc::new(udp_socket),
            Err(err) => {
                if err.kind() == io::ErrorKind::AddrInUse {
                    panic!("[local] the address {}:{} is already in use", host, port,)
                } else {
                    panic!("[local] failed to bind to the address {}:{}", host, port,)
                }
            }
        };

        LocalUdpSocket {
            udp_socket,
            upstream_https_client,
        }
    }

    pub async fn listen(&self) {
        loop {
            let mut buffer = [0; 512];
            let udp_socket = self.udp_socket.clone();
            let upstream_https_client = self.upstream_https_client.clone();
            let (_, addr) = match udp_socket.recv_from(&mut buffer).await {
                Ok(udp_recv_from_result) => udp_recv_from_result,
                Err(_) => {
                    println!("[local] failed to receive the datagram message");
                    continue;
                }
            };

            tokio::spawn(async move {
                LocalUdpSocket::process(upstream_https_client, udp_socket, addr, &buffer).await;
            });
        }
    }

    pub async fn process(
        mut upstream_https_client: UpstreamHttpsClient,
        udp_socket: Arc<UdpSocket>,
        addr: SocketAddr,
        buffer: &[u8; 512],
    ) {
        let request_message = match Message::from_vec(buffer) {
            Ok(request_message) => request_message,
            Err(_) => {
                return;
            }
        };

        let response_message = match upstream_https_client.process(request_message).await {
            Ok(response_message) => response_message,
            Err(error) => {
                println!("{}", error);
                return;
            }
        };

        let raw_response_message = match response_message.to_vec() {
            Ok(raw_response_message) => raw_response_message,
            Err(_) => {
                return;
            }
        };

        match udp_socket.send_to(&raw_response_message, &addr).await {
            Ok(_) => (),
            Err(_) => {
                println!("[local] failed to send the inbound response to the client");
            }
        };
    }
}
