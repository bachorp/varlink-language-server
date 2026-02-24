use std::collections::HashSet;

use auto_lsp::{
    anyhow,
    core::ast::AstNode,
    default::db::{BaseDatabase, tracked::get_ast},
    lsp_types::{
        CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, InsertTextFormat,
    },
};

use crate::{ast::Typedef, capabilities::util::get_file_from_db};

struct Snippet {
    label: &'static str,
    insert_text: &'static str,
}

const DECLARATIONS: &[Snippet] = &[
    Snippet {
        label: "interface",
        insert_text: "interface ${1}",
    },
    Snippet {
        label: "method",
        insert_text: "method ${1} (${2}) -> (${3})",
    },
    Snippet {
        label: "type",
        insert_text: "type ${1} (${2})",
    },
    Snippet {
        label: "error",
        insert_text: "error ${1} (${2})",
    },
];

const BUILTIN_TYPES: &[Snippet] = &[
    Snippet {
        label: "bool",
        insert_text: "bool",
    },
    Snippet {
        label: "int",
        insert_text: "int",
    },
    Snippet {
        label: "float",
        insert_text: "float",
    },
    Snippet {
        label: "string",
        insert_text: "string",
    },
    Snippet {
        label: "object",
        insert_text: "object",
    },
    Snippet {
        label: "any",
        insert_text: "any",
    },
];

pub fn completion(
    db: &impl BaseDatabase,
    params: CompletionParams,
) -> anyhow::Result<Option<CompletionResponse>> {
    let file = get_file_from_db(&params.text_document_position.text_document.uri, db)?;
    let ast = get_ast(db, file);
    let document_bytes = file.document(db).as_bytes();

    let pos = params.text_document_position.position;
    let line = file.document(db).texter.get_row(pos.line as usize).unwrap();

    let mut items: Vec<CompletionItem> = Vec::new();
    let line_before = &line[0..pos.character as usize - 1].trim(); // Trim the single typed character
    // We are on a blank line
    if *line_before == "" {
        for kind in DECLARATIONS {
            items.push(CompletionItem {
                label: kind.label.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                insert_text: Some(kind.insert_text.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }
    }

    // NOTE: Does not work across lines (would be allowed for `:`)
    if !line.contains("#")
        && (line_before.ends_with(":")
            || line_before.ends_with("?")
            || line_before.ends_with("[]")
            || line_before.ends_with("[string]"))
    {
        for kind in BUILTIN_TYPES {
            items.push(CompletionItem {
                label: kind.label.to_string(),
                kind: Some(CompletionItemKind::STRUCT),
                insert_text: Some(kind.insert_text.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        let mut typedefs = HashSet::new();
        ast.iter()
            .filter_map(|node| node.lower().downcast_ref::<Typedef>())
            .for_each(|typedef| {
                if let Ok(name) = typedef.name.cast(ast).get_text(document_bytes)
                    && !typedefs.contains(name)
                {
                    typedefs.insert(name);
                    items.push(CompletionItem {
                        label: name.to_string(),
                        kind: Some(CompletionItemKind::CLASS),
                        ..Default::default()
                    });
                }
            });
    }

    Ok(Some(CompletionResponse::Array(items)))
}
