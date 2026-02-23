use auto_lsp::{
    anyhow,
    core::ast::AstNode,
    default::db::{BaseDatabase, tracked::get_ast},
    lsp_types::{
        PrepareRenameResponse, RenameParams, TextDocumentPositionParams, TextEdit, WorkspaceEdit,
    },
};

use crate::{
    ast::{InterfaceName, Name, Typedef, Typeref},
    capabilities::util::{get_file_from_db, leaf_at},
};

pub fn prepare_rename(
    db: &impl BaseDatabase,
    params: TextDocumentPositionParams,
) -> anyhow::Result<Option<PrepareRenameResponse>> {
    let file = get_file_from_db(&params.text_document.uri, db)?;
    let ast = get_ast(db, file);

    if let Some(leaf) = leaf_at(ast, params.position) {
        let leaf = leaf.lower();
        if leaf.is::<Name>() || leaf.is::<InterfaceName>() {
            return Ok(Some(PrepareRenameResponse::Range(leaf.get_lsp_range())));
        }
    }
    return Ok(None);
}

pub fn rename(
    db: &impl BaseDatabase,
    params: RenameParams,
) -> anyhow::Result<Option<WorkspaceEdit>> {
    let uri = &params.text_document_position.text_document.uri;
    let file = get_file_from_db(uri, db)?;
    let ast = get_ast(db, file);
    let document_bytes = file.document(db).as_bytes();

    let Some(leaf) = leaf_at(ast, params.text_document_position.position) else {
        return Ok(None);
    };

    let leaf = leaf.lower();
    if !leaf.is::<Name>() && !leaf.is::<InterfaceName>() {
        return Ok(None);
    }

    let old_name = leaf.get_text(document_bytes)?;
    let mut edits = Vec::new();

    let decl = leaf.get_parent(ast).unwrap().lower();
    if decl.is::<Typedef>() || decl.is::<Typeref>() {
        ast.iter().for_each(|node| {
            let lower = node.lower();
            if let Some(typedef) = lower.downcast_ref::<Typedef>() {
                let name = typedef.name.cast(ast);
                if name.get_text(document_bytes).unwrap() == old_name {
                    edits.push(TextEdit::new(name.get_lsp_range(), params.new_name.clone()));
                }
            }

            if let Some(typeref) = lower.downcast_ref::<Typeref>() {
                if typeref.get_text(document_bytes).unwrap() == old_name {
                    edits.push(TextEdit::new(
                        typeref.get_lsp_range(),
                        params.new_name.clone(),
                    ));
                }
            }
        });
    } else {
        edits.push(TextEdit::new(leaf.get_lsp_range(), params.new_name.clone()));
    }

    return Ok(Some(WorkspaceEdit {
        changes: Some([(uri.clone(), edits)].into()),
        ..Default::default()
    }));
}
