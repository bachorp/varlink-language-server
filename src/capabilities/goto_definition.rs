use auto_lsp::{
    anyhow, core::ast::AstNode, default::db::{BaseDatabase, tracked::get_ast}, lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location}
};

use crate::{
    ast::{Typedef, Typeref},
    capabilities::util::{get_file_from_db, subtree_at},
};

pub fn goto_definition(
    db: &impl BaseDatabase,
    params: GotoDefinitionParams,
) -> anyhow::Result<Option<GotoDefinitionResponse>> {
    let file = get_file_from_db(&params.text_document_position_params.text_document.uri, db)?;
    let ast = get_ast(db, file);
    let position = params.text_document_position_params.position;
    let refs = ast.nodes[subtree_at(ast, position.line, position.character)]
        .iter()
        .filter_map(|n| {
            let auto_lsp::lsp_types::Range { start, end } = n.get_lsp_range();
            if (start.line, start.character) <= (position.line, position.character)
                && (position.line, position.character) <= (end.line, end.character)
            {
                n.lower().downcast_ref::<Typeref>()
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let document_bytes = file.document(db).as_bytes();
    assert!(refs.len() < 2);
    match refs.first() {
        None => Ok(None),
        Some(typeref) => {
            let mut defs = Vec::new();
            let name = typeref.get_text(document_bytes).unwrap();
            ast.iter().for_each(|node| {
                if let Some(typedef) = node.lower().downcast_ref::<Typedef>() {
                    let def = typedef.name.cast(ast);
                    if def.get_text(document_bytes).unwrap() == name {
                        defs.push(Location {
                            range: def.get_lsp_range(),
                            uri: params
                                .text_document_position_params
                                .text_document
                                .uri
                                .clone(),
                        });
                    }
                }
            });
            Ok(Some(GotoDefinitionResponse::Array(defs)))
        }
    }
}
