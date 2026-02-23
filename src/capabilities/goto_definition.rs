use auto_lsp::{
    anyhow::{self, Ok},
    core::ast::AstNode,
    default::db::{BaseDatabase, tracked::get_ast},
    lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location},
};

use crate::{
    ast::{Typedef, Typeref},
    capabilities::util::{get_file_from_db, leaf_at},
};

pub fn goto_definition(
    db: &impl BaseDatabase,
    params: GotoDefinitionParams,
) -> anyhow::Result<Option<GotoDefinitionResponse>> {
    let file = get_file_from_db(&params.text_document_position_params.text_document.uri, db)?;
    let ast = get_ast(db, file);

    if let Some(typeref) =
        leaf_at(ast, params.text_document_position_params.position).and_then(|n| {
            // TODO: It would be nice to have a utility to find a node of specified type in the local branch
            n.get_parent(ast)
                .and_then(|n| n.lower().downcast_ref::<Typeref>())
        })
    {
        let document_bytes = file.document(db).as_bytes();
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
    } else {
        Ok(None)
    }
}
