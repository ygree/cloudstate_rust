

fn main() {

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        // .out_dir("src") // Uncomment if need to take a look at the sources. Use default output to be able use include_proto! macro.
        // NOTE: when out_dir is specified then `tonic::include_proto!` won't be able to locate generated file
        .compile(&[
            "protocol/cloudstate/event_sourced.proto",
        ], &[
            "protocol",
        ])
        .expect("failed to compile protos");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        // .out_dir("src") // Uncomment if need to take a look at the sources. Use default output to be able use include_proto! macro.
        // NOTE: when out_dir is specified then `tonic::include_proto!` won't be able to locate generated file
        .compile(&[
            "example/shoppingcart/persistence/domain.proto",
        ], &[
            "example",
        ])
        .expect("failed to compile protos");
}
