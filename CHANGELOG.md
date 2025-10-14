# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial alpha release
- `LexKey` allocating API for ergonomic encoding
- `Encoder` zero-allocation API for hot-path performance
- Support for encoding strings, integers (signed/unsigned), floats, UUIDs, booleans, and composites
- Lexicographically sortable byte keys with natural ordering semantics
- Comprehensive encoding specification (SPEC.md)
- 100% test coverage across library sources
- Benchmarks for performance validation
- Examples demonstrating encoding patterns

### Design Decisions
- Encode-only stance (no generic decoding)
- Canonical numeric widths (narrower types widened for consistent ordering)
- NaN values not encodable (panic on attempt)
- No trailing separator in composites
- Safety-first approach with safe copy APIs
