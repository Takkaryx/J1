fn main() {
    let mut gen = micropb_gen::Generator::new();
    // Compile example.proto into a Rust module
    gen.add_protoc_arg("-I".to_owned() + &std::env::var("PROTO_DIR").unwrap())
        .compile_protos(
            &["accel.proto"],
            std::env::var("OUT_DIR").unwrap() + "/j1_proto.rs",
        )
        .unwrap();
}
