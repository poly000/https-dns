use https_dns::utils::{build_request_message, build_test_listener};
use tokio::{net::UdpSocket, test};
use trust_dns_proto::{
    op::Message,
    rr::{Name, RData, RecordType},
};

#[test]
async fn txt_record() {
    let udp_listener = build_test_listener().await;
    tokio::spawn(async move {
        udp_listener.listen().await;
    });

    for host in ["facebook.com", "cloudflare.com", "stripe.com"] {
        let request_name: Name = host.parse().unwrap();
        let request_message = build_request_message(request_name, RecordType::TXT);
        let raw_request_message = request_message.to_vec().unwrap();

        let udp_socket = UdpSocket::bind("127.0.0.1:10054").await.unwrap();
        udp_socket.connect("127.0.0.1:10053").await.unwrap();

        udp_socket.send(&raw_request_message).await.unwrap();
        let mut buffer = [0; 4096];
        udp_socket.recv(&mut buffer).await.unwrap();

        let response_message = Message::from_vec(&buffer).unwrap();
        let record_data = &response_message.answers()[0].data().unwrap();
        if let RData::TXT(txt) = record_data {
            assert!(!txt.txt_data().is_empty());
        } else {
            panic!("the record type is not TXT")
        }
    }
}
