
## Project

A Rust version of [Cloudstate](https://cloudstate.io/docs/index.html) client application.

### protocols

Contains [original protobuf files](https://github.com/cloudstateio/cloudstate/tree/master/protocols).

- [] How to generate compilable code? Maybe need to use PROTOC! macros for importing generated files?

### spike

A spike version of Cloudstate client application. Work in progress!


## :TODO: Add shopping_cart proto files to the project and generate Rust files with PROST! or tonic_build

How to compile proto files into the sources?

## :TODO: Implement dummy EventSourced service

[Bi-directional gRPC streaming with Tonic](https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md)
https://github.com/hyperium/tonic/blob/master/examples/src/authentication/server.rs#L56

[Async steams](https://github.com/tokio-rs/async-stream)

## :TODO: Bind EventSourced service to the ShopCart entity manually

## :TODO: Implement intermediate abstractions for binding an Entity to the server

## :TODO: Implement all missing parts

## :TODO: Test with Cloudstate TCK

## :TODO: Write documentation

