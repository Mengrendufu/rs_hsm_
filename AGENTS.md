# AGENTS.md -- rs_hsm_

## Scope

This file applies to the whole repository.

This crate is a minimal embedded-style hierarchical state machine engine. Keep
the implementation close to the local C `sm_hsm` design: static state
descriptors, function pointers, explicit event dispatch, fail-fast contracts,
and predictable stack/RAM use.

## Core Style

- Treat `src/` as `no_std` embedded runtime code. Do not introduce `std`, heap allocation, async, threads, channels, `Box`, `Rc`, `Arc`, trait objects, or smart-pointer ownership schemes in the engine.
- Prefer C-style explicitness over clever Rust abstraction. Static tables, plain structs, function pointers, and narrow public APIs are intentional.
- Preserve the existing `SM_` naming style in the runtime. The crate intentionally allows `non_camel_case_types` and `non_snake_case`.
- Keep state-machine behavior visible. Do not hide transitions behind generic helpers with vague names such as `process`, `handle_all`, or `execute`.
- Keep test/observation code out of the runtime. State-name lookup, trace formatting, reference sequences, and CLI selection helpers belong in examples/tests and should be marked as test-only.
- Do not add cleanup refactors unless they remove a real boundary problem. Small, explicit code is preferred over broad abstraction.

## Engine Model

The runtime is built from four pieces:

- `SM_HsmTrait`: a type-level interface that binds `ActiveObject`, `AO_Evt`, and `TOP_initial`.
- `SM_HsmState<T>`: one static state descriptor row. It stores parent, optional init, optional entry/exit, and handler pointers.
- `SM_Hsm<T, N>`: the embeddable engine. It stores only `curr`; it does not keep a C-style `next` field.
- `SM_RetState<T>`: handler result: `Handled`, `Super`, or `Tran(target)`.

The state graph must be static:

```rust
static AppIdle: SM_HsmState<AppSpec> = SM_HsmState {
    super_: None,
    init_: None,
    entry_: Some(AppIdle_entry_),
    exit_: Some(AppIdle_exit_),
    handler_: AppIdle_,
};
```

Handlers are plain functions:

```rust
fn AppIdle_(me: &mut App, e: &AppEvt) -> SM_RetState<AppSpec> {
    match e {
        AppEvt::Start => SM_RetState::Tran(&AppRunning),
        _ => SM_RetState::Super,
    }
}
```

## Active Object Pattern

Use the real embedded composition shape: the active object owns both user state
and the HSM engine.

```rust
pub struct App {
    sm_hsm_: SM_Hsm<AppSpec, 5>,
    foo: u8,
}

struct AppSpec;

impl SM_HsmTrait for AppSpec {
    type ActiveObject = App;
    type AO_Evt = AppEvt;

    fn TOP_initial(me: &mut Self::ActiveObject) -> SM_StatePtr<Self> {
        App_trace(me, "top-INIT.");
        &AppTopChild
    }
}
```

Because the engine is embedded in the active object, Rust cannot safely borrow
`self.sm_hsm_` and `self` mutably at the same time. If an AO method must call
the engine, keep the required `unsafe` local and documented:

```rust
pub fn dispatch(&mut self, e: &AppEvt) {
    let me = self as *mut Self;
    let sm_hsm_ = &mut self.sm_hsm_ as *mut SM_Hsm<AppSpec, 5>;

    // The embedded HSM mutates only its own current-state pointer while
    // state handlers mutate the active object.
    unsafe {
        (*sm_hsm_).dispatch(&mut *me, e);
    }
}
```

Do not wrap the AO in a separate `Harness` or `Object` layer for production
shape. If a wrapper exists only for tests, name and mark it as test-only.

## State Handler Rules

- Use `match event` or equivalent direct branching in handlers.
- Use guards directly at the transition decision point.
- Return `SM_RetState::Handled` when consumed locally.
- Return `SM_RetState::Super` to bubble to the parent.
- Return `SM_RetState::Tran(&TargetState)` for transitions.
- Do not mutate state during input cleanup/parsing code. Clean data first, then dispatch to state logic.
- Do not encode state as scattered booleans when it should be a state descriptor and handler.

## Memory Rules

- Default nest depth is `5`, matching the C `SM_MAX_NEST_DEPTH_` default. Do not raise it without a concrete hierarchy-depth reason.
- Path buffers are stack arrays: `[Option<SM_StatePtr<T>>; N]`. `Option<&'static T>` is expected to be pointer-sized.
- `SM_Hsm<T, N>` stores only `curr`. Do not reintroduce a `next` field unless the transition API changes require it.
- State descriptors should remain static. Do not allocate state descriptors at runtime.
- Keep runtime public APIs allocation-free and return-value-light. `dispatch()` intentionally returns `()`.

## Contracts And Assert Hook

Contract handling lives in `src/sm_assert.rs`.

- Use DBC macros internally: `DBC_ASSERT!`, `DBC_REQUIRE!`, `DBC_ENSURE!`, `DBC_INVARIANT!`, `DBC_ERROR!`.
- Contract payload is intentionally only `module + label`, aligned with SST-style `dbc_assert.h`.
- Do not reintroduce `SM_AssertKind`, `SM_AssertReason`, rich error enums, or per-trait assert hooks.
- All DBC failures go through system-level `SM_onAssert()`.
- Board/application code can install a system hook with `SM_setOnAssert()` during single-threaded startup.
- Contract violations are bugs or corrupted-state evidence. They should fail fast or enter the BSP fatal path, not become normal recoverable errors.

## Tests And Observation Code

The `hsmtst` fixture is both a semantic example and a test adapter.

- Code above the `Test/observation adapter` section models the real state machine.
- Code below that section is for tests/example output only: state-name lookup, trace access, sequence runner, and CLI sequence selection.
- Mark any new diagnostic or test-only helper with `TEST-ONLY`.
- Do not move state-name lookup or trace formatting into `src/lib.rs`.

## Verification

Before finalizing changes, run:

```bash
cargo fmt --check
cargo check --lib --no-default-features
cargo test --all-targets
git diff --check
```

For memory-sensitive changes, also check generated sizes or layout assumptions
instead of guessing.
