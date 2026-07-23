# gap-analysis.md — Phase 1 (RFC 8259 core)

Assessed doc-driven, reading [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259)
directly — there's no existing public surface in this crate yet to diff
against (source: `spec` for every row). Scope and staging per `ROADMAP.md`.
Nothing here is breaking since the crate starts empty; sizes assume the
listed dependency order (each row builds on the ones above it).

| Symbol | Category | Source | Platforms | Reference | Breaking? | Est. size | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `Value` | type | spec | core | RFC 8259 §3 | no | S | 6-variant enum: `Null`, `Bool`, `Number`, `String`, `Array`, `Object` |
| `Number` | type | spec | core | RFC 8259 §6 | no | S | Holds i64/u64/f64; needs to represent both integers and floats without losing exactness where possible |
| `Error` | type | spec | core | RFC 8259 (error reporting is implementation-defined) | no | S | Position-aware (line/col or byte offset) parse error type |
| Parser: literals + whitespace | fn | spec | core | RFC 8259 §2, §3 | no | S | `null`/`true`/`false`, whitespace skipping, structural chars |
| Parser: numbers | fn | spec | core | RFC 8259 §6 | no | M | Int/float/exponent, leading-zero rejection, negative, negative zero |
| Parser: strings | fn | spec | core | RFC 8259 §7 | no | M | Escapes, `\uXXXX`, UTF-16 surrogate pairs, control-char rejection, UTF-8 validation |
| Parser: arrays | fn | spec | core | RFC 8259 §5 | no | S | Depends on literals/numbers/strings parsing |
| Parser: objects | fn | spec | core | RFC 8259 §4 | no | S | Depends on strings (keys) + values |
| Serializer: compact | fn | spec | core | RFC 8259 (well-formed output) | no | M | `Value` → compact `String`/writer, correct string escaping on the way out |
| Serializer: pretty | fn | spec | core | n/a (ergonomics, not spec-mandated) | no | S | Indented form of the compact serializer |
| `Value` accessors | fn | spec | core | mirrors `serde_json::Value` conventions | no | M | `get`, `Index` for `&str`/`usize`, `as_*`/`is_*` predicates |
| `Value` `From` conversions | fn | spec | core | mirrors `serde_json::Value` conventions | no | S | `From<bool>`, integers, floats, `String`/`&str`, `Vec<Value>`, object map |
| `Display`/`std::error::Error` for `Error` | fn | spec | std | n/a (std ergonomics) | no | S | Gated behind the `std` feature |

**Est. size legend:** S = small (one focused PR, <~150 lines incl. tests), M = medium (a couple hundred lines incl. tests, may need a few sub-commits).
