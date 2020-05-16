
Debug macro expansion
=====================

Couldn't find a way to instruct `cargo expand` to run against a single integration test.

The work-around is to declare a test as a binary in `Cargo.toml`, e.g.

```
[[bin]]
name = "shopping_cart_test"
path = "tests/shopping_cart_test.rs"
```

And then run `cargo expand`:

```
cargo expand --bin shopping_cart_test > tests/shopping_cart_test-expanded.rs
```

It will produce `tests/shopping_cart_test-expanded.rs`.

For some reason, it complains about the prost attribute:
 
```
   |
21 |     #[prost(string, tag = "1")]
   |       ^^^^^
error: cannot find attribute `prost` in this scope
  --> command-macro-derive/tests/shopping_cart.rs:23:7
```
 
That's because after expansion the `derive` macro declaration was removed but its `prost` attributes weren't removed.

