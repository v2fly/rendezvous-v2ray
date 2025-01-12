use std::io;

fn main() -> io::Result<()> {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .build_transport(false)
        .compile_protos(&[
            "./vendor2/v2ray-core/app/observatory/command/command.proto",
            "./vendor2/v2ray-core/app/subscription/subscriptionmanager/command/command.proto",
            "./vendor2/v2ray-core/app/router/command/command.proto"
        ],
                        &["./vendor2/protoc/include", "./vendor2/v2ray-core"])
        .expect("Failed to compile proto");
    Ok(())
}