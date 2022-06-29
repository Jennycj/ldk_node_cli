# ldk_node_cli

This is a node based on LDK's sample node implementation. The CLI is intended to be used with the node and function just as `lncli` works for `LND`.

## Installation

``` bash
git clone git@github.com:Jennycj/ldk_node_cli.git
```

## Usage

### Start up the LDK node

``` bash
cd ldk_node_cli
cargo run --bin <bitcoind-rpc-username>:<bitcoind-rpc-password>@<bitcoind-rpc-host>:<bitcoind-rpc-port> <ldk_storage_directory_path> [<ldk-peer-listening-port>] [bitcoin-network] [announced-listen-addr announced-node-name]
```

`bitcoind`'s RPC username and password likely can be found through `cat ~/.bitcoin/.cookie`.

`bitcoin-network`: defaults to `testnet`. Options: `testnet`, `regtest`, and `signet`.

`ldk-peer-listening-port`: defaults to `9735`.

`announced-listen-addr` and `announced-node-name`: default to nothing, disabling any public announcements of this node. `announced-listen-addr` can be set to an `IPv4` or `IPv6` address to announce that as a publicly-connectable address for this node. `announced-node-name` can be any string up to `32 bytes` in length, representing this node's `alias`.

### Run the CLI

``` bash
cargo run --bin cli
```

## License

Licensed under either:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
