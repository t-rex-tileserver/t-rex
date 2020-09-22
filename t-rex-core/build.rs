use protoc_rust::Codegen;

fn main() {
    Codegen::new()
        .out_dir("src/mvt")
        .inputs(&["src/mvt/vector_tile.proto"])
        .include("src/mvt")
        .run()
        .expect("Running protoc failed.");
}
