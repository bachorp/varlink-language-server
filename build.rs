use std::io::{Read, Seek, Write};

fn main() {
    let generated = auto_lsp_codegen::generate(
        tree_sitter_varlink::NODE_TYPES,
        &tree_sitter_varlink::LANGUAGE.into(),
        None,
    )
    .to_string();

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(std::path::PathBuf::from("./src/ast.rs"))
        .unwrap();

    let mut current = String::new();
    file.read_to_string(&mut current).unwrap();

    if current != generated {
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        file.set_len(0).unwrap();
        file.write_all(generated.as_bytes()).unwrap();
    }
}
