use auto_lsp::{
    anyhow,
    core::ast::AstNode,
    default::db::{
        BaseDatabase,
        tracked::{ParsedAst, get_ast},
    },
    lsp_types::{DocumentHighlight, DocumentHighlightKind, DocumentHighlightParams},
};

use crate::{
    ast::{
        self, Any, Bool, EnumMemberName, ErrorName, Float, Int, InterfaceName, KeywordError,
        KeywordInterface, KeywordMethod, KeywordType, MethodName, Object, StructFieldName, Typedef,
        TypedefName, Typeref,
    },
    util::{get_file_from_db, leaf_at, walk_up},
};

fn custom_type(ast: &ParsedAst, document_bytes: &[u8], name: &str) -> Vec<DocumentHighlight> {
    ast.iter()
        .filter_map(|node| {
            let lower = node.lower();
            if let Some(typedef) = lower.downcast_ref::<Typedef>() {
                let other = typedef.name.cast(ast);
                if other.get_text(document_bytes).unwrap() == name {
                    return Some(DocumentHighlight {
                        range: other.get_lsp_range(),
                        kind: Some(DocumentHighlightKind::WRITE),
                    });
                }
            }

            if let Some(typeref) = lower.downcast_ref::<Typeref>() {
                if typeref.get_text(document_bytes).unwrap() == name {
                    return Some(DocumentHighlight {
                        range: typeref.get_lsp_range(),
                        kind: Some(DocumentHighlightKind::READ),
                    });
                }
            }
            None
        })
        .collect()
}

fn primitive<T: AstNode>(ast: &ParsedAst) -> Vec<DocumentHighlight> {
    ast.iter()
        .filter_map(|node| {
            node.lower()
                .downcast_ref::<T>()
                .map(|node| DocumentHighlight {
                    range: node.get_lsp_range(),
                    kind: Some(DocumentHighlightKind::READ),
                })
        })
        .collect()
}

fn single<T: AstNode>(
    ast: &ParsedAst,
    leaf: &dyn AstNode,
    kind: DocumentHighlightKind,
) -> Option<Vec<DocumentHighlight>> {
    walk_up::<T>(ast, leaf).map(|node| {
        vec![DocumentHighlight {
            range: node.get_lsp_range(),
            kind: Some(kind),
        }]
    })
}

pub fn highlight(
    db: &impl BaseDatabase,
    params: DocumentHighlightParams,
) -> anyhow::Result<Option<Vec<DocumentHighlight>>> {
    let file = get_file_from_db(&params.text_document_position_params.text_document.uri, db)?;
    let ast = get_ast(db, file);
    let document_bytes = file.document(db).as_bytes();

    let Some(leaf) = leaf_at(ast, params.text_document_position_params.position) else {
        return Ok(None);
    };
    let leaf = leaf.lower();

    let result = None
        .or_else(|| {
            walk_up::<Typeref>(ast, leaf).map(|typeref| {
                custom_type(
                    ast,
                    document_bytes,
                    typeref.get_text(document_bytes).unwrap(),
                )
            })
        })
        .or_else(|| {
            walk_up::<TypedefName>(ast, leaf).map(|typedef| {
                custom_type(
                    ast,
                    document_bytes,
                    typedef.get_text(document_bytes).unwrap(),
                )
            })
        })
        .or_else(|| walk_up::<Bool>(ast, leaf).map(|_| primitive::<Bool>(ast)))
        .or_else(|| walk_up::<Int>(ast, leaf).map(|_| primitive::<Int>(ast)))
        .or_else(|| walk_up::<Float>(ast, leaf).map(|_| primitive::<Float>(ast)))
        .or_else(|| walk_up::<ast::String>(ast, leaf).map(|_| primitive::<ast::String>(ast)))
        .or_else(|| walk_up::<Object>(ast, leaf).map(|_| primitive::<Object>(ast)))
        .or_else(|| walk_up::<Any>(ast, leaf).map(|_| primitive::<Any>(ast)))
        .or_else(|| single::<InterfaceName>(ast, leaf, DocumentHighlightKind::WRITE))
        .or_else(|| single::<ErrorName>(ast, leaf, DocumentHighlightKind::WRITE))
        .or_else(|| single::<MethodName>(ast, leaf, DocumentHighlightKind::WRITE))
        .or_else(|| single::<EnumMemberName>(ast, leaf, DocumentHighlightKind::TEXT))
        .or_else(|| single::<StructFieldName>(ast, leaf, DocumentHighlightKind::TEXT))
        .or_else(|| single::<KeywordInterface>(ast, leaf, DocumentHighlightKind::TEXT))
        .or_else(|| single::<KeywordError>(ast, leaf, DocumentHighlightKind::TEXT))
        .or_else(|| single::<KeywordMethod>(ast, leaf, DocumentHighlightKind::TEXT))
        .or_else(|| single::<KeywordType>(ast, leaf, DocumentHighlightKind::TEXT));

    Ok(result)
}
