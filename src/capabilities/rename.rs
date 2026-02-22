use auto_lsp::{
    anyhow,
    core::ast::AstNode,
    default::db::{BaseDatabase, tracked::get_ast},
    lsp_types::{
        PrepareRenameResponse, RenameParams, TextDocumentPositionParams, TextEdit, WorkspaceEdit,
    },
};

use crate::{
    ast::{Name, Typedef, Typeref},
    capabilities::util::{get_file_from_db, subtree_at},
};

// TODO: This should be a util
fn covers(range: auto_lsp::lsp_types::Range, pos: auto_lsp::lsp_types::Position) -> bool {
    let (s, e) = (range.start, range.end);
    (s.line, s.character) <= (pos.line, pos.character)
        && (pos.line, pos.character) <= (e.line, e.character)
}

pub fn prepare_rename(
    db: &impl BaseDatabase,
    params: TextDocumentPositionParams,
) -> anyhow::Result<Option<PrepareRenameResponse>> {
    let file = get_file_from_db(&params.text_document.uri, db)?;
    let ast = get_ast(db, file);
    let pos = params.position;
    let candidates = &ast.nodes[subtree_at(ast, pos.line, pos.character)];

    for n in candidates.iter().rev() {
        if covers(n.get_lsp_range(), pos) {
            if let Some(name) = n.lower().downcast_ref::<Name>() {
                return Ok(Some(PrepareRenameResponse::Range(name.get_lsp_range())));
            }
        }
    }

    Ok(None)
}

pub fn rename(
    db: &impl BaseDatabase,
    params: RenameParams,
) -> anyhow::Result<Option<WorkspaceEdit>> {
    let uri = &params.text_document_position.text_document.uri;
    let file = get_file_from_db(uri, db)?;
    let ast = get_ast(db, file);
    let pos = params.text_document_position.position;
    let document_bytes = file.document(db).as_bytes();
    let new_name = &params.new_name;

    let candidates = &ast.nodes[subtree_at(ast, pos.line, pos.character)];

    let name_node = candidates.iter().rev().find_map(|n| {
        if covers(n.get_lsp_range(), pos) {
            n.lower().downcast_ref::<Name>()
        } else {
            None
        }
    });

    let Some(name_node) = name_node else {
        return Ok(None);
    };

    let old_name = name_node.get_text(document_bytes)?;

    let decl = name_node.get_parent(ast).unwrap();
    if decl.lower().is::<Typedef>() || decl.lower().is::<Typeref>() {
        return Ok(Some(rename_type(
            ast,
            document_bytes,
            uri,
            old_name,
            new_name,
        )));
    }

    let edits = vec![TextEdit::new(name_node.get_lsp_range(), new_name.clone())];
    let changes = [(uri.clone(), edits)].into();
    return Ok(Some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    }));
}

fn rename_type(
    ast: &auto_lsp::default::db::tracked::ParsedAst,
    document_bytes: &[u8],
    uri: &auto_lsp::lsp_types::Url,
    old_name: &str,
    new_name: &str,
) -> WorkspaceEdit {
    let mut edits = Vec::new();

    ast.iter().for_each(|node| {
        let lower = node.lower();
        if let Some(typedef) = lower.downcast_ref::<Typedef>() {
            let name = typedef.name.cast(ast);
            if name.get_text(document_bytes).unwrap() == old_name {
                edits.push(TextEdit::new(name.get_lsp_range(), new_name.to_string()));
            }
        }
        if let Some(typeref) = lower.downcast_ref::<Typeref>() {
            if typeref.get_text(document_bytes).unwrap() == old_name {
                edits.push(TextEdit::new(typeref.get_lsp_range(), new_name.to_string()));
            }
        }
    });

    let changes = [(uri.clone(), edits)].into();
    WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    }
}
