use crate::local::UdpListener;
use crate::upstream::HttpsClient;
use rand::{thread_rng, Rng};
use trust_dns_proto::{
    op::{Message, MessageType, Query},
    rr::{Name, RecordType},
};

pub fn build_request_message(name: Name, record_type: RecordType) -> Message {
    let mut request_message = Message::new();

    let mut rng = thread_rng();
    let message_id = rng.gen_range(0..=65535);
    request_message.set_id(message_id);
    request_message.set_message_type(MessageType::Query);
    request_message.set_recursion_desired(true);

    let query = Query::query(name, record_type);
    request_message.add_query(query);

    request_message
}

#[allow(dead_code)]
pub async fn build_test_listener() -> UdpListener {
    let upstream_address = String::from("cloudflare-dns.com");
    let upstream_port = 443;
    let local_address = String::from("127.0.0.1");
    let local_port = 10053;

    let https_client = HttpsClient::new(upstream_address, upstream_port)
        .await
        .unwrap();
    UdpListener::new(local_address, local_port, https_client)
        .await
        .unwrap()
}
