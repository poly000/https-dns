use criterion::{black_box, criterion_group, criterion_main, Criterion};
use https_dns::cache::Cache;
use std::net::Ipv4Addr;
use tokio::runtime::Builder;
use trust_dns_proto::{
    op::{Message, Query},
    rr::{Name, RData, Record, RecordType},
};

async fn cache() {
    let cache = &Cache::new();

    let mut handle_list = Vec::new();
    for i in 0..10000 {
        let mut cache = cache.clone();
        let handle = tokio::spawn(async move {
            let mut query = Query::new();
            let name: Name = format!("{i}.example.com").parse().unwrap();
            query.set_name(name.clone());

            let mut answer = Record::with(name, RecordType::A, 1440);
            answer.set_data(Some(RData::A(Ipv4Addr::new(1, 1, 1, 1))));

            let mut response_message = Message::new();
            response_message.add_query(query.clone());
            response_message.add_answer(answer.clone());
            cache.put(black_box(response_message));

            let mut request_message = Message::new();
            request_message.add_query(query.clone());
            cache.get(black_box(&request_message));
        });
        handle_list.push(handle);
    }

    for handle in handle_list {
        handle.await.unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let runtime = Builder::new_multi_thread().enable_all().build().unwrap();
    c.bench_function("cache", |b| b.to_async(&runtime).iter(cache));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
