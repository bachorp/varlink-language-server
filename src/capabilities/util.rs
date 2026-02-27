use auto_lsp::{
    anyhow,
    core::ast::AstNode,
    default::db::{BaseDatabase, file::File},
    lsp_types::{Position, Url},
};

// It would be great to have some of these upstream

pub fn get_file_from_db(uri: &Url, db: &impl BaseDatabase) -> Result<File, anyhow::Error> {
    db.get_file(uri)
        .ok_or_else(|| anyhow::format_err!("Unknown file: {}", uri))
}

// Note that the given nodes are assumed to be ordered by their starting position

// Finds the rightmost leaf at or before the given position
fn prec_at<'a>(nodes: &'a [Box<dyn AstNode>], pos: Position) -> Option<&'a Box<dyn AstNode>> {
    let cutoff = nodes.partition_point(|p| {
        let start = p.get_lsp_range().start;
        (start.line, start.character) <= (pos.line, pos.character)
    });

    if cutoff == 0 {
        None
    } else {
        Some(&nodes[cutoff - 1])
    }
}

// Finds the leaf at the given position
pub fn leaf_at<'a>(nodes: &'a [Box<dyn AstNode>], pos: Position) -> Option<&'a Box<dyn AstNode>> {
    prec_at(nodes, pos).and_then(|candidate| {
        let end = candidate.get_lsp_range().end;
        if (end.line, end.character) > (pos.line, pos.character) {
            Some(candidate)
        } else {
            None
        }
    })
}

// Finds the deepest node at the given position
pub fn most_specific_at<'a>(
    nodes: &'a [Box<dyn AstNode>],
    pos: Position,
) -> Option<&'a Box<dyn AstNode>> {
    fn walk_up<'a>(
        nodes: &'a [Box<dyn AstNode>],
        node: &'a Box<dyn AstNode>,
        pos: Position,
    ) -> Option<&'a Box<dyn AstNode>> {
        let end = node.get_lsp_range().end;
        if (end.line, end.character) > (pos.line, pos.character) {
            Some(node)
        } else {
            node.get_parent(nodes).and_then(|p| walk_up(nodes, p, pos))
        }
    }

    prec_at(nodes, pos).and_then(|candidate| walk_up(nodes, candidate, pos))
}

// Finds the deepest node of some type at the given position
pub fn capture_at<'a, T: AstNode>(nodes: &'a [Box<dyn AstNode>], pos: Position) -> Option<&'a T> {
    fn walk_up<'a, T: AstNode>(
        nodes: &'a [Box<dyn AstNode>],
        node: &'a Box<dyn AstNode>,
        pos: Position,
    ) -> Option<&'a T> {
        node.lower()
            .downcast_ref::<T>()
            .or(node.get_parent(nodes).and_then(|p| walk_up(nodes, p, pos)))
    }

    most_specific_at(nodes, pos).and_then(|n| walk_up::<T>(nodes, n, pos))
}
