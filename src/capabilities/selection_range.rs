use auto_lsp::{
    anyhow,
    core::ast::AstNode,
    default::db::{
        BaseDatabase,
        tracked::{ParsedAst, get_ast},
    },
    lsp_types::{SelectionRange, SelectionRangeParams},
};

use crate::capabilities::util::{get_file_from_db, most_specific_at};

pub fn selection_range(
    db: &impl BaseDatabase,
    params: SelectionRangeParams,
) -> anyhow::Result<Option<Vec<SelectionRange>>> {
    let file = get_file_from_db(&params.text_document.uri, db)?;
    let ast = get_ast(db, file);

    Ok(Some(
        params
            .positions
            .iter()
            .filter_map(|&pos| most_specific_at(&ast.nodes, pos).map(|node| mk_range(node, ast)))
            .collect(),
    ))
}

fn mk_range(node: &Box<dyn AstNode>, ast: &ParsedAst) -> SelectionRange {
    SelectionRange {
        range: node.get_lsp_range(),
        parent: node.get_parent(ast).map(|p| Box::new(mk_range(p, ast))),
    }
}
