# gap-analysis.md — Phase 2, round 1 (serde_json API parity)

Assessed via `cargo public-api` (source: `diff`), diffing this crate's current
public surface against `serde_json` **1.0.151** (pinned for this run, default
features) by symbol name, per `ROADMAP.md`. Raw diff output filtered for
tooling noise: `PartialEq::eq`/`Iterator::Item`/`Serializer::Ok`/`FromStr::Err`
associated-type/method artifacts collapsed into the one real row they
represent (or dropped where there was no real gap behind them).

Two items below are **not** auto-implementable under this loop's rules and
need a decision before any issue in this batch touches them — see "Stop-and-ask
items" at the bottom.

## Round 1: pure additions (no new dependency, no breaking change)

| Symbol | Category | Source | Platforms | Reference | Breaking? | Est. size | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `Value::is_i64`/`is_u64`/`is_f64` | fn (existing type) | diff | core | `serde_json::Value` | no | S | Delegate to existing `Number::is_*` |
| `Value::as_null` | fn (existing type) | diff | core | `serde_json::Value::as_null` | no | S | Returns `Option<()>` |
| `Value::get_mut`/`as_array_mut`/`as_object_mut` | fn (existing type) | diff | core | `serde_json::Value` | no | M | Mutable counterparts to existing read-only accessors |
| `IndexMut<&str>`/`IndexMut<usize>` for `Value` | fn (existing type) | diff | core | `serde_json::Value` | no | M | Matches serde_json's auto-vivify: missing object key inserts `Null`; indexing a `Null` value promotes it to an empty `Object` first. Panics on index-out-of-bounds for arrays and on non-object/array+non-null, same as serde_json |
| `Value::take` | fn (existing type) | diff | core | `serde_json::Value::take` | no | S | `mem::take`-style: replace with `Null`, return old value |
| `Number::as_i128`/`as_u128`/`from_i128`/`from_u128` | fn (existing type) | diff | core | `serde_json::Number` | no | S | 128-bit variants alongside existing 64-bit ones |
| `PartialEq<T> for Value` (and reverse) | fn (existing type) | diff | core | `serde_json::Value`'s primitive `PartialEq` impls | no | M | `bool`/`i64`/`u64`/`f64`/`&str`/`String`, both directions — lets `value == "foo"` work directly |
| `pub type Result<T>` | type | diff | core | `serde_json::Result` | no | S | `Result<T> = core::result::Result<T, Error>` convenience alias |
| `Error::Category` + `classify`/`is_syntax`/`is_io`/`is_eof`/`is_data` | type + fn | diff | core | `serde_json::error::Category`, `serde_json::Error` | no | M | `Io`/`Syntax`/`Data`/`Eof` classification; our parser only ever produces `Syntax` today, `Eof` distinguishable from "unexpected end of input" cases |
| `Value::pointer`/`pointer_mut` | fn (existing type) | diff | core | `serde_json::Value::pointer`, [RFC 6901](https://www.rfc-editor.org/rfc/rfc6901) | no | M | JSON Pointer lookup |
| `FromIterator<(String, Value)>`/`FromIterator<Value>` for `Value` | fn | diff | core | `serde_json::Value`'s `FromIterator` impls | no | S | Object- and array-from-iterator construction |
| `impl FromStr for Value` | fn | diff | core | `serde_json::Value` (`str::parse`) | no | S | Thin wrapper over existing `from_str` free function |
| `json!` macro | macro | diff | core | `serde_json::json!` | no | M | Declarative macro building `Value` trees from Rust literal syntax; doesn't need serde, builds on existing `From` impls |

## Deferred to a later round (not in this batch)

| Symbol | Category | Source | Platforms | Reference | Breaking? | Est. size | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `RawValue` | type | roadmap | std | `serde_json::value::RawValue` | no | L | Behind serde_json's non-default `raw_value` feature; not in this round's diff (default-features only). Roadmap item, deferred |
| Arbitrary-precision numbers | fn (existing type) | roadmap | std | serde_json's `arbitrary_precision` feature | maybe | L | Would change `Number`'s internal representation; needs its own assessment pass once round 1 lands |
| `Value::sort_all_objects` | fn (existing type) | diff | core | `serde_json::Value::sort_all_objects` | no | — | serde_json's `preserve_order` interop no-op/normalizer; not meaningful until `Map` has an ordering choice (see stop-and-ask below) |

## Stop-and-ask items (not filed as issues yet)

These showed up in the diff but fail this loop's "pure addition only" bar —
flagging instead of auto-implementing, per parity-loop's rules.

1. **serde `Serialize`/`Deserialize` integration** (`Serializer`, `Deserializer`,
   `to_writer`/`to_vec`/`from_reader`/`from_slice` generic over `T: Serialize`/
   `Deserialize`, `Formatter`/`CompactFormatter`/`PrettyFormatter`,
   `StreamDeserializer`, `Error` implementing `serde::de::Error`/
   `serde::ser::Error`) — **requires adding `serde` as a new third-party
   dependency**, which this loop treats the same as a breaking change:
   stop and ask, don't add silently. This is the headline Phase 2 item from
   `ROADMAP.md`, sized far larger than "one function" — if approved, it needs
   splitting into its own multi-issue batch, not folded into round 1.
2. **`Map` becoming a real newtype with `.entry()`/iterator views** (`Entry`,
   `OccupiedEntry`, `VacantEntry`, `Keys`, `Values`, `ValuesMut`, `Iter`,
   `IterMut`, `IntoIter`, `IntoValues`) instead of today's
   `pub type Map = BTreeMap<String, Value>` alias — **breaking**: changes an
   existing public type's shape, not a pure addition. Also the natural place
   to reconsider insertion-order-preserving storage (serde_json's own `Map`
   uses `IndexMap` under a feature flag) — bundling that decision in is
   simpler than a second breaking change later.
