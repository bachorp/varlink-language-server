use std::collections::{BTreeSet, HashMap};

use auto_lsp::core::ast::AstNode;
use auto_lsp::core::document::Document;
use auto_lsp::core::errors::ParseErrorAccumulator;
use auto_lsp::default::db::BaseDatabase;
use auto_lsp::default::db::file::File;
use auto_lsp::default::db::tracked::{ParsedAst, get_ast};
use auto_lsp::lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, DocumentDiagnosticParams,
    DocumentDiagnosticReport, DocumentDiagnosticReportResult, FullDocumentDiagnosticReport,
    Location, Position, Range, RelatedFullDocumentDiagnosticReport, Url, WorkspaceDiagnosticParams,
    WorkspaceDiagnosticReport, WorkspaceDiagnosticReportResult, WorkspaceDocumentDiagnosticReport,
    WorkspaceFullDocumentDiagnosticReport,
};
use auto_lsp::{anyhow, lsp_types};

use crate::ast::{Enum, Error, Method, Struct, Typedef, Typeref};
use crate::capabilities::util::get_file_from_db;

fn get_parse_errors(db: &impl BaseDatabase, file: File) -> Vec<Diagnostic> {
    let mut error_positions: BTreeSet<(Position, Position)> = BTreeSet::new();
    get_ast::accumulated::<ParseErrorAccumulator>(db, file)
        .into_iter()
        .filter_map(|d| {
            let diagnostic: Diagnostic = d.into();
            let position = (diagnostic.range.start, diagnostic.range.end);
            if error_positions.contains(&position) {
                None
            } else {
                error_positions.insert(position);
                Some(Diagnostic {
                    message: "syntax error".to_string(),
                    code: None,
                    ..d.into()
                })
            }
        })
        .collect()
}

fn check_trailing_newline(ast: &ParsedAst, document: &Document) -> Vec<Diagnostic> {
    if let Some(last) = ast.last() {
        let row_count = usize::from(document.texter.br_indexes.row_count());
        if last.get_range().end_point.row + 1 == row_count {
            let end_of_document = Position {
                line: row_count as u32,
                character: 0,
            };
            return vec![Diagnostic {
                range: Range {
                    start: end_of_document,
                    end: end_of_document,
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "missing trailing newline".into(),
                ..Diagnostic::default()
            }];
        }
    }

    return Vec::new();
}

fn check_defs(kind: &str, collection: &HashMap<&str, Vec<Range>>, uri: &Url) -> Vec<Diagnostic> {
    collection
        .iter()
        .filter_map(|(name, occurences)| {
            if occurences.len() == 1 {
                None
            } else {
                Some(
                    occurences
                        .iter()
                        .enumerate()
                        .map(move |(i, occurence)| Diagnostic {
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
                                            uri: uri.clone(),
                                            range: *o,
                                        },
                                        message: format!("also declared here"),
                                    })
                                    .collect()
                            }),
                            ..Diagnostic::default()
                        }),
                )
            }
        })
        .flatten()
        .collect()
}

fn check_typerefs(
    typedefs: &HashMap<&str, Vec<Range>>,
    typerefs: &HashMap<&str, Vec<Range>>,
    uri: &Url,
) -> Vec<Diagnostic> {
    typerefs
        .iter()
        .flat_map(|(name, occurences)| {
            let def = typedefs.get(name);
            occurences.iter().filter_map(move |occurence| match def {
                None => Some(Diagnostic {
                    range: *occurence,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("unknown type `{}`", name),
                    ..Diagnostic::default()
                }),
                Some(definitions) => {
                    if definitions.len() == 1 {
                        None
                    } else {
                        Some(Diagnostic {
                            range: *occurence,
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: format!("type `{}` declared multiple times", name),
                            related_information: Some({
                                definitions
                                    .iter()
                                    .map(|o| DiagnosticRelatedInformation {
                                        location: Location {
                                            uri: uri.clone(),
                                            range: *o,
                                        },
                                        message: format!("declared here"),
                                    })
                                    .collect()
                            }),
                            ..Diagnostic::default()
                        })
                    }
                }
            })
        })
        .collect()
}

