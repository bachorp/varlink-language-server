use auto_lsp::{
    anyhow,
    core::{ast::AstNode, document::Document},
    default::db::{
        BaseDatabase,
        tracked::{ParsedAst, get_ast},
    },
    lsp_types::{SelectionRange, SelectionRangeParams},
};

use crate::util::{get_file_from_db, most_specific_at};

pub fn selection_range(
    db: &impl BaseDatabase,
    params: SelectionRangeParams,
) -> anyhow::Result<Option<Vec<SelectionRange>>> {
    let file = get_file_from_db(&params.text_document.uri, db)?;
    let ast = get_ast(db, file);
    let document = file.document(db);

    Ok(Some(
        params
            .positions
            .iter()
            .filter_map(|&pos| {
                most_specific_at(ast, document, pos).map(|node| mk_range(node, ast, document))
            })
            .collect(),
    ))
}

fn mk_range(node: &Box<dyn AstNode>, ast: &ParsedAst, document: &Document) -> SelectionRange {
    SelectionRange {
        range: node.get_lsp_range(document).unwrap(),
        parent: node.get_parent(ast).map(|p| Box::new(mk_range(p, ast, document))),
    }
}
