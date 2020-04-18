

fn main() {

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        // .out_dir("src") // Uncomment if need to take a look at the sources. Use default output to be able use include_proto! macro.
        .compile(&[
            "protocol/cloudstate/event_sourced.proto",
        ], &[
            "protocol",
        ])
        .expect("failed to compile protos");

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src") // Uncomment if need to take a look at the sources. Use default output to be able use include_proto! macro.
        .compile(&[
            "example/shoppingcart/persistence/domain.proto",
        ], &[
            "example",
        ])
        .expect("failed to compile protos");
}
