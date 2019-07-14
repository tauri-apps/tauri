extern crate includedir_codegen;

use includedir_codegen::Compression;

fn main() {
    includedir_codegen::start("ASSETS")
        .dir("./target/compiled-web", Compression::Gzip)
        .build("data.rs")
        .unwrap();
}
