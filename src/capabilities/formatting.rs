use auto_lsp::{
    anyhow,
    core::errors::ParseErrorAccumulator,
    default::db::{BaseDatabase, tracked::get_ast},
    lsp_types::{DocumentFormattingParams, Position, Range, TextEdit},
};
use varlinkfmt_core::{Indent, formatter_tree, mk_language};

use crate::util::get_file_from_db;

pub fn formatting(
    db: &impl BaseDatabase,
    params: DocumentFormattingParams,
) -> anyhow::Result<Option<Vec<TextEdit>>> {
    let file = get_file_from_db(&params.text_document.uri, db)?;
    if get_ast::accumulated::<ParseErrorAccumulator>(db, file)
        .into_iter()
        .peekable()
        .peek()
        .is_some()
    {
        return Ok(None);
    }

    let document = file.document(db);

    let mut output = Vec::new();
    formatter_tree(
        document.tree.clone().into(),
        &document.as_str(),
        &mut output,
        &mk_language(match params.options.insert_spaces {
            false => Indent::Tab,
            true => Indent::Spaces(params.options.tab_size as usize),
        }),
        Default::default(),
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))?;

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
        String::from_utf8(output).unwrap(),
    )]))
}
