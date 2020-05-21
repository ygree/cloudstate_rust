
## Project

A Rust version of [Cloudstate](https://cloudstate.io/docs/index.html) client application.

Run server:
```
cargo run --example shopping_cart
```

Run client:
```
cargo run --example client
```

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

### entity_spike

Entity implementation.

- [x] Implement end user entity code to see how it may look like.
- [x] Add shopping_cart proto files to the project and generate Rust files.
    - [x] Generate proto classes out to the crates to navigate to in the IDE.

- [x] Verify type name for incoming commands that are encoded as Any type

    - [x] Had an idea to try until succeed but it won't work because deserialization may work for a wrong type, e.g. RemoveLine instead of AddLine
        That's because the RemoteLine has a subset of AddLine's fields.
        So, this approach is not valid because it could change the meaning of the command completely.
            ```
            impl CommandDecoder for ShoppingCartCommand {
                fn decode(type_url: String, bytes: Bytes) -> Option<Self> {
                    let mut bytes_mut = bytes;
                    //
                    let result = <AddLineItem as Message>::decode(&mut bytes_mut).map(|v| Some(ShoppingCartCommand::AddLine(v)));
                    // should try the next if this one failed and so forth
                    let result = <RemoveLineItem as Message>::decode(&mut bytes_mut).map(|v| Some(ShoppingCartCommand::RemoveLine(v)));
                    let result = <GetShoppingCart as Message>::decode(&mut bytes_mut).map(|v| Some(ShoppingCartCommand::GetCart(v)));
                    let result = result.unwrap_or_else(|_| None);
                    //
                    result
                }
            }
            ```

    - [ ] Is it possible to get the full type_name out of the generated protobuf messages?
        - [-] Temporal simple solution can only match the message type name and ignore the package completely.
            E.g. instead of matching `type.googleapis.com/com.example.shoppingcart.AddLineItem` match only the last part `.AddLineItem`.
            It doesn't solve an issue at all, because there may be overlapping names, e.g. `AddLineItem` and `CreateAddLineItem`
            Also it doesn't solve encoding problem. As we need to send an event back to the frontend we will need to encode it's type in Any.
        - [ ] Enhance Prost to preserve package name as an attribute for the generated message types.
            It can be done by adding an extra attribute to a generated types, e.g. `prost(package = <...>)`
            This logic should be placed in `prost-build/src/code_generator.rs/append_message`.
        - [x] Provide the type name as an attribute, so the derive macro can use it.
            That's currently the only possible solution that can be replaced in the future if needed and it can also coexist with other solutions.
            
    - [x] Implement correct type matching in `command_macro_derive`

     * final val DefaultTypeUrlPrefix = "type.googleapis.com"

     * Get the type's fully-qualified name, within the proto language's namespace. This differs from
     * the Java name. For example, given this {@code .proto}:
     *
     * <pre>
     *   package foo.bar;
     *   option java_package = "com.example.protos"
     *   message Baz {}
     * </pre>
     *
     * {@code Baz}'s full name is "foo.bar.Baz".    
     
     
    
- [x] There is a separate proto message for each type of command. How to handle it nicely?
    Group it into one type, e.g. enum.


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

- [x] Implement intermediate abstractions for binding an Entity to the server
    Bare minimum, to be able to forward requests to the entities.

- [x] Test with Cloudstate TCK
    cloudstate:rust-tck-test
    tck/it:test

- [x] Implement stub version of the `entity_discovery_server` service. TCK Relies on discovery service implementation.

- [ ] Discovery server implementation requires generation protobuf descriptors
    
- [ ] Preserve service protobuf and return on the discovery call to solve:
    ---> EntityDiscovery.report_error: error = UserFunctionError { message: "Service [com.example.shoppingcart.ShoppingCart] not found in descriptors!" }
    Unfortunately, Prost! doesn't support protobuf descriptors at all! 
    Also, it doesn't support protobuf Any type serialization and there seem to be no way to extract package name that is needed for Any type (de)serialization.
    
    Try to use `rust-protobuf` for the user service protobuf messages and descriptors generation and for the gRPC implementation use `prost`?

- [x] Try [rust-protobuf](https://github.com/stepancheg/rust-protobuf)
    Find out how to generate a gRPC server implementation or if it's possible to use PROTOC! for Cloudstate parts and Rust-Protobuf for user defined services and messages.

    Generate Rust code for the user service messages with rust `protobuf` instead of `prost`.
    
    - [-] How to remove inner attributes out of generated files? They are not compiled with the current version of Rust.
        No, need to remove it. It was complaining because of include! macro, once import generated files as simple modules it all works.
    
    - [x] Need to adapt the current CommandDecoder implementation to use `protobuf`. Currently it relies on  prost::message::Message
        Done. Tests are failing because error don't exactly match. Need to find a better way to make it less fragile.
        
    Successfully migrated to the protobuf for the main code, and the shopping_cart.rs example. The client.rs examples is still using prost!.
    
    - [ ] Leverage Any protobuf type support
    - [ ] Leverage FileDescriptors to implement Discovery server

- [ ] command-macro-derive tests are fragile because they match exact compilation error and it depends on the `server-spike` module that makes it very fragile. The only need for the dependency is the CommandDecoder trait. Moving it to more stable crate should resolve this issue.
    
        
- [ ] Integrate into Cloudstate TCK        

- [ ] Write documentation

- [ ] Implement Gatling stress test
