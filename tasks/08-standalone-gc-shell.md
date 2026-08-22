# Iteration 08 — Standalone GC shell

## Goal

Return real interface objects to CoreCLR while every unsupported operation fails explicitly.

## Assignment

- [ ] Implement the required pinned `IGCHeap`, `IGCHandleManager`, and `IGCHandleStore` surface in the C++ shim.
- [ ] Keep interface objects thin and delegate state and policy to Rust through the existing C ABI.
- [ ] Wrap the supplied `IGCToCLR` callbacks without leaking C++ layout into Rust.
- [ ] Add capability/configuration checks for Windows x64 workstation mode.
- [ ] Inventory required, temporarily stubbed, and unsupported methods.

## Constraints

- A stub may log and fail fast; it may not claim success without satisfying its contract.
- No collector policy or duplicate heap state belongs in C++.
- Exceptions and panics must be contained at every boundary.

## Acceptance criteria

- CoreCLR accepts version negotiation and initialization and receives stable interface pointers.
- Unsupported server GC and architecture configurations fail with useful diagnostics.
- ABI tests cover representative calls in both directions.
- The interface inventory is tied to the pinned runtime header commit.

## Hints

Implement the smallest startup path revealed by tracing. Resist filling a large interface with silent zero-return stubs.

## Agent review focus

Ask the agent to compare every implemented signature with the pinned headers and identify any stub whose return value could let CoreCLR continue unsafely.
