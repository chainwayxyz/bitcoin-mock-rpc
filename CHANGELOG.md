# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.5...HEAD
[0.0.5]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/chainwayxyz/bitcoin-mock-rpc/compare/0.0.1...v0.0.2
[0.0.1]: https://github.com/chainwayxyz/bitcoin-mock-rpc/releases/tag/0.0.1
