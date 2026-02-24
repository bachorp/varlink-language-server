use auto_lsp::{
    anyhow::{self, Ok},
    core::ast::AstNode,
    default::db::{BaseDatabase, tracked::get_ast},
    lsp_types::{Location, ReferenceParams},
};

use crate::{
    ast::{Typedef, Typeref},
    capabilities::util::{capture_at, get_file_from_db},
};

pub fn references(
    db: &impl BaseDatabase,
    params: ReferenceParams,
) -> anyhow::Result<Option<Vec<Location>>> {
    let file = get_file_from_db(&params.text_document_position.text_document.uri, db)?;
    let ast = get_ast(db, file);

    if let Some(typedef) = capture_at::<Typedef>(ast, params.text_document_position.position) {
        let document_bytes = file.document(db).as_bytes();
        let name = typedef.name.cast(ast).get_text(document_bytes).unwrap();
        let refs: Vec<Location> = ast
            .iter()
            .filter_map(|node| node.lower().downcast_ref::<Typeref>())
            .filter_map(|typeref| {
                if typeref.get_text(document_bytes).unwrap() == name {
                    Some(Location {
                        range: typeref.get_lsp_range(),
                        uri: params.text_document_position.text_document.uri.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(Some(refs))
    } else {
        Ok(None)
    }
}
