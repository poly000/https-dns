use crate::error::LocalError::{self, InvalidAddress, PermissionDenied, Unknown};
use crate::upstream::HttpsClient;
use std::{io, net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;
use tracing::{info, instrument, warn};
use trust_dns_proto::op::message::Message;

#[derive(Debug)]
pub struct UdpListener {
    udp_socket: Arc<UdpSocket>,
    https_client: HttpsClient,
}

impl UdpListener {
    #[instrument(skip(https_client))]
    pub async fn new(
        host: String,
        port: u16,
        https_client: HttpsClient,
    ) -> Result<Self, LocalError> {
        let socket_addr: SocketAddr = match format!("{}:{}", host, port).parse() {
            Ok(socket_addr) => socket_addr,
            Err(_) => return Err(InvalidAddress(host, port)),
        };

        let udp_socket = match UdpSocket::bind(socket_addr).await {
            Ok(udp_socket) => Arc::new(udp_socket),
            Err(error) => match error.kind() {
                io::ErrorKind::PermissionDenied => return Err(PermissionDenied(host, port)),
                _ => return Err(Unknown(host, port)),
            },
        };
        info!("listened on {}:{}", host, port);

        Ok(UdpListener {
            udp_socket,
            https_client,
        })
    }

    #[instrument(skip(self))]
    pub async fn listen(&self) {
        loop {
            let mut buffer = [0; 512];
            let mut https_client = self.https_client.clone();
            let udp_socket = self.udp_socket.clone();

            let (_, addr) = match udp_socket.recv_from(&mut buffer).await {
                Ok(udp_recv_from_result) => udp_recv_from_result,
                Err(_) => {
                    warn!("failed to receive the datagram message");
                    continue;
                }
            };

            info!("received DNS request from {}", addr);

            tokio::spawn(async move {
                let request_message = match Message::from_vec(&buffer) {
                    Ok(request_message) => request_message,
                    Err(_) => {
                        warn!("failed to parse the DNS request");
                        return;
                    }
                };

                let response_message = match https_client.process(request_message).await {
                    Ok(response_message) => response_message,
                    Err(error) => {
                        warn!("{}", error);
                        return;
                    }
                };

                let raw_response_message = match response_message.to_vec() {
                    Ok(raw_response_message) => raw_response_message,
                    Err(_) => {
                        warn!("failed to parse the DNS response");
                        return;
                    }
                };

                match udp_socket.send_to(&raw_response_message, &addr).await {
                    Ok(_) => info!("sent DNS response to {}", addr),
                    Err(_) => warn!("failed to send the inbound response to the client"),
                };
            });
        }
    }
}
