# Roadmap

`rusty_json` is a from-scratch JSON library for Rust. This document is the
scope-of-record for what "parity" means at each stage — the `parity-loop`
skill audits against this file before generating any new gap list.

## Phase 1 — RFC 8259-compliant core (current)

A minimal, correct JSON implementation, independent of `serde_json`'s exact
API shape:

- `Value` enum (`Null`, `Bool`, `Number`, `String`, `Array`, `Object`)
- Parser: `&str`/bytes → `Value`, spec-correct per [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259)
  (literals, numbers, string escapes incl. `\uXXXX` and surrogate pairs,
  arrays, objects, whitespace), with position-aware errors
- Serializer: `Value` → compact string, plus a pretty-printed form
- `Value` accessors (`get`, indexing, `as_*`/`is_*`, `From` conversions)
- Design target: `no_std` + `alloc` where feasible, with a default-on `std`
  feature for ergonomics (`Display`/`Error` impls, convenience I/O) — mirrors
  `serde_json`'s own `alloc` feature split.

Assessed doc-driven (direct read of the RFC), since there's no existing
public surface yet to diff against.

## Phase 2 — `serde_json` API parity (future)

Once Phase 1 lands, re-assess by diffing `rusty_json`'s public API against a
pinned `serde_json` version (`cargo public-api`), symbol by symbol:

- `Serialize`/`Deserialize` traits and derive-macro integration
- Streaming `Serializer`/`Deserializer` (`to_writer`, `from_reader`)
- `RawValue`, arbitrary-precision numbers (`arbitrary_precision` feature)
- `json!` macro
- Any remaining `serde_json::Value` / `serde_json::Error` / `serde_json::Map`
  surface not already covered by Phase 1

Out of scope until Phase 1 is complete and re-assessed.

## Out of scope (this round)

- Anything not reachable from `no_std` + `alloc` unless explicitly gated
  behind the `std` feature.
- `serde` derive-macro integration (Phase 2).
