# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-03-25

### Added

- `ShellCommand` struct with builder pattern for ergonomic command construction
- Serialization/deserialization support via optional `serde` feature
- Conversion to `std::process::Command` and `tokio::process::Command`
- Mutable handle and scoped mutation API for `ShellCommand`
- Tokio test support behind `tokio` feature flag
