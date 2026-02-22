use auto_lsp::{
    anyhow,
    core::ast::AstNode,
    default::db::{
        BaseDatabase,
        tracked::{ParsedAst, get_ast},
    },
    lsp_types::{SelectionRange, SelectionRangeParams},
};

use crate::capabilities::util::{get_file_from_db, leaf_at};

pub fn selection_range(
    db: &impl BaseDatabase,
    params: SelectionRangeParams,
) -> anyhow::Result<Option<Vec<SelectionRange>>> {
    let file = get_file_from_db(&params.text_document.uri, db)?;
    let ast = get_ast(db, file);

    let results = params
        .positions
        .iter()
        .map(|&pos| {
            let leaf = leaf_at(&ast.nodes, pos.line, pos.character).unwrap();
            SelectionRange {
                range: leaf.get_lsp_range(),
                parent: mk_parent(leaf, ast),
            }
        })
        .collect();

    Ok(Some(results))
}

fn mk_parent(node: &Box<dyn AstNode>, ast: &ParsedAst) -> Option<Box<SelectionRange>> {
    node.get_parent(ast).map(|p| {
        Box::new(SelectionRange {
            range: p.get_lsp_range(),
            parent: mk_parent(p, ast),
        })
    })
}
