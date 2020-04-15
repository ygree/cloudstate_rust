

fn main() {

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src")
        // .out_dir("src")
        .compile(&[
            "protocol/cloudstate/event_sourced.proto",

        ], &[
            "protocol",
        ])
        .expect("failed to compile protos");
}
