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

// Zero-allocation hot path using Encoder reuse
let mut enc = Encoder::with_capacity(64);
enc.encode_string_into("tenant");
enc.push_byte(LexKey::SEPARATOR);
enc.encode_i64_into(123);
let bytes = enc.freeze();
assert!(!bytes.is_empty());
```

## API surface

- `LexKey`
  - Allocating encoders: `encode_string`, `encode_u8`, `encode_u16`, `encode_u32`, `encode_u64`, `encode_i8`, `encode_i16`, `encode_i32`, `encode_i64`, `encode_f32`, `encode_f64`, `encode_uuid`, `encode_bool`, `encode_end_marker`, `encode_time_unix_nanos`, `encode_composite`, `encode_first`, `encode_last`.
  - Into-Vec encoders: `encode_u8_into`, `encode_u16_into`, `encode_u32_into`, `encode_u64_into`, `encode_i8_into`, `encode_i16_into`, `encode_i32_into`, `encode_i64_into`, `encode_f32_into`, `encode_f64_into`, `encode_bool_into`, `encode_uuid_into`, `encode_composite_into`.
  - Prefix/range Vec helpers: `prefix_successor`, `prefix_scan_bounds`, `prefix_end`, `range_upper_vec`, `prefix_range_bounds`, `range_bounds_vec`.
  - Accessors: `as_bytes`, `is_empty`, `to_hex_string`. Constants: `SEPARATOR=0x00`, `END_MARKER=0xFF`.
- `Encoder`
  - Lifecycle: `with_capacity`, `clear`, `freeze`, `into_vec`, `as_slice`, `push_byte`.
  - Writers: `push_separator`, `push_end_marker`, `encode_string_into`, `encode_bytes_into`, `encode_u8_into`, `encode_u16_into`, `encode_u32_into`, `encode_u64_into`, `encode_i8_into`, `encode_i16_into`, `encode_i32_into`, `encode_i64_into`, `encode_f32_into`, `encode_f64_into`, `encode_uuid_into_buf`, `encode_composite_into_buf`.

## Performance

- Reuse the same `Vec` or `Encoder` to see near zero-allocation performance. Representative times (on a typical machine):
  - Encoder reuse: u64/i64/f64/uuid ~0.7-0.9 ns; small string ~4 ns; composite (3–4 parts) ~8.6 ns.
  - Allocating convenience APIs: u64/i64/f64/string ~9-13 ns; composite (3 parts) ~18 ns.
  - Reused `Vec` composite writes are ~6.9 ns for 3 parts.
  - Owned storage-key output with `Encoder::into_vec` is ~12.2 ns for a Fitz-style `realm/domain/` prefix, versus ~24.3 ns through `freeze().to_vec()`.
  - Structured `prefix_end` is ~12.5 ns, versus ~27 ns through `encode_range_upper(...).as_bytes().to_vec()`; arbitrary raw-prefix `prefix_successor` is ~14 ns.

## Intentional design choices (read this!)

This crate is for building sortable keys, not round-trip decoding. Some choices are deliberate to keep the encoding simple, fast, and unambiguous:

- Separator is `0x00`: `SEPARATOR` is `0x00` and is used between composite parts. This mirrors the on-the-wire format and keeps composition simple. Don’t try to parse composites by splitting on `0x00` unless your schema dictates where to split; this crate no longer provides `encode_nil()` to avoid encouraging split-on-0x00 decoding.
- Encode-only stance: Composite parts may contain `0x00`. That’s fine for ordering but makes generic decoding ambiguous. This crate does not ship decoders; bring your own schema if you need parsing.
- Typed numeric widths: `Encodable` and `encode_composite!` preserve the Rust type width (`u8` → 1 byte, `u16` → 2, `u32`/`f32` → 4, `u64`/`i64`/`f64` → 8). This keeps typed product keys compact. Use explicit 64-bit values when cross-width canonicalization is required.
- NaN handling: NaN values are not encodable and will cause a panic. Use a schema-level presence/marker value to represent missing or invalid floats.
- No trailing separator: `encode_composite` inserts one `0x00` between each adjacent pair of parts but nothing after the last. Empty parts are preserved, so adjacent separators are possible. Use `encode_first`/`encode_last` to build prefix bounds.
- First/last markers: `encode_first` appends `SEPARATOR (0x00)` and `encode_last` appends `END_MARKER (0xFF)` to a prefix to construct structured composite range bounds.
- Prefix bounds: use `prefix_successor`/`prefix_scan_bounds` for arbitrary raw-byte prefix scans. Use `prefix_end`/`range_upper_vec` only for structured LexKey partition ranges where child keys continue below `0xFF`.
- Safety over micro-optimizations: Allocating and into-Vec paths use safe copies (`Bytes::copy_from_slice`, `Vec::extend_from_slice`). The reusable `Encoder` uses a `Vec<u8>` internally and freezes it into `Bytes` without copying the encoded payload.
- Owned storage keys: if the caller ultimately needs `Vec<u8>`, use `Encoder::into_vec` and the Vec-returning range helpers to avoid `Bytes` roundtrips.

## Safety & implementation notes

- Allocating and into-Vec encoders use safe copy APIs. The `Encoder` uses `Vec<u8>` writes and returns immutable `Bytes` without `unsafe`.
- Tests cover edge cases (narrow and 64-bit numerics, ±0, infinities, NaN, composites, macro evaluation, and range bounds).

## License

MIT
