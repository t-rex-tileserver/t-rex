use protoc_rust::Customize;

fn main() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/mvt",
        input: &["src/mvt/vector_tile.proto"],
        includes: &["src/mvt"],
        customize: Customize {
            ..Default::default()
        },
    })
    .expect("protoc");
}
