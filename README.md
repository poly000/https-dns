# https-dns

**https-dns** is a minimal and efficient DNS-over-HTTPS (DoH) client. DNS-over-HTTPS ([RFC 8484](https://datatracker.ietf.org/doc/html/rfc8484)) is a protocol for performing DNS resolution through the HTTPS protocol that prevents manipulation of DNS response. **https-dns** forwards DNS queries from the client to upstream DoH servers, caches the response, and sends the response back to the client.

[![Crates.io](https://img.shields.io/crates/v/https-dns?style=for-the-badge&logo=rust)](https://crates.io/crates/https-dns)
[![Crates.io](https://img.shields.io/crates/d/https-dns?style=for-the-badge&logo=rust)](https://crates.io/crates/https-dns)
[![GitHub Actions](https://img.shields.io/github/workflow/status/xiaoyang-sde/https-dns/check-test-lint?style=for-the-badge&logo=github)](https://github.com/xiaoyang-sde/https-dns/actions)

## Installation

```cmd
cargo install https-dns
```

## Usage

```cmd
$ sudo https-dns
[upstream] connected to https://1.1.1.1:443
[local] listening on 127.0.0.1:53
```

```cmd
$ https-dns -h

https-dns 0.1.1
Minimal and efficient DNS-over-HTTPS (DoH) client

USAGE:
    https-dns [OPTIONS]

OPTIONS:
    -h, --help                                   Print help information
    -l, --local-address <LOCAL_ADDRESS>          [default: 127.0.0.1]
    -o, --upstream-port <UPSTREAM_PORT>          [default: 443]
    -p, --local-port <LOCAL_PORT>                [default: 53]
    -u, --upstream-address <UPSTREAM_ADDRESS>    [default: 1.1.1.1]
    -V, --version                                Print version information
```
