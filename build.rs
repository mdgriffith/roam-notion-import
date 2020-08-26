use std::path::PathBuf;
extern crate cc;

fn main() {
    // let dir: PathBuf = ["tree-sitter-markdown", "src"].iter().collect();

    // cc::Build::new()
    //     .include(&dir)
    //     .file(dir.join("parser.c"))
    //     .file(dir.join("scanner.c"))
    //     .compile("tree-sitter-markdown");

    let elm: PathBuf = ["tree-sitter-markdown", "src"].iter().collect();

    cc::Build::new()
        .include(&elm)
        .cpp(true)
        .file(elm.join("scanner.cc"))
        .file(elm.join("parser.c"))
        .compile("tree-sitter-markdown");
}
