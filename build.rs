use tonic_build;

fn main() {
    let proto_root = "proto";
    println!("cargo:rerun-if-changed={}", proto_root);

    tonic_build::configure()
        .out_dir("src/route")
        .compile(&["proto/route_guide.proto"], &[proto_root])
        .expect("Failed to compile route_guide.proto");
}
