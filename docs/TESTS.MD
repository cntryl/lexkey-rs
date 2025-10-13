# Test Guidelines

These guidelines define how we write tests across the repository to keep them consistent, readable, and reliable.

## Naming

Use **behavior-first** test names:

```
should_<expected>_given_<preconditions>_when_<action>
```

Examples:

- `should_deliver_notice_given_active_subscription_when_published`
- `should_reject_pub_given_missing_auth_when_publishing`

Be descriptive and include what matters — clarity beats brevity.

## File & Directory Layout

- Every test file mirrors the **same path and name** as the file it tests under `src/`.
- Each file under `tests/` imports and validates the corresponding module from `src/`.

Example:

```
src/
  core/
    router.rs
    engine.rs
  transport/
    ws.rs
    tcp.rs

tests/
  core/
    router.rs
    engine.rs
  transport/
    ws.rs
    tcp.rs
```

This keeps ownership and discoverability 1:1 between source and tests.
Cross-cutting or end-to-end tests may live at the root of `/tests/` (e.g. `notice_flow.rs`).

## Single Behavior Principle (SBP)

Each test verifies **one behavior**.

- One **Act** per test.
- Multiple assertions are fine only if they describe the same behavior.
- Split tests if you’re checking unrelated logic.
- Use table-driven tests for multiple inputs of the same behavior.

### Single-Act tests — explicit guidance

To reduce reviewer cognitive load and make intent explicit, prefer tests that perform a single Action (Act) and then assert the resulting observable behavior. Put another way: "one Act, one behavior under test."

Why this matters:

- Tests that perform multiple Acts can hide which Act produced a failing assertion.
- Single-Act tests are easier to name, reason about, and maintain.

Bad example (anti-pattern — multiple Acts in one test):

```rust
#[test]
fn should_exercise_many_things() {
    // Arrange
    let mut tx = Transaction::new(1, 0);
    tx.put(Bytes::from_static(b"a"), Bytes::from_static(b"1"), None);
    tx.delete(Bytes::from_static(b"b"));

    // Act 1: query staged reads
    let got = tx.get_local(b"a");

    // Act 2: commit
    let muts = tx.commit().unwrap();

    // Assert: mixes expectations for two different behaviours
    assert_eq!(got, Some(Some(Bytes::from_static(b"1"))));
    assert_eq!(muts.len(), 2);
}
```

Good example (preferred — split into focused tests):

```rust
#[test]
fn should_resolve_put_on_get_local() {
    // Arrange
    let mut tx = Transaction::new(1, 0);
    tx.put(Bytes::from_static(b"a"), Bytes::from_static(b"1"), None);

    // Act
    let got = tx.get_local(b"a");

    // Assert
    assert_eq!(got, Some(Some(Bytes::from_static(b"1"))));
}

#[test]
fn should_return_staged_mutations_in_order_when_commit() {
    // Arrange
    let mut tx = Transaction::new(1, 0);
    tx.put(Bytes::from_static(b"a"), Bytes::from_static(b"1"), None);

    // Act
    let muts = tx.commit().unwrap();

    // Assert
    assert_eq!(muts.len(), 1);
}
```

Reviewer checklist for single-Act tests

- Does the test name describe a single observable behavior? (yes / no)
- Is there exactly one Act (call/operation) that produces the outcome being asserted? (yes / no)
- Do the assertions focus on a single behavior or closely related side-effects? (yes / no)
- If multiple behaviors are present, can the test be split into smaller tests? (yes / no)

If any of the answers is "no" then prefer splitting the test.

### Table-driven test example (async)

Below is a minimal pattern for a table-driven async test using `tokio::test` and `tokio::time::timeout`. It keeps each row focused on a single behavior and uses a small timeout to avoid flakiness.

```rust
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn should_return_expected_count_given_input_variations() {
    // Arrange: table of (name, input, expected)
    let cases = vec![
        ("empty", "", 0usize),
        ("single", "a", 1usize),
        ("many", "abc", 3usize),
    ];

    for (name, input, expected) in cases {
        // Use a sub-scope so each case has isolated variables
        let name = name.to_string();
        let input = input.to_string();

        // Act: run the behaviour under a timeout to prevent hangs
        let fut = async move {
            // replace `count_items` with the actual function under test
            let got = crate::your_module::count_items(&input).await;
            assert_eq!(got, expected, "case: {}", name);
        };

        // 200ms is usually sufficient; prefer small values and tune per-suite
        let res = timeout(Duration::from_millis(200), fut).await;
        assert!(res.is_ok(), "timeout in case: {}", name);
    }
}
```

Notes:

- Keep each table row small and focused on one behavior. If assertions diverge in intent, split into separate tests.
- Prefer `timeout` over `sleep` so the test fails fast on deadlocks or hangs.
- Consider extracting a small `tests::time` helper (e.g., `short_timeout() -> Duration`) if you need a project-wide standard.

## Structure: Arrange // Act // Assert