fn _diagnostics(db: &impl BaseDatabase, file: &File) -> Vec<Diagnostic> {
    let ast = get_ast(db, *file);
    let document = file.document(db);
    let uri = file.url(db);
    let document_bytes = document.as_bytes();

    let mut items: Vec<Diagnostic> = Vec::new();
    items.append(&mut get_parse_errors(db, *file));
    items.append(&mut check_trailing_newline(ast, document));

    let typedefs = {
        let mut result = HashMap::new();
        ast.iter().for_each(|node| {
            if let Some(typedef) = node.lower().downcast_ref::<Typedef>() {
                let name = typedef.name.cast(ast);
                result
                    .entry(name.get_text(document_bytes).unwrap())
                    .or_insert_with(Vec::new)
                    .push(name.get_lsp_range());
            }
        });

        result
    };

    items.append(&mut check_defs("type definition", &typedefs, uri));

    let errors = {
        let mut result = HashMap::new();
        ast.iter().for_each(|node| {
            if let Some(typedef) = node.lower().downcast_ref::<Error>() {
                let name = typedef.name.cast(ast);
                result
                    .entry(name.get_text(document_bytes).unwrap())
                    .or_insert_with(Vec::new)
                    .push(name.get_lsp_range());
            }
        });

        result
    };

    items.append(&mut check_defs("error", &errors, uri));

    let methods = {
        let mut result = HashMap::new();
        ast.iter().for_each(|node| {
            if let Some(typedef) = node.lower().downcast_ref::<Method>() {
                let name = typedef.name.cast(ast);
                result
                    .entry(name.get_text(document_bytes).unwrap())
                    .or_insert_with(Vec::new)
                    .push(name.get_lsp_range());
            }
        });

        result
    };

    items.append(&mut check_defs("method", &methods, uri));

    let typerefs = {
        let mut result = HashMap::new();
        ast.iter().for_each(|node| {
            if let Some(typedef) = node.lower().downcast_ref::<Typeref>() {
                let name = typedef.children.cast(ast);
                result
                    .entry(name.get_text(document_bytes).unwrap())
                    .or_insert_with(Vec::new)
                    .push(name.get_lsp_range());
            }
        });

        result
    };

    items.append(&mut check_typerefs(&typedefs, &typerefs, uri));

    ast.iter().for_each(|node| {
        if let Some(struct_) = node.lower().downcast_ref::<Struct>() {
            let mut members = HashMap::new();
            struct_.member.iter().for_each(|member| {
                let name = member.cast(ast).name.cast(ast);
                members
                    .entry(name.get_text(document_bytes).unwrap())
                    .or_insert_with(Vec::new)
                    .push(name.get_lsp_range());
            });

            items.append(&mut check_defs("struct field", &members, uri));
        }
    });

    ast.iter().for_each(|node| {
        if let Some(enum_) = node.lower().downcast_ref::<Enum>() {
            let mut members = HashMap::new();
            enum_.member.iter().for_each(|member| {
                let name = member.cast(ast);
                members
                    .entry(name.get_text(document_bytes).unwrap())
                    .or_insert_with(Vec::new)
                    .push(name.get_lsp_range());
            });

            items.append(&mut check_defs("enum member", &members, uri));
        }
    });

    items
}

pub fn diagnostics(
    db: &impl BaseDatabase,
    params: DocumentDiagnosticParams,
) -> anyhow::Result<DocumentDiagnosticReportResult> {
    let file = get_file_from_db(&params.text_document.uri, db)?;
    let items = _diagnostics(db, &file);
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

pub fn workspace_diagnostics(
    db: &impl BaseDatabase,
    _params: WorkspaceDiagnosticParams,
) -> anyhow::Result<WorkspaceDiagnosticReportResult> {
    let items: Vec<lsp_types::WorkspaceDocumentDiagnosticReport> = db
        .get_files()
        .iter()
        .map(|file| {
            let items = _diagnostics(db, &file);
            WorkspaceDocumentDiagnosticReport::Full(WorkspaceFullDocumentDiagnosticReport {
                version: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items,
                },
                uri: file.url(db).clone(),
            })
        })
        .collect();

    Ok(WorkspaceDiagnosticReportResult::Report(
        WorkspaceDiagnosticReport { items },
    ))
}
