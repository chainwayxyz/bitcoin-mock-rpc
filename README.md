# Bitcoin Mock Remote Procedure Call

This library mocks [bitcoincore-rpc](https://github.com/rust-bitcoin/rust-bitcoincore-rpc)
library. This mock takes the advantage of `bitcoincore-rpc` trait interface,
called `RpcApi`. With this mock, every test can have an isolated Bitcoin
environment without changing your existing code too much.

[bitcoin-simulator](https://github.com/Bitcoin-Wildlife-Sanctuary/bitcoin-simulator)
is used for creating an isolated Bitcoin environment. Which can be also called a
mock.

These 2 means one don't need any external connection/binary to test code that
uses Bitcoin network.

## License

This project is licensed with [MIT license](LICENSE).
