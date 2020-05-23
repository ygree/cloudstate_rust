
## Project

A Rust version of [Cloudstate](https://cloudstate.io/docs/index.html) client application.

Run server: `shoppingcart-server`
Run client: `shoppingcart-client`

Generate file descriptor proto for the shopping card example with: `protocols/generate-desc`
// TODO it's a temporal solution and in generatl should be done in `build.rs`

### Running TCK

Use `github.com/cloudstateio/cloudstate`

Set rust's frontend implementation in: `tck/src/it/resources/application.conf`

Run tests with: `sbt tck/it:test`


### gRPC client

#### Bloom RPC

https://github.com/uw-labs/bloomrpc

**NOTE**: Set Import Paths to `eventsourced-spike/protocols/protocol` before importing `event_sourced.proto`. Otherwise, it will fail with:
```
Error while importing protos
no such Type or Enum 'ClientAction' in Type .cloudstate.eventsourced.EventSourcedReply
```


### protocols

Contains [original protobuf files](https://github.com/cloudstateio/cloudstate/tree/master/protocols).


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

