# lexkey-rs

Lexicographically sortable byte keys for numbers, strings, UUIDs, and composites.

This crate offers two complementary APIs:
- `LexKey`: ergonomic, allocating constructors that return an immutable key.
- `Encoder`: a reusable buffer for zero-allocation hot paths.

## Quick start

```rust
use lexkey::{LexKey, Encoder};
use uuid::Uuid;

// Allocating convenience APIs
let k = LexKey::encode_i64(42);
assert!(k.as_bytes() < LexKey::encode_i64(100).as_bytes());

// Composite: parts joined with 0x00, no trailing separator
let user_id = Uuid::nil();
let comp = LexKey::encode_composite(&[b"tenant", b"user", user_id.as_bytes()]);
assert!(comp.as_bytes().windows(1).any(|w| w == [0x00]));

## Examples

See `examples/encode_demo.rs` for a small demo showing how to build composite keys and a recommended
pattern for encoding optional values (presence marker).

// Zero-allocation hot path using Encoder reuse
enc.encode_string_into("tenant");
enc.push_byte(LexKey::SEPARATOR);
enc.encode_i64_into(123);
assert!(!bytes.is_empty());
```

## API surface

- `LexKey`
	- Allocating encoders: `encode_string`, `encode_u64`, `encode_i64`, `encode_f64`, `encode_uuid`, `encode_bool`, `encode_end_marker`, `encode_time_unix_nanos`, `encode_composite`, `encode_first`, `encode_last`.
  - Into-Vec encoders: `encode_u64_into`, `encode_i64_into`, `encode_f64_into`, `encode_bool_into`, `encode_uuid_into`, `encode_composite_into`.
  - Accessors: `as_bytes`, `is_empty`, `to_hex_string`. Constants: `SEPARATOR=0x00`, `END_MARKER=0xFF`.
- `Encoder`
  - Lifecycle: `with_capacity`, `clear`, `freeze`, `as_slice`, `push_byte`.
  - Writers: `encode_string_into`, `encode_u64_into`, `encode_i64_into`, `encode_f64_into`, `encode_uuid_into_buf`, `encode_composite_into_buf`.

## Performance

- Reuse the same `Vec` or `Encoder` to see near zero-allocation performance. Representative times (on a typical machine):
  - Encoder reuse: u64/i64/f64/uuid ~6–7 ns; small string ~7–8 ns; composite (3–4 parts) ~28 ns.
  - Allocating convenience APIs are a bit slower due to buffer creation/copies.

## Intentional design choices (read this!)

This crate is for building sortable keys, not round-trip decoding. Some choices are deliberate to keep the encoding simple, fast, and unambiguous:

- Separator is `0x00`: `SEPARATOR` is `0x00` and is used between composite parts. This mirrors the on-the-wire format and keeps composition simple. Don’t try to parse composites by splitting on `0x00` unless your schema dictates where to split; this crate no longer provides `encode_nil()` to avoid encouraging split-on-0x00 decoding.
- Encode-only stance: Composite parts may contain `0x00`. That’s fine for ordering but makes generic decoding ambiguous. This crate does not ship decoders; bring your own schema if you need parsing.
- Canonical numeric widths: Narrower numeric types are widened (signed → i64, unsigned → u64, float32 → float64) so values compare consistently across widths. Example: `u32(1)` and `u64(1)` encode identically.
- Canonical NaN: All NaNs map to a single quiet NaN (`0x7FF8_0000_0000_0001`) to ensure deterministic ordering.
- NaN handling: NaN values are not encodable and will cause a panic. Use a schema-level presence/marker value to represent missing or invalid floats.
- No trailing separator: `encode_composite` inserts one `0x00` between adjacent parts but nothing after the last. Use `encode_first`/`encode_last` to build prefix bounds.
- First/last markers: `encode_first` appends `SEPARATOR (0x00)` and `encode_last` appends `END_MARKER (0xFF)` to a prefix to construct range bounds that sort before/after any extension of that prefix.
- Safety over micro-optimizations: As of 2025-10-13, allocating and into-Vec paths use safe copies (`Bytes::copy_from_slice`, `Vec::extend_from_slice`). The reusable `Encoder` already uses safe `BytesMut + BufMut` writes.

## Safety & implementation notes

- Allocating and into-Vec encoders use safe copy APIs. The `Encoder` uses `bytes::BytesMut` and `BufMut` for efficient writes without `unsafe`.
- Tests cover edge cases (i64 min/max, ±0, infinities, NaN) and composites; coverage is 100% across library sources.

## License

MIT

