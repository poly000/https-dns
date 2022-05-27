# https-dns

**https-dns** is a minimal and efficient DNS-over-HTTPS (DoH) client. DNS-over-HTTPS ([RFC 8484](https://datatracker.ietf.org/doc/html/rfc8484)) is a protocol for performing DNS resolution through the HTTPS protocol that prevents manipulation of DNS response. **https-dns** forwards DNS queries from the client to upstream DoH servers, caches the response, and sends the response back to the client.

[![Crates.io](https://img.shields.io/crates/v/https-dns?style=for-the-badge&logo=rust)](https://crates.io/crates/https-dns)
[![Crates.io](https://img.shields.io/crates/d/https-dns?style=for-the-badge&logo=rust)](https://crates.io/crates/https-dns)
[![GitHub Actions](https://img.shields.io/github/workflow/status/xiaoyang-sde/https-dns/check-test-lint?style=for-the-badge&logo=github)](https://github.com/xiaoyang-sde/https-dns/actions)

## Installation

```shell
cargo install https-dns
```

## Usage

```shell
# udp://localhost:53 -> https://1.1.1.1 (default)
sudo https-dns

# udp://localhost:53 -> https://cloudflare-dns.com
sudo https-dns --upstream-address cloudflare-dns.com

# udp://localhost:10053 -> https://dns.google
sudo https-dns --local-port 10053 --upstream-address dns.google
```

### CLI Reference

```shell
$ https-dns --help

https-dns 0.2.0
Minimal and efficient DNS-over-HTTPS (DoH) client

USAGE:
    https-dns [OPTIONS]

OPTIONS:
    -h, --help                                   Print help information
        --local-address <LOCAL_ADDRESS>          [default: 127.0.0.1]
        --local-port <LOCAL_PORT>                [default: 53]
        --upstream-address <UPSTREAM_ADDRESS>    [default: 1.1.1.1]
        --upstream-port <UPSTREAM_PORT>          [default: 443]
    -V, --version                                Print version information
```
