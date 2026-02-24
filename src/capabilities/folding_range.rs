use auto_lsp::{
    anyhow,
    default::db::{BaseDatabase, tracked::get_ast},
    lsp_types::{FoldingRange, FoldingRangeParams},
};

use crate::{
    ast::{Enum, Error, Method, Struct, Typedef},
    capabilities::util::get_file_from_db,
};

pub fn folding_range(
    db: &impl BaseDatabase,
    params: FoldingRangeParams,
) -> anyhow::Result<Option<Vec<FoldingRange>>> {
    let file = get_file_from_db(&params.text_document.uri, db)?;
    let ast = get_ast(db, file);

    let ranges = ast
        .iter()
        .filter_map(|node| {
            let node = node.lower();
            if node.is::<Method>()
                || node.is::<Error>()
                || node.is::<Typedef>()
                || node.is::<Struct>()
                || node.is::<Enum>()
            {
                let range = node.get_lsp_range();
                Some(FoldingRange {
                    start_line: range.start.line,
                    start_character: Some(range.start.character),
                    end_line: range.end.line,
                    end_character: Some(range.end.character),
                    ..Default::default()
                })
            } else {
                None
            }
        })
        .collect();

    Ok(Some(ranges))
}
