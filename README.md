
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