Organize each test with clear sections:

```rust
#[tokio::test]
async fn should_deliver_notice_given_active_subscription_when_published() {
    // Arrange
    let (addr, shutdown_tx, _store) = fitz::test_util::start_inprocess_broker().await;

    // Act
    // publish a notice...

    // Assert
    // expect ACK then DAT
}
```

Guidelines:

- Keep setup minimal and focused.
- One Act per test.
- Assert the main behavior, plus key side effects when relevant.
- Avoid mixing setup and assertions in the same block.

## Async & Time-Based Tests

- Use `#[tokio::test]` for async tests.
- Avoid arbitrary `sleep` — prefer timeouts or polling.

Example pattern:

```rust
use tokio::time::{timeout, Duration};

let res = timeout(Duration::from_millis(200), ws.next()).await;
assert!(res.is_err(), "unexpected message received");
```

Keep any required delay minimal (50–200 ms) and justify it in comments.

## Determinism & Isolation

- Tests must be **order-independent**.
- Avoid shared global state — use an in-process broker per test.
- No external dependencies beyond loopback WS/TCP.
- Clean up after each test (e.g., `shutdown_tx.send(())`).

## Assertions

- Assert only what proves correctness.
- Prefer exact matches when deterministic (frame type codes, specific TLVs).
- For opaque payloads, assert key presence rather than full byte matches.

## Negative & Edge Cases

Include explicit tests for:

- Auth failures and malformed payloads
- Unsubscribes and resubscribe flows
- Backpressure and bounded channel drops
- Timeout-based “no message expected” cases

Example:

```rust
let res = timeout(Duration::from_millis(200), ws.next()).await;
assert!(res.is_err(), "unexpected message received");
```

## Trait Behavior Tests

When a trait has multiple implementations, write tests that **enforce the trait’s behavior**, not each implementation’s internals.
All implementations must pass the same behavior contract.

### Principles

- Define shared tests describing _what the trait must do_.
- Reuse those tests for every implementation.
- Minimize duplication using helper functions or macros.
- Each behavior runs in isolation via a fresh instance.

### Example Pattern

```rust
trait Storage {
    fn put(&self, key: &str, value: &str);
    fn get(&self, key: &str) -> Option<String>;
}

// Shared behavior tests (factory creates a fresh instance)
fn run_storage_behavior_tests<F, T>(factory: F)
where
    F: Fn() -> T,
    T: Storage,
{
    // should_store_and_retrieve_value
    {
        // Arrange
        let store = factory();
        let key = "a";
        let val = "1";

        // Act
        store.put(key, val);
        let result = store.get(key);

        // Assert
        assert_eq!(result.as_deref(), Some("1"));
    }

    // should_return_none_given_missing_key
    {
        // Arrange
        let store = factory();

        // Act
        let result = store.get("missing");

        // Assert
        assert!(result.is_none());
    }
}

// Each implementation runs the same shared tests
#[test]
fn storage_behavior_in_memory() {
    run_storage_behavior_tests(|| InMemoryStore::new());
}

#[test]
fn storage_behavior_redis() {
    run_storage_behavior_tests(|| RedisStore::connect("localhost:6379"));
}
```

### Notes

- Each `{ … }` block defines one isolated behavior with its own `// Arrange // Act // Assert`.
- The factory ensures a **fresh instance per behavior** for full isolation.
- The shared helper enforces contract consistency across all implementations.

## Checklist

- [ ] File mirrors `src/` path and name
- [ ] Descriptive test name: `should_x_given_y_when_z`
- [ ] Single behavior (one Act)
- [ ] Clear `// Arrange // Act // Assert` sections
- [ ] Timeout instead of sleep
- [ ] Isolated broker per test
- [ ] Assert primary behavior and key side effects
- [ ] Shared trait tests use factory pattern for isolation

These guidelines evolve with the codebase.
When adding new subsystems (e.g., reliable notices), extend examples and update patterns as needed.

## CI / Local commands

Add these to CI or run locally to validate tests and coverage. They are the canonical commands we use during development and CI checks.

```powershell
# Run full test suite
cargo test --all

# Run single integration test file (example)
cargo test --test wal -- --nocapture

# Generate coverage (requires cargo-llvm-cov installed in CI runner)
cargo llvm-cov --tests
```

## Negative-test template

Use this minimal template for destructive/negative tests (corruption, truncation, invalid header). Keep Arrange/Act/Assert and isolate via temp dirs.

```rust
#[test]
fn should_error_on_corrupted_wal() {
    // Arrange: create a temp dir and write a valid file
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("wal.log");
    std::fs::write(&path, b"...valid wal bytes...").unwrap();

    // Act: corrupt a prefix or truncate the file
    let mut b = std::fs::read(&path).unwrap();
    b.truncate(4);
    std::fs::write(&path, &b).unwrap();

    // Assert: operation fails with a corruption error
    let res = crate::wal_fs::replay_wal_file(&path);
    assert!(res.is_err());
}
```
