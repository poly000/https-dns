extern crate lru;

use lru::LruCache;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use trust_dns_proto::op::{message::Message, Query};

#[derive(Debug, Hash, PartialEq, Eq)]
struct CacheKey {
    query: Query,
}

#[derive(Debug)]
struct CacheValue {
    message: Message,
    instant: Instant,
    ttl: Duration,
}

#[derive(Clone)]
pub struct Cache {
    lru_cache: Arc<Mutex<LruCache<CacheKey, CacheValue>>>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            lru_cache: Arc::new(Mutex::new(LruCache::new(1024))),
        }
    }

    pub fn put(&mut self, message: Message) {
        let query = match message.queries().iter().next() {
            Some(query) => query.to_owned(),
            None => {
                return;
            }
        };
        let cache_key = CacheKey { query };

        let ttl = match message.answers().iter().next() {
            Some(record) => Duration::from_secs(record.ttl().into()),
            None => {
                return;
            }
        };
        let cache_value = CacheValue {
            ttl,
            instant: Instant::now(),
            message,
        };

        let mut lru_cache = self.lru_cache.lock().unwrap();
        lru_cache.put(cache_key, cache_value);
    }

    pub fn get(&mut self, message: &Message) -> Option<Message> {
        let mut lru_cache = self.lru_cache.lock().unwrap();
        if lru_cache.len() == 0 {
            return None;
        }

        let message_id = message.id();
        let query = match message.queries().iter().next() {
            Some(query) => query.to_owned(),
            None => {
                return None;
            }
        };
        let cache_key = CacheKey { query };

        let cache_value = match lru_cache.get(&cache_key) {
            Some(cache_value) => cache_value,
            None => {
                return None;
            }
        };

        let instant = cache_value.instant;
        let ttl = cache_value.ttl;
        let mut message = cache_value.message.clone();

        if instant.elapsed() < ttl {
            message.set_id(message_id);
            Some(message)
        } else {
            lru_cache.pop(&cache_key);
            None
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Cache;
    use std::net::Ipv4Addr;
    use trust_dns_proto::{
        op::{message::Message, Query},
        rr::{Name, RData, Record, RecordType},
    };

    #[test]
    fn test_cache_hit() {
        let mut cache = Cache::new();
        let mut query = Query::new();
        let name = match "example.com".parse::<Name>() {
            Ok(name) => name,
            Err(_) => panic!("[test] failed to parse example.com"),
        };
        query.set_name(name.clone());

        let mut answer = Record::with(name, RecordType::A, 1000);
        answer.set_data(Some(RData::A(Ipv4Addr::new(1, 1, 1, 1))));

        let mut response_message = Message::new();
        response_message.add_query(query.clone());
        response_message.add_answer(answer.clone());
        cache.put(response_message);

        let mut request_message = Message::new();
        request_message.add_query(query.clone());
        cache.get(&request_message).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_cache_expire() {
        let mut cache = Cache::new();
        let mut query = Query::new();
        let name = match "example.com".parse::<Name>() {
            Ok(name) => name,
            Err(_) => panic!("[test] failed to parse example.com"),
        };
        query.set_name(name.clone());

        let mut answer = Record::with(name, RecordType::A, 0);
        answer.set_data(Some(RData::A(Ipv4Addr::new(1, 1, 1, 1))));

        let mut response_message = Message::new();
        response_message.add_query(query.clone());
        response_message.add_answer(answer.clone());
        cache.put(response_message);

        let mut request_message = Message::new();
        request_message.add_query(query.clone());
        cache.get(&request_message).unwrap();
    }
}
