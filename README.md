
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

- [ ] Verify type name for incoming commands that are encoded as Any type

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
        There are two possible ways
        - [ ] Provide the type name as an attribute, so the derive macro can use it.
        - [ ] Enhance Prost to preserve package name as an attribute for the generated message types.
        - [ ] Implement custom code generator that will preserve a prototype package.
            That will only solve it for event-sourced command, but we need more general solution that will work for 
            events and all possible messages. It will allow to wrap any message into protobuf Any type and decode it back.
        - [x] Temporal simple solution can only match the message type name and ignore the package completely.
            E.g. instead of matching `type.googleapis.com/com.example.shoppingcart.AddLineItem` match only the last part `.AddLineItem`.
            
    - [ ] Implement correct type matching in `command_macro_derive`

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

- [ ] Bind EventSourced service to the ShopCart entity manually

- [ ] Implement intermediate abstractions for binding an Entity to the server

- [ ] Test with Cloudstate TCK

- [ ] Write documentation

- [ ] Implement Gatling stress test
