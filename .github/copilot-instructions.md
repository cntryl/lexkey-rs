# GitHub Copilot Instructions for the Shale Project

Purpose: provide strict, machine-checkable guidelines for generating tests. Follow these exactly — the repo includes a meta-test that enforces them.

## Quick Summary
- **Test names:** must start with `should_` (never `test_`).
- **Long tests (>5 lines):** must include exactly `// Arrange`, `// Act`, `// Assert` (these exact comments).
- **One behavior per test:** split different inputs into separate tests.
- **Single `// Act`:** do not use multiple `// Act` sections in one test.
- **Small tests (≤5 lines):** may omit AAA comments but must still use `should_` naming.

## Detailed Rules (Read Carefully)

1) Naming convention (MANDATORY)

- Use the `should_*` pattern: `should_{action}_{condition}_given_{context}`.
- Never use names that start with `test_` (the meta-test will fail such tests).

Example:

```rust
// ✅ CORRECT
#[test]
fn should_return_value_when_key_exists() { }

// ❌ WRONG - Will fail meta-test!
#[test]
fn test_get_value() { }
```

2) AAA structure (MANDATORY for tests >5 lines)

- If a test is more than 5 lines, it must contain exactly these three comment lines (no variants or combined forms):

  // Arrange
  // Act
  // Assert

- Do not combine sections (`// Arrange & Act`, `// Act & Assert`) and do not use alternative text like `// Setup`.

Correct pattern:

```rust
#[test]
fn should_do_something() {
    // Arrange
    let setup = create_test_data();

    // Act
    let result = perform_operation(setup);

    // Assert
    assert_eq!(result, expected);
}
```

3) Single-behavior principle (MANDATORY)

- Each test should verify one behavior. If you need to assert different input→output mappings, create separate tests.

Wrong (three different inputs in one test):

```rust
#[test]
fn should_return_files_at_level() {
    let l0 = manifest.files_at_level(0);
    let l1 = manifest.files_at_level(1);
    let l2 = manifest.files_at_level(2);
    assert_eq!(l0.len(), 1);
    assert_eq!(l1.len(), 2);
    assert_eq!(l2.len(), 0);
}
```

Correct — split into focused tests:

```rust
#[test]
fn should_return_files_at_level_zero() {
    // Arrange
    let manifest = setup_with_level_0_files();

    // Act
    let result = manifest.files_at_level(0);

    // Assert
    assert_eq!(result.len(), 1);
}
```

Exception: Multiple assertions are allowed if they verify different facets of the same operation (same Arrange+Act).

4) No multiple `// Act` sections

- A single test must not contain more than one `// Act`. If you need multiple operations, split them into separate tests.

5) Small tests (≤5 lines)

- May omit AAA comments but still must use `should_` naming and focus on a single behavior.

## Common Patterns and Examples

Serialization / deserialization — keep separate tests for each direction:

```rust
#[test]
fn should_serialize_manifest() {
    // Arrange
    let manifest = create_manifest();

    // Act
    let result = serde_json::to_string(&manifest);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn should_deserialize_manifest() {
    // Arrange
    let original = create_manifest();
    let json = serde_json::to_string(&original).unwrap();

    // Act
    let deserialized: Manifest = serde_json::from_str(&json).unwrap();

    // Assert
    assert_eq!(deserialized.id, original.id);
}
```

Table-driven tests are acceptable when the same operation is being validated across inputs.

## Quick Checklist for Copilot (copy before suggesting a test)

- **Name:** starts with `should_`.
- **Long test (>5 lines):** contains `// Arrange`, `// Act`, `// Assert` (exact strings).
- **Single `// Act`:** confirm only one Act section.
- **Single behavior:** the test verifies exactly one behavior.
- **Multiple assertions:** only if they are facets of the same operation.

## Meta-test enforcement

All tests are validated by `tests/test_guidelines_compliance.rs`. The meta-test will FAIL if:

- any test uses `test_*` naming;
- tests >5 lines are missing exact AAA comments;
- tests combine AAA comments (e.g. `// Arrange & Act`).

Run the meta-test with:

```powershell
cargo test test_guidelines_compliance
```

## Good examples in this repo

- `src/manifest.rs` — good AAA structure
- `src/index/range_tombstone.rs` — single-behavior tests
- `src/cloud/mock.rs` — upload/download split into focused tests

## Rationale

- **Consistency:** easier review and maintenance
- **Debuggability:** failing tests point to a single behavior
- **CI safety:** meta-test keeps the suite consistent

## Final note

When in doubt, prefer more smaller tests rather than fewer large tests. If you want, I can run `cargo test test_guidelines_compliance` locally and report results.

