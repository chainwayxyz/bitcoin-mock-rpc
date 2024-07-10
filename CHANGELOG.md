# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Taproot key path spending check added
- Commented out spending requirements are re-added

### Changed

- Ledger holds data in a SQLite database, not in memory
- Tag name in changelog fixed

### Removed

- UTXO management is removed, because it's not needed and can cause problems with other parts

## [0.0.1] - 2024-07-04

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

[Unreleased]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/0.0.1...HEAD
[0.0.1]: https://github.com/chainwayxyz/bitcoin-mock-rpc/releases/tag/0.0.1
