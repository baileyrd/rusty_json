# rusty_json

A from-scratch JSON library for Rust. `no_std` + `alloc` by default, with a
default-on `std` feature for ergonomics.

```rust
let value = rusty_json::from_str(r#"{"a": [1, 2, true]}"#).unwrap();
assert_eq!(value["a"][1].as_i64(), Some(2));
assert_eq!(rusty_json::to_string(&value), r#"{"a":[1,2,true]}"#);
```

See [`ROADMAP.md`](ROADMAP.md) for the phased parity plan (currently Phase 1:
an RFC 8259-compliant core), and [`gap-analysis.md`](gap-analysis.md) for the
tracked gap list this phase was built from.