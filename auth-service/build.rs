fn main() {
    let mut config = prost_build::Config::new();
    config.message_attribute("*", "#[derive(Default)]");
    tonic_build::configure()
        .compile_with_config(
            config,
            &["proto/auth.proto"],
            &["proto", "proto/prost"],
        )
        .unwrap();
}
