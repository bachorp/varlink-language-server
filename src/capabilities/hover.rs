use auto_lsp::{
    anyhow::{self, Ok},
    core::ast::AstNode,
    default::db::{BaseDatabase, tracked::get_ast},
    lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind},
};

use crate::{
    ast::{Comment, InterfaceName, Name, Typedef, Typeref},
    capabilities::util::{get_file_from_db, leaf_at},
};

pub fn hover(db: &impl BaseDatabase, params: HoverParams) -> anyhow::Result<Option<Hover>> {
    let file = get_file_from_db(&params.text_document_position_params.text_document.uri, db)?;
    let ast = get_ast(db, file);
    let document_bytes = file.document(db).as_bytes();

    let leaf = leaf_at(&ast.nodes, params.text_document_position_params.position);
    let Some(leaf) = leaf else { return Ok(None) };
    let leaf = leaf.lower();
    // TODO: We could have a hover in many more cases
    if !leaf.is::<Name>() && !leaf.is::<InterfaceName>() {
        return Ok(None);
    }

    let parent = leaf.get_parent(ast).unwrap().lower();
    let target = {
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

            let Some(def) = def else {
                return Ok(None);
            };

            def
        } else {
            parent
        }
    };

    let id = target.get_id();
    let mut i = ast.partition_point(|x| x.get_id() < id);
    let mut l = target.get_start_position().line as i32 - 1;
    let mut comments_rev = Vec::new();
    while ast[i].get_start_position().line as i32 >= l {
        if let Some(comment) = ast[i].lower().downcast_ref::<Comment>() {
            if comment.get_start_position().character != 0 {
                break;
            }

            comments_rev.push(comment.get_text(document_bytes).unwrap());
            l -= 1;
        }

        if i == 0 {
            break;
        }

        i -= 1;
    }

    let mut value = format!("```varlink\n{}\n```", target.get_text(document_bytes)?);
    if !comments_rev.is_empty() {
        value.push_str("\n\n---\n");
        for line in comments_rev.iter().rev() {
            // TODO: Use `trim_prefix` when available (https://github.com/rust-lang/rust/issues/142312)
            value.push_str(
                line.trim_start_matches("#")
                    .strip_prefix(" ")
                    .unwrap_or(line),
            );
            value.push('\n');
        }
    }

    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        }),
        range: Some(target.get_lsp_range()),
    }))
}
