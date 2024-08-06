# Bitcoin Mock Remote Procedure Call

This library is a mock of [bitcoincore-rpc](https://github.com/rust-bitcoin/rust-bitcoincore-rpc)
library, which is a wrapper of Bitcoin RPC for Rust. This library aims to mock
`RpcApi` trait interface of bitcoincore-rpc and provide a separate mock
blockchain for every unit and integration test.

## Differences Between Real Bitcoin RPC

Some of the RPC functions behave similarly with real RPC while some of them are
not. To check if an RPC function behaves different than the real one, please
check function comments in [`src/client/rpc_api.rs`](src/client/rpc_api.rs).

## Usage

This mock won't provide a CLI tool. Instead, you should use this in your Rust
code. You can use generics as your RPC struct and use this mock in your tests
and real RPC interface in your application code.

Example:

```rust
struct MyStruct<R: RpcApiWrapper> {
    data: u32,
    rpc: R,
}

fn my_func() {
    let strct = MyStruct {
        data: 0x45,
        // This will connect Bitcoin RPC.
        rpc: bitcoincore_rpc::Client::new(/** parameters here **/),
    };

    // Do stuff...
}

#[test]
fn test() {
    let strct = MyStruct {
        data: 0x1F,
        // This will create a mock blockchain, on memory.
        rpc: bitcoin_mock_rpc::Client::new(/** parameters here **/),
    };

    // Do stuff...
}
```

This library is aimed to help development of [clementine](https://github.com/chainwayxyz/clementine).
Therefore, it's usage of this library can be taken as a reference.

## Testing

Standard Rust tools are sufficient for testing:

```bash
cargo test
```

## Documentation

No external documentation is provided. Please read comments in code for
documentation.

## License

This project is licensed with [MIT license](LICENSE).
