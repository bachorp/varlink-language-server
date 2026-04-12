use auto_lsp::{
    anyhow,
    core::ast::AstNode,
    default::db::{
        BaseDatabase,
        tracked::{ParsedAst, get_ast},
    },
    lsp_types::{
        PrepareRenameResponse, Range, RenameParams, TextDocumentPositionParams, TextEdit,
        WorkspaceEdit,
    },
};

use crate::{
    ast::{ErrorName, InterfaceName, MethodName, Typedef, TypedefName, Typeref},
    util::{get_file_from_db, leaf_at, walk_up},
};

fn find_type(ast: &ParsedAst, document_bytes: &[u8], old_name: &str) -> Option<Vec<Range>> {
    let mut edits = Vec::new();
    let mut n = 0;
    ast.iter().for_each(|node| {
        let lower = node.lower();
        if let Some(typedef) = lower.downcast_ref::<Typedef>() {
            let name = typedef.name.cast(ast);
            if name.get_text(document_bytes).unwrap() == old_name {
                edits.push(name.get_lsp_range());
                n += 1;
            }
        }

        if let Some(typeref) = lower.downcast_ref::<Typeref>() {
            let name = typeref.children.cast(ast);
            if name.get_text(document_bytes).unwrap() == old_name {
                edits.push(typeref.get_lsp_range());
            }
        }
    });

    if n == 1 { Some(edits) } else { None }
}

pub fn prepare_rename(
    db: &impl BaseDatabase,
    params: TextDocumentPositionParams,
) -> anyhow::Result<Option<PrepareRenameResponse>> {
    let file = get_file_from_db(&params.text_document.uri, db)?;
    let ast = get_ast(db, file);

    Ok(leaf_at(ast, params.position)
        .and_then(|leaf| {
            let leaf = leaf.lower();
            if let Some(interface_name) = walk_up::<InterfaceName>(ast, leaf) {
                Some(interface_name.get_lsp_range())
            } else if let Some(typedef_name) = walk_up::<TypedefName>(ast, leaf) {
                Some(typedef_name.get_lsp_range())
            } else if let Some(error_name) = walk_up::<ErrorName>(ast, leaf) {
                Some(error_name.get_lsp_range())
            } else if let Some(method_name) = walk_up::<MethodName>(ast, leaf) {
                Some(method_name.get_lsp_range())
            } else if let Some(typeref) = walk_up::<Typeref>(ast, leaf) {
                let document_bytes = file.document(db).as_bytes();
                find_type(
                    ast,
                    document_bytes,
                    typeref.get_text(document_bytes).unwrap(),
                )
                .map(|_| typeref.get_lsp_range())
            } else {
                None
            }
        })
        .map(|range| PrepareRenameResponse::Range(range)))
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

    let edits = {
        if let Some(interface_name) = walk_up::<InterfaceName>(ast, leaf) {
            // FIXME: `get_lsp_range` does not respect the encoding (also other places)
            vec![interface_name.get_lsp_range()]
        } else if let Some(error_name) = walk_up::<ErrorName>(ast, leaf) {
            vec![error_name.get_lsp_range()]
        } else if let Some(method_name) = walk_up::<MethodName>(ast, leaf) {
            vec![method_name.get_lsp_range()]
        } else if let Some(typedef_name) = walk_up::<TypedefName>(ast, leaf) {
            find_type(
                ast,
                document_bytes,
                typedef_name.get_text(document_bytes).unwrap(),
            )
            .unwrap_or(vec![typedef_name.get_lsp_range()])
        } else if let Some(typeref_name) = walk_up::<Typeref>(ast, leaf) {
            find_type(
                ast,
                document_bytes,
                typeref_name.get_text(document_bytes).unwrap(),
            )
            .unwrap_or(vec![])
        } else {
            vec![]
        }
    }
    .iter()
    .map(|range| TextEdit::new(*range, params.new_name.clone()))
    .collect();

    return Ok(Some(WorkspaceEdit {
        changes: Some([(uri.clone(), edits)].into()),
        ..Default::default()
    }));
}
