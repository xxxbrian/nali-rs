# nali-rs

## About

This project is a Rust-based rewrite of the original [Nali project](https://github.com/zu1k/nali), which was originally developed in Golang.

## Installation

```
cargo install nali
```

## Usage

### Query a simple IP address with `pipe`

```
$ echo IP 6.6.6.6 | nali
IP 6.6.6.6 [United States]
```

### Interactive query

use `Ctrl + C` to exit

```
$ nali
123.23.23.23
123.23.23.23 [Vietnam Vĩnh Long Province]
1.0.0.1
1.0.0.1 [Australia]
8.8.8.8
8.8.8.8 [United States]
^C
```

### Use with `dog`

```
$ dog crates.io @quic://doh.nszero.net | nali
crates.io.      A       IN      1s      13.33.88.13 [United States]     dns.nszero.net:853
crates.io.      A       IN      1s      13.33.88.97 [United States]     dns.nszero.net:853
crates.io.      A       IN      1s      13.33.88.22 [United States]     dns.nszero.net:853
crates.io.      A       IN      1s      13.33.88.79 [United States]     dns.nszero.net:853
```

### Use with `nslookup`

```
$ nslookup crates.io 8.8.8.8 | nali
Server:         8.8.8.8 [United States]
Address:        8.8.8.8 [United States]
#53

Non-authoritative answer:
Name:   crates.io
Address: 13.226.61.78 [United States]
Name:   crates.io
Address: 13.226.61.46 [United States]
Name:   crates.io
Address: 13.226.61.96 [United States]
Name:   crates.io
Address: 13.226.61.110 [United States]
```

### Use with any other program

Because nali can read the contents of the `stdin` pipeline, it can be used with any program.

```
bash abc.sh | nali
```

Nali will insert IP information after IP address.

### IPv6 support

Use like IPv4

```
$ dog crates.io @quic://doh.nszero.net | nali
NAME            TYPE    CLASS   TTL     ADDRESS         NAMESERVER
crates.io.      A       IN      5s      13.33.88.13 [United States]     doh.nszero.net:853
crates.io.      A       IN      5s      13.33.88.97 [United States]     doh.nszero.net:853
crates.io.      A       IN      5s      13.33.88.22 [United States]     doh.nszero.net:853
crates.io.      A       IN      5s      13.33.88.79 [United States]     doh.nszero.net:853
```
