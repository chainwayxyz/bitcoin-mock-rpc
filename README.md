# Bitcoin Mock Remote Procedure Call

`bitcoin-mock-rpc` is a mock Bitcoin ledger without a wallet support built on
`RpcApi` trait in
[bitcoincore-rpc](https://github.com/rust-bitcoin/rust-bitcoincore-rpc) library.
Meaning there are only checks for consensus details of an operation. This
library can be used to test Bitcoin applications, without needing to set up
Bitcoin and with a **sandboxed environment**, for each test.

This library is aimed to help the development of
[Clementine](https://github.com/chainwayxyz/clementine). Therefore, it's usage
of this library can be taken as a reference.

**Disclaimer:** This library **must not** be used for consensus purposes. And
not gives any guarantee to act as the Bitcoin itself at any scale. Use it at
your own risk.

## Differences Between Real Bitcoin RPC and Feature Set

This library is currently **under heavy development**. And it is not expected to
provide a full Bitcoin experience. Code needs to be checked for what really is
available as futures. Also, [changelog](CHANGELOG.md) is a great summary for
what's available.

Some of the RPC functions behave similarly with real RPC while some of them are
not (mostly wallet operations). To check if an RPC function behaves different
than the real one, please check function comments in
[`src/client/rpc_api.rs`](src/client/rpc_api.rs).

## Usage

`RpcApiWrapper` trait can be used to select between real and mock RPC:

```rust
struct MyStruct<R: RpcApiWrapper> {
    data: u32,
    rpc: R,
}

fn my_func() {
    let strct = MyStruct {
        data: 0x45,
        // This will connect to Bitcoin RPC.
        rpc: bitcoincore_rpc::Client::new(/** parameters here **/),
    };

    // Do stuff...
}

#[test]
fn test() {
    let strct = MyStruct {
        data: 0x1F,
        // This will connect to mock RPC.
        rpc: bitcoin_mock_rpc::Client::new(/** parameters here **/),
    };

    // Do stuff...
}
```

## Testing

Standard Rust tools are sufficient for testing:

```bash
cargo test
```

Additionally, logging level can be set to view crucial information. There are
multiple levels of logs:

```bash
RUST_LOG=trace cargo test # Prints every detail
RUST_LOG=debug cargo test
RUST_LOG=info cargo test
RUST_LOG=warn cargo test
RUST_LOG=error cargo test # Prints only errors
```

Please check
[log library's documentation](https://docs.rs/log/latest/log/enum.Level.html)
for more detail.

## Documentation

No external documentation is provided. Please read code comments or run:

```bash
cargo doc
```

## License

This project is licensed with [MIT license](LICENSE).
