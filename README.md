
## Project

A Rust version of [Cloudstate](https://cloudstate.io/docs/index.html) client application.

Run server:
```
cargo run --example shopping_cart
```

Run client:
```
cargo run --bin spike-client
```


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

### entity_spike

Entity implementation.

- [x] Implement end user entity code to see how it may look like.
- [x] Add shopping_cart proto files to the project and generate Rust files.
    - [x] Generate proto classes out to the crates to navigate to in the IDE.    
- [ ] There is a separate proto message for each type of command. How to handle it nicely?


### entity_server

A spike version of Cloudstate client application.


- [x] Generate Cloudstate protocol Rust code out of Protobuf definitions.

- [x] Implement dummy EventSourced service
    [Bi-directional gRPC streaming with Tonic](https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md)
    https://github.com/hyperium/tonic/blob/master/examples/src/authentication/server.rs#L56
    
- [x] Implement some EventSourced service interaction and find a way to test it
    [Bi-directional gRPC streaming with Tonic](https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md#server-state)
    [Async steams](https://github.com/tokio-rs/async-stream)
    
- [ ] Implement test that runs the server and test it with a client

- [ ] Bind EventSourced service to the ShopCart entity manually

- [ ] Implement intermediate abstractions for binding an Entity to the server

- [ ] Test with Cloudstate TCK

- [ ] Write documentation

- [ ] Implement Gatling stress test
