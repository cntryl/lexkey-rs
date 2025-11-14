# LexKey Encoding Specification

This document defines a precise, language-agnostic byte-level encoding for LexKey. It enables faithful implementations in other languages (C#, Python, Java, Rust, etc.) that produce identical bytes and preserve the same lexicographic ordering semantics.

Breaking change (2025-10-01): canonical numeric widths are now the default. Narrower numerics are widened before encoding:

- Signed ints: int, int8, int16, int32 → int64
- Unsigned: uint8, uint16, uint32 → uint64
- Floats: float32 → float64

This ensures logical ordering across widths (for example, `uint32(1)` and `uint64(1)` encode identically). See “Legacy native-width mode” for compatibility behavior.

Goals

- Produce byte sequences where unsigned byte-wise lexicographic order matches the natural order of encoded values.
- Keep encodings simple and fast to produce/compare (no decoding needed for ordering).
- Use fixed-width encodings for numeric types; copy bytes for strings/byte arrays.

Conventions

- Hex bytes are shown lowercase and space-separated, e.g. `0x00` → `00`.
- Big-endian = most-significant byte first.
- “Lexicographic order” means unsigned byte-wise comparison (memcmp, bytes.Compare, etc.).

## Quick reference (default behavior)

- String: raw bytes, `n` bytes, no transform.
- Bytes (`[]byte`): raw bytes, `n` bytes, no transform.
- UUID (RFC4122): 16 bytes, raw bytes.
- Boolean: 1 byte — `false` → `00`, `true` → `01`.
- Signed integers: canonical width 8 bytes (int64). Transform: `u = uint64(v) XOR 0x8000000000000000`, write big-endian.
- Unsigned integers: canonical width 8 bytes (uint64), big-endian, no transform.
- Floating-point: canonical width 8 bytes (float64). Transform: if NaN → error (use schema marker); else if negative → bitwise NOT; else → flip sign bit; write big-endian.
- Time instants: Unix nanoseconds encoded as signed int64, then signed-int transform and big-endian.
- Nil: single byte `00`.
- End sentinel: single byte `ff`.
- Part separator (composite): single byte `00` between adjacent parts; no trailing separator.

Legacy mode preserves native widths (int16 → 2 bytes, int32 → 4 bytes, float32 → 4 bytes, etc.). See “Legacy native-width mode”.

## Special bytes

- Separator: `00` — inserted between parts in a composite and used to encode `nil` and boolean `false`.
- End marker: `ff` — used for range upper bounds / `encode_last`.

Note: `00` can appear as data (strings, byte arrays, numeric leading zero bytes). LexKey does not escape `00`; decoding composite parts is not generally supported except where helpers explicitly split (e.g., PrimaryKey splitting on the first `00`).

## Composite keys

- Concatenate encoded parts with a single `00` separator between adjacent parts.
- Do not append a trailing separator after the last part.

Example — parts `("foo", 42, true)`:

- `"foo"` bytes: `66 6f 6f`
- separator: `00`
- int64 42: `80 00 00 00 00 00 00 2a`
- separator: `00`
- bool true: `01`
- Result (hex): `66 6f 6f 00 80 00 00 00 00 00 00 2a 00 01`

## Type encodings

Strings

- Raw bytes of the string. Recommended UTF-8 but any bytes allowed. No length prefix or terminator.

Byte arrays

- Raw bytes. No length prefix or terminator.

UUID (128-bit)

- 16 raw bytes in network order (RFC4122), matching the hyphenless lowercase hex form.
- Example: `550e8400-e29b-41d4-a716-446655440000` → `55 0e 84 00 e2 9b 41 d4 a7 16 44 66 55 44 00 00`.

Booleans

- `false` → `00`
- `true`  → `01`

Signed integers (int16, int32, int64, duration)

- Widths: int16 → 2 bytes, int32 → 4 bytes, int64 → 8 bytes. Default canonical width: widen to int64.
- Endianness: big-endian.
- Transform: XOR the sign bit to map signed domain to monotonic unsigned order:
  - 64-bit: `u = uint64(value) XOR 0x8000000000000000`
  - 32-bit: `u = uint32(value) XOR 0x80000000`
  - 16-bit: `u = uint16(value) XOR 0x8000`

Examples

- `int64 123` → `80 00 00 00 00 00 00 7b`
- `int64 -123` → `7f ff ff ff ff ff ff 85`

Unsigned integers (uint8, uint16, uint32, uint64)

- Widths: 1, 2, 4, 8 bytes. Default canonical width: widen to uint64 (8 bytes) in big-endian.

Examples (canonical width)

- `uint8 123` → `00 00 00 00 00 00 00 7b`
- `uint64 123` → `00 00 00 00 00 00 00 7b`

Floating-point numbers (float32, float64)

- Use IEEE 754 binary64 (canonical width). Transform for ordering:
  - If NaN: not encodable (implementations should return an error or use a schema-level marker).
  - If value < 0: bitwise NOT of all bits.
  - Else: flip the sign bit (XOR signBit).
- Write the transformed bits in big-endian.

Notes

- The transform yields a total order consistent with numeric order; all NaNs are effectively excluded (or represented by a schema marker).
- `-0.0` sorts before `+0.0`.

Examples (canonical width)

- `float64 +3.14` → `c0 09 1e b8 51 eb 85 1f`
- `float64 -3.14` → `3f f6 e1 47 9f ff ff ff`

Time instants (time.Time / DateTime)

- Encode UTC Unix time in nanoseconds as signed int64, then apply the signed-int transform (XOR with `0x8000000000000000`) and write big-endian.

Examples

- `1970-01-01T00:00:00Z` → `80 00 00 00 00 00 00 00`
- `time.Unix(1700000000,0)` → `97 97 9c fe 36 2a 00 00`

Nil (null)

- Encoded as a single byte `00`.

End sentinel (struct{})

- Encoded as a single byte `ff`.

## Range boundaries and helpers

EncodeFirst(parts…)

- Build the prefix from parts, then append `00`. Result sorts before any key that extends the same prefix.

EncodeLast(parts…)

- Build the prefix from parts, then append `ff`. Result sorts after any key that extends the same prefix.

Primary keys

- `Encode(partitionKey, rowKey)` uses a single `00` separator between them.
- Example: `partition="partition"`, `row="row"` → `70 61 72 74 69 74 69 6f 6e 00 72 6f 77`.

Decoding (PrimaryKey only)

- Split on the first `00`: bytes before are the partition key; bytes after are the row key. This requires the partition key not to contain `00` if you intend to decode it this way.

RangeKey boundaries

- Given partition `P` and row bounds `[L, U]`:
  - Lower bound: `P || 00 || L` (if `L` is empty, `P || 00`).
  - Upper bound: `P || 00 || U || ff` (if `U` is empty, `P || ff`).

This yields an inclusive/exclusive range suitable for lexicographic scans: `[lower, upper)`.

## Comparison

- Compare two LexKeys using unsigned byte-wise comparison (e.g., `memcmp`). No decoding required.

Cross-type numeric ordering

- With canonical numeric width, mixed-width numerics sort logically across widths (they encode to the same width).

Legacy native-width mode

For compatibility with older keys or systems expecting native widths:

- Encode numeric types at their native widths (e.g., `uint32` → 4 bytes, `float32` → 4 bytes).
- Behavior matches pre-2025-10-01 libraries; cross-width ordering is not guaranteed.

## JSON and hex helpers (optional)

- Hex: lowercase, no `0x` prefix, even length.
- JSON: represent keys as lowercase hex strings; JSON `null` → empty key.

These are convenience formats; they do not change the wire format.

## Test vectors

Single values (canonical width)

- `"hello"` → `68 65 6c 6c 6f`
- UUID `550e8400-e29b-41d4-a716-446655440000` → `55 0e 84 00 e2 9b 41 d4 a7 16 44 66 55 44 00 00`
- `int64 123` → `80 00 00 00 00 00 00 7b`
- `int64 -123` → `7f ff ff ff ff ff ff 85`
- `uint8 255` → `00 00 00 00 00 00 00 ff` (widened to uint64)
- `float64 +3.14` → `c0 09 1e b8 51 eb 85 1f`
- `bool false` → `00`
- `bool true` → `01`
- `time.Unix(0,0)` → `80 00 00 00 00 00 00 00`
- `duration 42` → `80 00 00 00 00 00 00 2a`
- `nil` → `00`
- `struct{}` (end sentinel) → `ff`

Composite

- `("foo", 42, true)` → `66 6f 6f 00 80 00 00 00 00 00 00 2a 00 01`
- PrimaryKey(`"partition"`, `"row"`) → `70 61 72 74 69 74 69 6f 6e 00 72 6f 77`
- Range lower with partition and start=`"start"`: `70 61 72 74 00 73 74 61 72 74`
- Range upper with partition and end=`"end"`: `70 61 72 74 00 65 6e 64 ff`

Legacy native-width examples (reference only)

- `int32 -123` → `7f ff ff 85`
- `int16 -123` → `7f 85`
- `uint8 255` → `ff`
- `float32 +3.14` → `c0 48 f5 c3`

## Reference pseudocode

Signed int64 (big-endian):

```text
function encodeInt64(v):
    u = uint64(v) XOR 0x8000000000000000
    return toBigEndianBytes(u, 8)
```

Float64 (big-endian):

```text
function encodeFloat64(x):
    if isNaN(x):
        error("NaN not encodable; use schema marker")
    bits = ieee754Bits(x)  // 64-bit
    if x < 0:
        bits = NOT bits
    else:
        bits = bits XOR 0x8000000000000000
    return toBigEndianBytes(bits, 8)
```

Time instant:

```text
function encodeTimeUTC(t):  // t in nanoseconds since Unix epoch
    return encodeInt64(t)
```

Composite key:

```text
function encodeKey(parts):
    out = []
    for i, p in enumerate(parts):
        out += encodePart(p)
        if i < len(parts)-1:
            out += [0x00]
    return out
```

Range bounds with partition P and row bounds [L, U]:

```text
lower = P + [0x00] + L
upper = P + [0x00] + U + [0xFF]
# If L is empty: lower = P + [0x00]
# If U is empty: upper = P + [0xFF]
```

## Implementation notes

- Use efficient byte copies for strings/byte arrays and UUIDs.
- Always write integers/floats/times in big-endian after applying transforms.
- Avoid allocations on hot paths; pre-size buffers when possible (sum of part sizes + separators).
- Compare keys using direct byte-wise comparisons, not string conversions.

This specification is derived from the reference Go implementation in this repository and is intended to be stable and portable.