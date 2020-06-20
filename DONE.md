DONE
====

## entity_spike

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
            
    - [x] Implement correct type matching in `cloudstate_prost_derive`

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

## entity_server

A spike version of Cloudstate client application.

### :DONE: Generate Cloudstate protocol Rust code out of Protobuf definitions.

### :DONE: Implement dummy EventSourced service
    
[Bi-directional gRPC streaming with Tonic](https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md)
https://github.com/hyperium/tonic/blob/master/examples/src/authentication/server.rs#L56
    
    
### :DONE: Implement some EventSourced service interaction and find a way to test it

[Bi-directional gRPC streaming with Tonic](https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md#server-state)
[Async steams](https://github.com/tokio-rs/async-stream)
  
    
### :TODO: Implement a test running the server and test it with a client


### :DONE: Implement intermediate abstractions for binding an Entity to the server

Bare minimum, to be able to forward requests to the entities.

### :DONE: Test with Cloudstate TCK

```
cloudstate:rust-tck-test
tck/it:test
```

### :DONE: Implement stub version of the `entity_discovery_server` service. TCK Relies on discovery service implementation.

### :DONE: Discovery server implementation requires generation protobuf descriptors

Implement in `generate-desc`.
    
### :TODO: Preserve service protobuf and return on the discovery call to solve:
---> EntityDiscovery.report_error: error = UserFunctionError { message: "Service [com.example.shoppingcart.ShoppingCart] not found in descriptors!" }
Unfortunately, Prost! doesn't support protobuf descriptors at all! 
Also, it doesn't support protobuf Any type serialization and there seem to be no way to extract package name that is needed for Any type (de)serialization.

Try to use `rust-protobuf` for the user service protobuf messages and descriptors generation and for the gRPC implementation use `prost`?

### :DONE: Try [rust-protobuf](https://github.com/stepancheg/rust-protobuf)
Find out how to generate a gRPC server implementation or if it's possible to use PROTOC! for Cloudstate parts and Rust-Protobuf for user defined services and messages.

Generate Rust code for the user service messages with rust `protobuf` instead of `prost`.

### :DONE: How to remove inner attributes out of generated files? They are not compiled with the current version of Rust.

No, need to remove it. It was complaining because of include! macro, once import generated files as simple modules it all works.

#### :DONE: Need to adapt the current CommandDecoder implementation to use `protobuf`. Currently, it relies on  prost::message::Message

Done. Tests are failing because error don't exactly match. Need to find a better way to make it less fragile.

Successfully migrated to the protobuf for the main code, and the shopping_cart.rs example. The client.rs examples is still using prost!.
    
### :DONE: implement a file descriptor required for the incoming discovery call to send back to proxy.

#### :DONE: Leverage FileDescriptors to implement Discovery server
    
Generated file descriptor (example/shoppingcart.rs:1089) doesn't seem to contain any service descriptor!

//Try to generate file descriptor properly. Should be possible to do with protoc or protobuf_codegen_pure
// https://github.com/stepancheg/rust-protobuf/issues/292#issuecomment-392607319

trying to 

In Prost! there is a relatively fresh PR for adding file descriptor support: https://github.com/danburkert/prost/pull/311.
Unfortunately, the PR was closed with the proposal to use include_byte! to import generated FileDescriptor.
Well, there is new superseded PR has been open: https://github.com/danburkert/prost/pull/326.

Well. Looks like in `progobuf-codegen-pure-2.14.0` it skips parsing services in `parser.rs`:
    if let Some(_service) = self.next_service_opt()? {
        continue;
    }
but in `master` it's implemented: https://github.com/stepancheg/rust-protobuf/blob/9498605ac57708a022ad8398286d8a86e7146ca9/protobuf-codegen-pure/src/parser.rs#L1091

There should be a way to use master branch as a dependency.
protobuf = { git = "https://github.com/stepancheg/rust-protobuf" }
protobuf-codegen-pure = { git = "https://github.com/stepancheg/rust-protobuf" }
But it didn't work either. The generated code isn't different from the 2.14.0 and `file_descriptor_proto_data` didn't contain service descriptors.

Try to use `protoc-rust = { version = "2" }`
Woot! It generates larger `file_descriptor_proto_data` that probably contains the service descriptor.
Let's try it with TCK.

Woot! The TCK can see the `ShoppingCart` service now. 
---> EntityDiscovery.discover : request.message = ProxyInfo { protocol_major_version: 0, protocol_minor_version: 1, proxy_name: "cloudstate-proxy-core", proxy_version: "0.4.1-200-254ed387", supported_entity_types: ["cloudstate.crdt.Crdt", "cloudstate.function.StatelessFunction", "cloudstate.eventsourced.EventSourced"] }
---> service : "ShoppingCart" 

But it fails because of incomplete file descriptor.
2020-05-22 00:01:31.285 ERROR akka.actor.OneForOneStrategy - Descriptor dependency [google/protobuf/empty.proto] not found, dependency path: [shoppingcart/shoppingcart.proto]

Try to add empty's descriptor as well.
Okay, now it wants yet another descriptor:  
Descriptor dependency [cloudstate/entity_key.proto] not found, dependency path: [shoppingcart/shoppingcart.proto]

#### :DONE: Maybe it's easier to use `protoc` to generate one file and use `include_bytes!` then. How?

protoc --proto_path=./ \
    --proto_path=protocol \
    --descriptor_set_out=user-function.desc \
    example/shoppingcart/shoppingcart.proto
        
https://github.com/danburkert/prost/pull/326/files#diff-eb205fd0a0569ec3478a1f78f1df4ec3R537-R541
        
```rust
let mut cmd = Command::new(protoc());
cmd.arg("--include_imports")
    .arg("--include_source_info")
    .arg("-o")
    .arg(&descriptor_set);
```

Okay. This seems to work with the fix in `annotations.proto` to point to `google/protobuf/descriptor.proto` instead of `google/proto/descriptor.proto`.
Actually, no fix needed. After I replaced proto files with ones from the cloudstate repo.

See `protocols/generate_desc`

### :DONE: extract all parts that don't depend on the gRPC implementation out to `cloudstate-core` module

### :FIXED: Next TCK error

init service: com.example.shoppingcart.ShoppingCart entity_id: testuser:1
Can't handle a command until the entity is initialized!
Unexpected reply, had no current command: EventSourcedReply(1,None,Vector(),Vector(),None,UnknownFieldSet(Map()))

### :DONE: Return proper response for the command

Introduce a new associated type: Response.

- [x] Implement encoder for the command response.

### :DONE: provide file descriptor proto with the service description.

### :DONE: use Vec<u8> in `CommandDecoder` implementation instead of Bytes

### :DONE: Pass all the Cloudstate TCK tests

### :DONE: Use protobuf::well_known_types::Empty for the empty result.

### :FIXED: `shopping_cart_protobuf.rs` doesn't pass TCK test. It returns an empty cart for some reason.

### :DONE: Fix Empty type_url inconsistency

2020-05-28 23:07:06.766 WARN io.cloudstate.proxy.Serve$ - com.example.shoppingcart.ShoppingCart.RemoveItem: Expected reply type_url to be [type.googleapis.com/google.protobuf.Empty] but was [type.googleapis.com/com.example.shoppingcart.Empty].

> Introduce and EmptyReply. From the user impl perspective None assumed to be an empty response.

### :DONE: TCK test throws an exception

[ERROR] [05/28/2020 23:07:08.330] [CloudStateTCK-akka.actor.default-dispatcher-26] [akka://CloudStateTCK/system/pool-master] connection pool for Pool(shared->http://127.0.0.1:9000) has shut down unexpectedly
java.lang.IllegalStateException: Pool shutdown unexpectedly

> Actually, it throws the same error for the Java sample TCK test.

### :DONE: Clean up server-spike from examples
    Consider moving shopping_cart_protobuf.rs into an alternative binary crate into shopcart-example

### :DONE: rename `CommandDecoder` to something more descriptive

### :DONE: does `protobuf` provides a better way to get package name?
    Doesn't seem so. The only place it has the package name encoded is the file descriptor.
