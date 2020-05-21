

fn main() {

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/protocol")
        .compile(&[
            "protocol/cloudstate/entity.proto",
            "protocol/cloudstate/event_sourced.proto",
        ], &[
            "protocol",
        ])
        .expect("failed to compile protos");

    // This also generates some frontend stuff because the frontend proto folder has to be provided in includes.
    // Otherwise it won't be able to compile *.proto files.
    // TODO: doesn't seem to be possible to generate frontend separately
    // NOTE: it's not an issue because we don't need anything from the frontend module yet.
    tonic_build::configure()
        // Skip server / client generation for the example because it's implemented by Cloudstate sidecar.
        .build_server(false)
        .build_client(false)
        // TODO this approach doesn't work either because of
        //  error: macro attributes must be placed before `#[derive]`
        //  Consider submit a PR for tonic-build or to prost
        // .type_attribute("com.example.shoppingcart.AddLineItem", "#[proto_type_macro::proto_type]")
        .out_dir("src/prost_example/shoppingcart")
        .compile(&[
            "example/shoppingcart/persistence/domain.proto",
            "example/shoppingcart/shoppingcart.proto",
        ], &[
            "example",
            "frontend",
        ])
        .expect("failed to compile protos");
    //TODO implement a custom ServiceGenerator to generate service specific command types with access to package name for unmarshaling code generation.


    // Generate Rust code for the user service messages with rust `protobuf` instead of `prost`.
    protobuf_codegen_pure::Codegen::new()
        .out_dir("src/example")
        .inputs(&[
            "example/shoppingcart/persistence/domain.proto",
            "example/shoppingcart/shoppingcart.proto",
        ])
        .includes(&[
            "example",
            "frontend",
            ".", // needed for google any/descriptor/empty
        ]).run().expect("protoc");

}
