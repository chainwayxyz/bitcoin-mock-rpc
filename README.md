# Bitcoin Mock Remote Procedure Call

This library mocks [bitcoincore-rpc](https://github.com/rust-bitcoin/rust-bitcoincore-rpc)
library. This mock takes the advantage of `bitcoincore-rpc` trait interface,
called `RpcApi`. With this mock, every test can have an isolated Bitcoin
environment without changing your existing code too much.

## License

**(C) 2024 Chainway Limited** `bitcoin-mock-rpc` was developed by Chainway
Limited. While we plan to adopt an open source license, we have not yet selected
one. As such, all rights are reserved for the time being. Please reach out to us
if you have thoughts on licensing.
