use std::path::PathBuf;

fn main() {
    // Use the generated Tree-sitter parser
    let src_dir = PathBuf::from("src");

    cc::Build::new()
        .include(&src_dir)
        .file(src_dir.join("parser.c"))
        .file(src_dir.join("scanner.c")) // Include external scanner
        .compile("tree-sitter-nix");
}
