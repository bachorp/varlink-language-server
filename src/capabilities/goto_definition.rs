use auto_lsp::{
    anyhow::{self, Ok},
    core::{ast::AstNode},
    default::db::{BaseDatabase, tracked::get_ast},
    lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location},
};

use crate::{
    ast::{Typedef, Typeref},
    capabilities::util::{capture_at, get_file_from_db},
};

pub fn goto_definition(
    db: &impl BaseDatabase,
    params: GotoDefinitionParams,
) -> anyhow::Result<Option<GotoDefinitionResponse>> {
    let file = get_file_from_db(&params.text_document_position_params.text_document.uri, db)?;
    let ast = get_ast(db, file);

    if let Some(typeref) = capture_at::<Typeref>(ast, params.text_document_position_params.position)
    {
        let document_bytes = file.document(db).as_bytes();
        let name = typeref.get_text(document_bytes).unwrap();
        let defs: Vec<Location> = ast
            .iter()
            .filter_map(|node| node.lower().downcast_ref::<Typedef>())
            .filter_map(|typedef| {
                let def = typedef.name.cast(ast);
                if def.get_text(document_bytes).unwrap() == name {
                    Some(Location {
                        range: def.get_lsp_range(),
                        uri: params
                            .text_document_position_params
                            .text_document
                            .uri
                            .clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(Some(GotoDefinitionResponse::Array(defs)))
    } else {
        Ok(None)
    }
}
