# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial Bitcoin ledger that supports basic transaction management
  - Input output check
  - Address based UTXO management
- Initial RpcApi trait implementation
  - `send_raw_transaction`
  - `get_raw_transaction`
  - `get_raw_transaction_info`
  - `get_transaction`
  - `send_to_address`: Sends the amount to address, regardless of the balance
  - `get_new_address`
  - `generate_to_address`
  - `get_balance`

[unreleased]: https://github.com/chainwayxyz/bitcoin-mock-rpc
