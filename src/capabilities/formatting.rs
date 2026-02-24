use auto_lsp::{
    anyhow,
    default::db::BaseDatabase,
    lsp_types::{DocumentFormattingParams, Position, Range, TextEdit},
};
use varlinkfmt_core::{Indent, format, mk_language};

use crate::capabilities::util::get_file_from_db;

pub fn formatting(
    db: &impl BaseDatabase,
    params: DocumentFormattingParams,
) -> anyhow::Result<Option<Vec<TextEdit>>> {
    let indent = match params.options.insert_spaces {
        false => Indent::Tab,
        true => Indent::Spaces(params.options.tab_size as usize),
    };

    let document = get_file_from_db(&params.text_document.uri, db)?.document(db);
    let formatted = format(&mk_language(indent), &mut document.as_bytes())
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(Some(vec![TextEdit::new(
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: usize::from(document.texter.br_indexes.row_count()) as u32,
                character: 0,
            },
        },
        formatted,
    )]))
}
