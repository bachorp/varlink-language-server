use auto_lsp::{
    anyhow::{self, Ok},
    core::ast::AstNode,
    default::db::{
        BaseDatabase,
        tracked::{ParsedAst, get_ast},
    },
    lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind, Range},
};

use crate::{
    ast::{Comment, InterfaceName, Name, Typedef, Typeref},
    capabilities::util::{get_file_from_db, leaf_at},
};

fn markup(docstring_rev: &[&str], code: &str, range: Range) -> Hover {
    let mut value = format!("```varlink\n{}\n```", code);
    if !docstring_rev.is_empty() {
        value.push_str("\n\n---\n");
        for line in docstring_rev.iter().rev() {
            value.push_str(line.trim_start_matches('#').trim_start());
            value.push('\n');
        }
    }

    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        }),
        range: Some(range),
    }
}

// FIXME: Naming, merge with `markup`
fn get_docstring<'a>(
    node: &dyn AstNode,
    nodes: &ParsedAst,
    document_bytes: &'a [u8],
) -> Vec<&'a str> {
    let id = node.get_id();
    let mut i = nodes.partition_point(|x| x.get_id() < id);
    let mut l = node.get_start_position().line as i32 - 1;
    let mut comments = Vec::new();
    while nodes[i].get_start_position().line as i32 >= l {
        if let Some(comment) = nodes[i].lower().downcast_ref::<Comment>() {
            if comment.get_start_position().character != 0 {
                break;
            }

            comments.push(comment.get_text(document_bytes).unwrap());
            l -= 1;
        }

        if i == 0 {
            break;
        }

        i -= 1;
    }

    return comments;
}

pub fn hover(db: &impl BaseDatabase, params: HoverParams) -> anyhow::Result<Option<Hover>> {
    let file = get_file_from_db(&params.text_document_position_params.text_document.uri, db)?;
    let ast = get_ast(db, file);
    let position = params.text_document_position_params.position;
    let document_bytes = file.document(db).as_bytes();

    let leaf = leaf_at(&ast.nodes, position.line, position.character).unwrap();
    if leaf.is::<Name>() || leaf.is::<InterfaceName>() {
        let parent = leaf.get_parent(ast).unwrap().lower();
        if let Some(typeref) = parent.downcast_ref::<Typeref>() {
            let name = typeref.get_text(document_bytes)?;
            let mut def = None;
            for node in ast.iter() {
                if let Some(typedef) = node.lower().downcast_ref::<Typedef>()
                    && typedef.name.cast(ast).get_text(document_bytes).unwrap() == name
                {
                    if def.is_some() {
                        return Ok(None);
                    }

                    def = Some(typedef);
                }
            }

            if let Some(def) = def {
                // TODO: Deduplicate
                return Ok(Some(markup(
                    &get_docstring(def, ast, document_bytes),
                    def.get_text(document_bytes)?,
                    def.get_lsp_range(),
                )));
            }
        } else {
            return Ok(Some(markup(
                &get_docstring(parent, ast, document_bytes),
                parent.get_text(document_bytes)?,
                parent.get_lsp_range(),
            )));
        }
    }

    Ok(None)
}
