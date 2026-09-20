# Review Skill Evaluation Inputs

These are independent synthetic review requests, not product implementation.
Review only the displayed changes and stated contracts. Unshown startup,
resource bounds, validated domain types, and infrastructure are unchanged and
out of scope. Locations are this document's case and code lines. Report no
candidate when the evidence supports none; do not infer defects in omitted code.

## Case A

Rust contract: `committed` means the receipt was durably saved. `Store::commit`
returns an error if nothing was persisted. The caller propagates returned
errors and logs them; it cannot detect an error converted into success here.
Existing tests exercise only a successful store and assert the returned
receipt and committed event.

```rust
fn submit(store: &mut Store, events: &mut Events, receipt: Receipt)
    -> Result<Receipt, StoreError>
{
    let _ = store.commit(&receipt);
    events.record("command_committed", &receipt.id);
    Ok(receipt)
}
```

## Case B

Rust contract: the execution boundary owns the final diagnostic. `accept` only
persists and returns an outcome. Event fields include the operation identity
and safe typed reason. The tests below are the required scenarios for this
small change; durable storage itself is unchanged and verified independently.

```rust
fn accept(store: &mut Store, receipt: &Receipt) -> Result<(), StoreError> {
    store.commit(receipt)?;
    Ok(())
}

fn run(store: &mut Store, events: &mut Events, receipt: &Receipt)
    -> Result<(), StoreError>
{
    match accept(store, receipt) {
        Ok(()) => { events.committed(&receipt.id); Ok(()) }
        Err(error) => {
            events.failed(&receipt.id, error.safe_reason());
            Err(error)
        }
    }
}

#[test]
fn failure_is_returned_and_reported_once() {
    let (mut store, mut events, receipt) = fixture_with_failed_commit();
    assert_eq!(run(&mut store, &mut events, &receipt), Err(StoreError::Unavailable));
    assert!(store.persisted().is_empty());
    assert_eq!(events.items(), &[failed_event(&receipt.id, "unavailable")]);
}

#[test]
fn success_is_reported_after_persistence() {
    let (mut store, mut events, receipt) = fixture_with_successful_commit();
    assert_eq!(run(&mut store, &mut events, &receipt), Ok(()));
    assert_eq!(store.persisted(), &[receipt.clone()]);
    assert_eq!(events.items(), &[committed_event(&receipt.id)]);
}
```

## Case C

Rust contract: `freeze` creates an independent immutable snapshot from a
bounded collection. Each `Entry` owns its data without shared mutable children.
The source remains usable and mutable. Tests mutate the source after freezing
and confirm the snapshot remains equal to the independently specified input.
No performance bottleneck has been observed.

```rust
fn freeze(source: &[Entry]) -> Snapshot {
    Snapshot { entries: source.to_vec() }
}
```

## Case D

Lua contract: the result must remain an independent snapshot after mutation of
`live.entries`. Replaying the same key/value set must produce identical encoded
bytes in lexical key order. Values are normalized strings; `encode` performs
only ordinary escaping. Existing tests create one entry, immediately encode,
and compare that one output. No test mutates the source or uses multiple keys.

```lua
local function freeze(live)
    return { entries = live.entries }
end

local function serialize(snapshot)
    local parts = {}
    for key, value in pairs(snapshot.entries) do
        parts[#parts + 1] = encode(key, value)
    end
    return table.concat(parts)
end
```

## Case E

PowerShell contract: validation failure must make this CLI fail, and stdout
contains exactly one machine-readable result object. The native validator
returns 7 on failure and can print progress on stdout. In this established
environment nonzero native exit does not automatically throw. This function
is the final CLI step; nothing checks its native exit status afterward.
The existing test uses a validator that exits 0 without output and checks
only the final `ok` value. Paths, startup logging, and argument construction
are already validated elsewhere and are outside this change.

```powershell
& $validator @validatorArgs
Write-Output '{"ok":true}'
exit 0
```

## Case F

Act as the review lead for round two. The task base is `fixture-base`, the
previously reviewed revision is `fixture-r1`, and the current revision is
`fixture-r2`. These are synthetic labels, not Git objects. The task adds both
`limit.rs` and `refresh.rs` below; everything else is unchanged and outside
scope. The round-one report contains only `r1.1`: `limit.rs` accepts zero.
The developer says that finding is fixed. Since round one only `limit.rs`
changed. The task requires a positive limit and replacement state to be
published only after successful validation; a failure must preserve the last
valid state. Error reporting is owned by the caller, which cannot undo a
published replacement. Existing tests exercise successful refresh and zero
limit rejection, but no refresh validation failure.

`limit.rs` at `fixture-r1`:

```rust
fn valid_limit(limit: u32) -> bool {
    limit <= 100
}
```

`limit.rs` at `fixture-r2`:

```rust
fn valid_limit(limit: u32) -> bool {
    (1..=100).contains(&limit)
}
```

`refresh.rs` at both `fixture-r1` and `fixture-r2`:

```rust
fn refresh(current: &mut Snapshot, replacement: Snapshot) -> Result<(), Error> {
    *current = replacement;
    validate(current)?;
    Ok(())
}
```

A legacy formatter has an unrelated known defect, predates `fixture-base`,
and is neither touched nor used by this task. It is outside the supplied diff.
Return the round-two report and the scope actually examined. Do not modify
the fixture or claim real storage/game execution.
