fn main() {
    std::fs::write(
        std::path::PathBuf::from("./src/ast.rs"),
        auto_lsp_codegen::generate(
            tree_sitter_varlink::NODE_TYPES,
            &tree_sitter_varlink::LANGUAGE.into(),
            None,
        )
        .to_string(),
    )
    .unwrap();
}
