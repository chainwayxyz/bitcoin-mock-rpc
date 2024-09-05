# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- RPC server interface is disabled by default
  - It can be enabled by a feature flag

## [0.0.10] - 2024-09-04

### Changed

- If there is a connection to ledger while calling `Client::new()`, database won't get cleaned
- Block lock checks fixed

## [0.0.9] - 2024-09-03

### Added

- Simple UTXO management

### Changed

- Improved logging
- Error on unimplemented RPC argument
- Improved main binary CLI interface

## [0.0.8] - 2024-08-28

### Added

- Logging
  - RPC server binary can output logs
  - Tests print logs
- 2 new RPC functions
  - fund_raw_transaction and sign_raw_transaction_with_wallet
- Coinbase transaction
- RPC log middleware
- Encode/decode utilities

### Changed

- Fixed wrong encode/decode operations in RPC functions
- Fixed RPC server not responding when spawned in same process
- RPC server ports are assigned by OS

## [0.0.7] - 2024-08-15

### Added

- 4 new RPC functions
  - getbestblockhash, getblock, getblockheader and getblockcount
- RPC interface
  - Can be used as a standalone binary (much like the real Bitcoin)
  - Can be spawned multiple times independently in a binary (for sandboxed tests)

## [0.0.6] - 2024-08-06

### Added

- Block support
  - Block bodies are stored in ledger
  - Block hash calculation

### Changed

- Removed some of the unnecessary third-party libraries
- Some of the error messages are detailed
- Block height and time storage simplified in ledger
- RPC functions now return block related informations

## [0.0.5] - 2024-07-29

### Changed

- Fixed the way of relative time locks handled

### Removed

- UTXO related database tables and codes

## [0.0.4] - 2024-07-25

### Added

- Custom cloning support
  - Users of this library can establish a new connection while preserving ledger database
- Block have time
  - Blocks have fixed 10 minute interval
  - First block have current time
- Initial relative lock time support
  - Currently only supports script CSV

### Changed

- Script CSV is now relative, rather than absolute

## [0.0.3] - 2024-07-16

### Added

- Initial block implemenatation
  - Only block height is held in ledger
- Mempool implementation
  - Blocks can be mined using `generate_to_address()` call
- CSV block height check in ledger level

### Changed

- Script execution has it's own module and scripts aren't directly executed in spending reqirements

## [0.0.2] - 2024-07-10

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

[Unreleased]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.10...HEAD
[0.0.10]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.9...v0.0.10
[0.0.9]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.8...v0.0.9
[0.0.8]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.7...v0.0.8
[0.0.7]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/0.0.1...v0.0.2
[0.0.1]: https://github.com/chainwayxyz/bitcoin-mock-rpc/releases/tag/0.0.1
