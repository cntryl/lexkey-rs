# LexKey Encoding Specification

This document defines a precise, language-agnostic byte-level encoding for LexKey. It enables faithful implementations in other languages (C#, Python, Java, Rust, etc.) that produce identical bytes and maintain the same lexicographic ordering semantics.

Breaking change notice (2025-10-01):
- Canonical numeric width is now the default: narrower numerics are upcast before encoding
  - int, int8, int16, int32 → int64
  - uint8, uint16, uint32 → uint64
  - float32 → float64
- This ensures logical numeric ordering across widths (e.g., uint32(1) and uint64(1) encode identically).
- For legacy behavior that preserves native widths, see “Legacy native-width mode” below.

The goals of the encoding:
- Concatenate multiple typed values into a single byte sequence such that byte-wise lexicographic order matches natural value order.
- Be simple and fast to encode/compare (no decoding required for ordering).
- Provide predictable, fixed-width encodings for numeric types; direct copy for byte sequences.

Notation:
- All byte values are written in hexadecimal, e.g., 0x00.
- Big-endian means most-significant byte first.
- “Lexicographic order” refers to standard unsigned byte-wise comparison.

## Quick reference (default behavior)

| Category         | Types                                               | Default width | Transform (ordering)                                                                | Encoding |
|------------------|-----------------------------------------------------|---------------|-------------------------------------------------------------------------------------|----------|
| String           | string                                              | n bytes       | none                                                                                | raw bytes |
| Bytes            | []byte                                              | n bytes       | none                                                                                | raw bytes |
| UUID             | RFC4122 UUID                                        | 16 bytes      | none                                                                                | raw bytes |
| Boolean          | bool                                                | 1 byte        | false → 0x00; true → 0x01                                                           | single byte |
| Signed integers  | int, int8, int16, int32, int64, time.Duration       | 8 bytes       | widen to int64; XOR sign bit (u = uint64(v) XOR 0x8000000000000000)                 | big-endian |
| Unsigned integers| uint8, uint16, uint32, uint64                       | 8 bytes       | widen to uint64; no transform                                                       | big-endian |
| Floating-point   | float32, float64                                    | 8 bytes       | widen to float64; NaN is not encodable (error); v<0: NOT bits; v≥0: flip sign bit     | big-endian IEEE754 |
| Time instant     | time.Time (UTC)                                     | 8 bytes       | UnixNano as int64; XOR sign bit                                                     | big-endian |
| Nil              | nil                                                 | 1 byte        | 0x00                                                                                | single byte |
| End sentinel     | struct{}                                            | 1 byte        | 0xFF                                                                                | single byte |
| Part separator   | between parts                                       | 1 byte        | 0x00                                                                                | single byte |

Legacy note: In legacy mode, native widths are used (e.g., int16→2 bytes, int32→4, uint8→1, uint16→2, uint32→4, float32→4) with the same transforms per type. See the “Legacy native-width mode” and examples below.

## Special bytes
- Separator (also spelled Seperator in some code): 0x00 – inserted between parts in a composite key; also the encoding for nil and for boolean false.
- EndMarker: 0xFF – used by range upper bounds and by EncodeLast; also the encoding for the Go sentinel type struct{}.

These sentinel bytes are part of the on-the-wire format. Keep in mind that 0x00 may also appear as data (e.g., in strings/byte arrays, unsigned/signed integers with leading zero, false boolean, or nil). The LexKey design does not escape 0x00; decoding of composite parts is not generally supported (only specific helpers like PrimaryKey decode by splitting on the first 0x00).

## Composite keys (parts ...any)
- A LexKey is formed by concatenating encoded parts with a single 0x00 separator between adjacent parts.
- No trailing separator is appended after the last part.

Example ("foo", 42, true):
- "foo" bytes: 66 6f 6f
- separator: 00
- int64 42 (see signed integers): 80 00 00 00 00 00 00 2a
- separator: 00
- bool true: 01
- Result (hex): 66 6f 6f 00 80 00 00 00 00 00 00 2a 00 01

## Type encodings

### Strings
- Encoding: raw bytes of the string as-is. Recommended to use UTF-8, but any bytes are allowed.
- No length prefix or terminator.

### Byte arrays (byte[]/[]byte)
- Encoding: raw bytes as-is.
- No length prefix or terminator.

### UUID (128-bit)
- Encoding: 16 raw bytes in network order (RFC 4122). This matches the hyphenless lowercase hex form.
- Example: 550e8400-e29b-41d4-a716-446655440000 → 55 0e 84 00 e2 9b 41 d4 a7 16 44 66 55 44 00 00

### Booleans
- false → 0x00
- true  → 0x01

Note: 0x00 is indistinguishable from the separator byte at the byte level. This is okay because no type tag is used and ordering is preserved.

### Signed integers (int16, int32, int64, duration)
- Widths: int16 → 2 bytes, int32 → 4 bytes, int64 → 8 bytes.
- Endianness: big-endian.
- Ordering transform: XOR the sign bit to map the signed domain to monotonically increasing unsigned bytes.
  - For 64-bit: u = uint64(value) XOR 0x8000000000000000
  - For 32-bit: u = uint32(value) XOR 0x80000000
  - For 16-bit: u = uint16(value) XOR 0x8000
- Write u in big-endian.
- time.Duration is encoded exactly as int64 using the same transform.
- Plain int is encoded as int64.
- Default canonical width: any narrower signed integer is first widened to int64.

Rationale: flipping the sign bit yields an unsigned space that sorts lexicographically in the same order as the original signed values.

Examples:
- int64 123 → 80 00 00 00 00 00 00 7b
- int64 -123 → 7f ff ff ff ff ff ff 85
- int32 123 → 80 00 00 7b
- int16 -123 → 7f 85
- duration 42 → 80 00 00 00 00 00 00 2a

### Unsigned integers (uint8, uint16, uint32, uint64)
- Widths: 1, 2, 4, 8 bytes.
- Endianness: big-endian.
- No transform is applied.
- Default canonical width: any narrower unsigned integer is first widened to uint64.

Examples (default canonical width):
- uint8 123 → 00 00 00 00 00 00 00 7b
- uint16 123 → 00 00 00 00 00 00 00 7b
- uint32 123 → 00 00 00 00 00 00 00 7b
- uint64 123 → 00 00 00 00 00 00 00 7b

### Floating-point numbers (float32, float64)
- IEEE 754 binary32/binary64 bit patterns.
- Ordering transform to make lex order match numeric order:
  - If NaN: NaN values are not encodable; implementations SHOULD surface an error or use a schema-level presence/marker value to represent missing floats.
  - Else if value < 0: bitwise NOT of all bits (bits = ^bits)
  - Else (value >= 0): flip the sign bit (bits = bits XOR signBit)
- Write the resulting bits in big-endian.
- Default canonical width: float32 is first widened to float64 before applying the transform.

Notes:
- This transformation yields a total order consistent with numeric order and places all NaNs at the high end of the positive range (above +Inf), using a single canonical NaN encoding.
- +0 sorts after negatives and before positive values, as desired.

Examples (default canonical width):
- float32 +3.14 → C0 09 1E B8 60 00 00 00  (float64 of 3.14f32, then transformed)
- float32 −3.14 → 3F F6 E1 47 9F FF FF FF  (float64 of −3.14f32, then transformed)
- float64 +3.14 → C0 09 1E B8 51 EB 85 1F
-- float64 NaN  → Not encodable (error); use a presence/marker in the schema

### time instants (time.Time / DateTime)
- Encode the UTC Unix time in nanoseconds as a signed 64-bit integer, then apply the signed int64 transform (XOR with 0x8000000000000000) and write big-endian.
- Example:
  - 1970-01-01T00:00:00Z → 80 00 00 00 00 00 00 00
  - 2023-11-14T22:13:20Z (1700000000 seconds) → 97 97 9c fe 36 2a 00 00

### Nil (null)
- Encoded as a single byte 0x00.

### End sentinel (struct{})
- Encoded as a single byte 0xFF.
- Used internally for range upper bounds; not typically used in user keys.

## Range boundaries and helpers

### EncodeFirst(parts…)
- Build the prefix from parts, then append 0x00.
- Result sorts before any key that extends the same prefix.

### EncodeLast(parts…)
- Build the prefix from parts, then append 0xFF.
- Result sorts after any key that extends the same prefix.

### Primary keys
- Encode(partitionKey, rowKey) with a single 0x00 separator between them.
- Example: partition="partition" (70 61 72 74 69 74 69 6f 6e), row="row" (72 6f 77)
  - Encoded: 70 61 72 74 69 74 69 6f 6e 00 72 6f 77

Decoding (PrimaryKey only):
- Split on the first 0x00. Bytes before are the partition key; bytes after are the row key.
- Caveat: This requires that the partition key does not itself contain an embedded 0x00 byte if you intend to decode it this way.

### RangeKey boundaries
Given a partition key P and row key bounds [L, U]:
- Lower bound with partition: P || 0x00 || L (if L is empty, just P || 0x00)
- Upper bound with partition: P || 0x00 || U || 0xFF (if U is empty, just P || 0xFF)

This yields an inclusive/exclusive range suitable for lexicographic scans: [lower, upper).

## Comparison
- Compare two LexKeys using unsigned byte-wise comparison (e.g., memcmp / bytes.Compare). No decoding is necessary.
- The transforms above guarantee that numeric/time values sort correctly in lex order.

### Cross-type numeric ordering
- With canonical numeric width (default), mixed-width numerics sort logically across widths because they encode to the same width.
- If you must preserve legacy native-width bytes, see the legacy mode below; ordering across types is not guaranteed there.

## Legacy native-width mode
For compatibility with older keys or systems expecting native widths:
- Encode numeric types at their native widths (uint32 → 4 bytes, float32 → 4 bytes, etc.).
- Behavior matches the earlier (pre-2025-10-01) library. Note that cross-width numeric ordering is not guaranteed in this mode.

## JSON and hex helpers (optional, for reference)
- Hex string format: lowercase, no 0x prefix, even length.
- JSON representation: a JSON string containing the lowercase hex form; JSON null maps to an empty key.

These are convenience formats only; they do not affect the core byte-level specification.

## Test vectors
Use these to validate cross-language implementations.

Single values (default canonical width):
- "hello" → 68 65 6c 6c 6f
- UUID 550e8400-e29b-41d4-a716-446655440000 → 55 0e 84 00 e2 9b 41 d4 a7 16 44 66 55 44 00 00
- int64 123 → 80 00 00 00 00 00 00 7b
- int64 −123 → 7f ff ff ff ff ff ff 85
- int32 −123 → 7f ff ff ff ff ff ff 85  (widened to int64)
- uint8 255 → 00 00 00 00 00 00 00 ff  (widened to uint64)
- uint16 123 → 00 00 00 00 00 00 00 7b  (widened to uint64)
- uint32 123 → 00 00 00 00 00 00 00 7b  (widened to uint64)
- uint64 123 → 00 00 00 00 00 00 00 7b
- float32 +3.14 → c0 09 1e b8 60 00 00 00  (widened to float64)
- float32 −3.14 → 3f f6 e1 47 9f ff ff ff  (widened to float64)
- float64 +3.14 → c0 09 1e b8 51 eb 85 1f
- bool false → 00
- bool true → 01
- time.Unix(0,0) → 80 00 00 00 00 00 00 00
- time.Unix(1700000000,0) → 97 97 9c fe 36 2a 00 00
- duration 42 → 80 00 00 00 00 00 00 2a
- nil → 00
- struct{} (end sentinel) → ff

Composite:
- ("foo", 42, true) → 66 6f 6f 00 80 00 00 00 00 00 00 2a 00 01
- PrimaryKey("partition","row") → 70 61 72 74 69 74 69 6f 6e 00 72 6f 77
- Range lower with partition and start="start": 70 61 72 74 00 73 74 61 72 74
- Range upper with partition and end="end": 70 61 72 74 00 65 6e 64 ff

Legacy native-width examples (for reference/compat only):
- int32 −123 → 7f ff ff 85
- int16 −123 → 7f 85
- uint8 255 → ff
- uint16 123 → 00 7b
- uint32 123 → 00 00 00 7b
- float32 +3.14 → c0 48 f5 c3
- float32 −3.14 → 3f b7 0a 3c

## Reference pseudocode

Signed int64 (big-endian):
```
function encodeInt64(v):
    u = uint64(v) XOR 0x8000000000000000
    return toBigEndianBytes(u, 8)
```

Float64 (big-endian):
```
function encodeFloat64(x):
    if isNaN(x):
        bits = 0x7FF8000000000001
    else:
        bits = ieee754Bits(x)  // 64-bit
        if x < 0:
            bits = NOT bits
        else:
            bits = bits XOR 0x8000000000000000
    return toBigEndianBytes(bits, 8)
```

Time instant:
```
function encodeTimeUTC(t):  // t in nanoseconds since Unix epoch
    return encodeInt64(t)
```

Composite key:
```
function encodeKey(parts):
    out = []
    for i, p in enumerate(parts):
        out += encodePart(p)
        if i < len(parts)-1:
            out += [0x00]
    return out
```

Range bounds with partition P and row bounds [L, U]:
```
lower = P + [0x00] + L
upper = P + [0x00] + U + [0xFF]
# If L is empty: lower = P + [0x00]
# If U is empty: upper = P + [0xFF]
```

## Implementation notes
- Use constant-time byte copies for strings/byte arrays and UUIDs.
- Always write integers/floats/times in big-endian after applying the transforms.
- Do not allocate when avoidable on hot paths; pre-size buffers when possible (e.g., size = sum(part sizes) + separators).
- For comparisons/sorting, use direct byte-wise comparisons (e.g., memcmp), not string conversions.

This specification is derived from the reference Go implementation in this repository and is intended to be stable and portable.