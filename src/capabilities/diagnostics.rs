use std::collections::HashMap;

use auto_lsp::anyhow;
use auto_lsp::core::ast::AstNode;
use auto_lsp::core::errors::ParseErrorAccumulator;
use auto_lsp::default::db::BaseDatabase;
use auto_lsp::default::db::tracked::get_ast;
use auto_lsp::lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, DocumentDiagnosticParams,
    DocumentDiagnosticReport, DocumentDiagnosticReportResult, FullDocumentDiagnosticReport,
    Location, Position, Range, RelatedFullDocumentDiagnosticReport,
};

use crate::ast::{Enum, Eol, Error, Method, Struct, Typedef, Typeref};
use crate::capabilities::util::get_file_from_db;

pub fn diagnostics(
    db: &impl BaseDatabase,
    params: DocumentDiagnosticParams,
) -> anyhow::Result<DocumentDiagnosticReportResult> {
    let file = get_file_from_db(&params.text_document.uri, db)?;
    let document = file.document(db);
    let ast = get_ast(db, file);

    let mut items: Vec<Diagnostic> = get_ast::accumulated::<ParseErrorAccumulator>(db, file)
        .into_iter()
        .map(|d| d.into())
        .collect();

    let document_bytes = document.as_bytes();

    let (mut typedefs, mut errors, mut methods, mut typerefs) = (
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    );

    ast.iter().for_each(|node| {
        if let Some(typedef) = node.lower().downcast_ref::<Typedef>() {
            let name = typedef.name.cast(ast);

            typedefs
                .entry(name.get_text(document_bytes).unwrap())
                .or_insert_with(Vec::new)
                .push(name.get_lsp_range());
        }

        if let Some(error) = node.lower().downcast_ref::<Error>() {
            let name = error.name.cast(ast);

            errors
                .entry(name.get_text(document_bytes).unwrap())
                .or_insert_with(Vec::new)
                .push(name.get_lsp_range());
        }

        if let Some(method) = node.lower().downcast_ref::<Method>() {
            let name = method.name.cast(ast);

            methods
                .entry(name.get_text(document_bytes).unwrap())
                .or_insert_with(Vec::new)
                .push(name.get_lsp_range());
        }

        if let Some(typeref) = node.lower().downcast_ref::<Typeref>() {
            let name = typeref.children.cast(ast);

            typerefs
                .entry(name.get_text(document_bytes).unwrap())
                .or_insert_with(Vec::new)
                .push(name.get_lsp_range());
        }

        if let Some(struct_) = node.lower().downcast_ref::<Struct>() {
            let mut members = HashMap::new();
            struct_.member.iter().for_each(|member| {
                let name = member.cast(ast).name.cast(ast);
                members
                    .entry(name.get_text(document_bytes).unwrap())
                    .or_insert_with(Vec::new)
                    .push(name.get_lsp_range());
            });
            members.iter().for_each(|(name, occurences)| {
                if occurences.len() > 1 {
                    occurences.iter().enumerate().for_each(|(i, occurence)| {
                        items.push(Diagnostic {
                            range: *occurence,
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!("struct field `{}` declared multiple times", name),
                            related_information: Some({
                                occurences
                                    .iter()
                                    .enumerate()
                                    .filter(|(j, _)| *j != i)
                                    .map(|(_, o)| DiagnosticRelatedInformation {
                                        location: Location {
                                            uri: params.text_document.uri.clone(),
                                            range: *o,
                                        },
                                        message: format!("also declared here"),
                                    })
                                    .collect()
                            }),
                            ..Diagnostic::default()
                        });
                    });
                }
            });
        }

        if let Some(enum_) = node.lower().downcast_ref::<Enum>() {
            let mut members = HashMap::new();
            enum_.member.iter().for_each(|member| {
                let name = member.cast(ast);
                members
                    .entry(name.get_text(document_bytes).unwrap())
                    .or_insert_with(Vec::new)
                    .push(name.get_lsp_range());
            });
            members.iter().for_each(|(name, occurences)| {
                if occurences.len() > 1 {
                    occurences.iter().enumerate().for_each(|(i, occurence)| {
                        items.push(Diagnostic {
                            range: *occurence,
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!("enum member `{}` declared multiple times", name),
                            related_information: Some({
                                occurences
                                    .iter()
                                    .enumerate()
                                    .filter(|(j, _)| *j != i)
                                    .map(|(_, o)| DiagnosticRelatedInformation {
                                        location: Location {
                                            uri: params.text_document.uri.clone(),
                                            range: *o,
                                        },
                                        message: format!("also declared here"),
                                    })
                                    .collect()
                            }),
                            ..Diagnostic::default()
                        });
                    });
                }
            });
        }
    });

    typerefs.iter().for_each(|(name, occurences)| {
        if !typedefs.contains_key(name) {
            occurences.iter().for_each(|occurence| {
                items.push(Diagnostic {
                    range: *occurence,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("unknown type `{}`", name),
                    ..Diagnostic::default()
                });
            });
        }
    });

    for (kind, collection) in [
        ("type definition", typedefs),
        ("error", errors),
        ("method", methods),
    ]
    .iter()
    {
        collection.iter().for_each(|(name, occurences)| {
            if occurences.len() > 1 {
                occurences.iter().enumerate().for_each(|(i, occurence)| {
                    items.push(Diagnostic {
                        range: *occurence,
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: format!("{} `{}` declared multiple times", kind, name),
                        related_information: Some({
                            occurences
                                .iter()
                                .enumerate()
                                .filter(|(j, _)| *j != i)
                                .map(|(_, o)| DiagnosticRelatedInformation {
                                    location: Location {
                                        uri: params.text_document.uri.clone(),
                                        range: *o,
                                    },
                                    message: format!("also declared here"),
                                })
                                .collect()
                        }),
                        ..Diagnostic::default()
                    });
                });
            }
        });
    }

    if let Some(last) = ast.last() {
        if !(last.lower().is::<Eol>()
            // is top-level (below root)
            && last.get_parent(ast).map(|node| node.get_id())
                == ast.get_root().map(|node| node.get_id()))
        {
            let end_of_document = Position {
                line: usize::from(document.texter.br_indexes.row_count()).try_into()?,
                character: 0,
            };

            items.push(Diagnostic {
                range: Range {
                    start: end_of_document,
                    end: end_of_document,
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "missing trailing newline".into(),
                ..Diagnostic::default()
            });
        }
    }

    Ok(DocumentDiagnosticReportResult::Report(
        DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
            related_documents: None,
            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                result_id: None,
                items,
            },
        }),
    ))
}
