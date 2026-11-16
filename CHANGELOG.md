# Changelog

## Unreleased

- Renamed `fs-activity` to `fs-activities` (second step after `fs-history`; ADR 0087, plural rule): crates `fs-activities-core` and `fs-activities-hub`, runtime name in `Capabilities::new` is now `fs-activities-core`. Types, contract and capability IDs stay `history` (wire unchanged).
- The runtime name in `Capabilities::new` (`crates/fs-activity-core/src/repo/provide.rs`) is now `fs-activity-core` (was `fs-history-core`), matching the crate name after the rename `fs-history` to `fs-activity` (ADR 0087). Everything on the wire stays as it was (`history.capnp`, capability IDs `history.*`, `PROGRAM = "history"`, MIME `vnd.freesynergy.history-*`).
