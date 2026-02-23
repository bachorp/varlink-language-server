use auto_lsp::{
    anyhow,
    core::ast::AstNode,
    default::db::{BaseDatabase, file::File},
    lsp_types::{Position, Url},
};

// NOTE: It would be great to have these upstream

pub fn get_file_from_db(uri: &Url, db: &impl BaseDatabase) -> Result<File, anyhow::Error> {
    db.get_file(uri)
        .ok_or_else(|| anyhow::format_err!("File not found in database: {}", uri))
}

pub fn leaf_at<'a>(nodes: &'a [Box<dyn AstNode>], pos: Position) -> Option<&'a Box<dyn AstNode>> {
    let start = nodes.partition_point(|p| {
        let end = p.get_lsp_range().end;
        (end.line, end.character) < (pos.line, pos.character)
    });

    let end = start
        + nodes[start..].partition_point(|p| {
            let start = p.get_lsp_range().start;
            (start.line, start.character) <= (pos.line, pos.character)
        });

    if start < end {
        Some(&nodes[end - 1])
    } else {
        None
    }
}
