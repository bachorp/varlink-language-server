#![allow(deprecated)]

use crate::ast::{Error, InterfaceDeclaration, Method, Typedef};
use crate::capabilities::util::get_file_from_db;
use auto_lsp::core::ast::AstNode;
use auto_lsp::core::dispatch_once;
use auto_lsp::core::document::Document;
use auto_lsp::core::document_symbols_builder::DocumentSymbolsBuilder;
use auto_lsp::default::db::BaseDatabase;
use auto_lsp::default::db::file::File;
use auto_lsp::default::db::tracked::{ParsedAst, get_ast};
use auto_lsp::lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, Location, OneOf, WorkspaceSymbol,
    WorkspaceSymbolParams, WorkspaceSymbolResponse,
};
use auto_lsp::{anyhow, lsp_types};

fn _symbols(db: &impl BaseDatabase, file: &File) -> Vec<DocumentSymbol> {
    let doc = file.document(db);
    let mut builder = DocumentSymbolsBuilder::default();

    let ast = get_ast(db, *file);
    ast.iter().for_each(|node| {
        dispatch_once!(
            node.lower(),
            [
                InterfaceDeclaration => build_document_symbols(&doc, ast, &mut builder),
                Error => build_document_symbols(&doc, ast, &mut builder),
                Typedef => build_document_symbols(&doc, ast, &mut builder),
                Method => build_document_symbols(&doc, ast, &mut builder)
            ]
        );
    });
    builder.finalize()
}

impl InterfaceDeclaration {
    pub(crate) fn build_document_symbols(
        &self,
        doc: &Document,
        ast: &ParsedAst,
        builder: &mut DocumentSymbolsBuilder,
    ) {
        let name = self.name.cast(ast);
        builder.push_symbol(lsp_types::DocumentSymbol {
            name: name.get_text(doc.as_bytes()).unwrap().to_string(),
            kind: lsp_types::SymbolKind::NAMESPACE,
            range: self.get_lsp_range(),
            selection_range: name.get_lsp_range(),
            tags: None,
            detail: None,
            deprecated: None,
            children: None,
        });
    }
}

impl Error {
    pub(crate) fn build_document_symbols(
        &self,
        doc: &Document,
        ast: &ParsedAst,
        builder: &mut DocumentSymbolsBuilder,
    ) {
        let name = self.name.cast(ast);
        builder.push_symbol(lsp_types::DocumentSymbol {
            name: name.get_text(doc.as_bytes()).unwrap().to_string(),
            kind: lsp_types::SymbolKind::EVENT,
            range: self.get_lsp_range(),
            selection_range: name.get_lsp_range(),
            tags: None,
            detail: None,
            deprecated: None,
            children: None,
        });
    }
}

impl Method {
    pub(crate) fn build_document_symbols(
        &self,
        doc: &Document,
        ast: &ParsedAst,
        builder: &mut DocumentSymbolsBuilder,
    ) {
        let name = self.name.cast(ast);
        builder.push_symbol(lsp_types::DocumentSymbol {
            name: name.get_text(doc.as_bytes()).unwrap().to_string(),
            kind: lsp_types::SymbolKind::METHOD,
            range: self.get_lsp_range(),
            selection_range: name.get_lsp_range(),
            tags: None,
            detail: None,
            deprecated: None,
            children: None,
        });
    }
}

impl Typedef {
    pub(crate) fn build_document_symbols(
        &self,
        doc: &Document,
        ast: &ParsedAst,
        builder: &mut DocumentSymbolsBuilder,
    ) {
        let name = self.name.cast(ast);
        builder.push_symbol(lsp_types::DocumentSymbol {
            name: name.get_text(doc.as_bytes()).unwrap().to_string(),
            kind: lsp_types::SymbolKind::CLASS,
            range: self.get_lsp_range(),
            selection_range: name.get_lsp_range(),
            tags: None,
            detail: None,
            deprecated: None,
            children: None,
        });
    }
}

pub fn document_symbols(
    db: &impl BaseDatabase,
    params: DocumentSymbolParams,
) -> anyhow::Result<Option<DocumentSymbolResponse>> {
    let file = get_file_from_db(&params.text_document.uri, db)?;
    Ok(Some(DocumentSymbolResponse::Nested(_symbols(db, &file))))
}

pub fn workspace_symbols(
    db: &impl BaseDatabase,
    _params: WorkspaceSymbolParams,
) -> anyhow::Result<Option<WorkspaceSymbolResponse>> {
    let mut symbols = vec![];

    db.get_files().iter().for_each(|file| {
        let file = *file;
        let url = file.url(db);

        symbols.extend(
            _symbols(db, &file)
                .into_iter()
                .map(|p| WorkspaceSymbol {
                    name: p.name,
                    kind: p.kind,
                    tags: None,
                    container_name: None,
                    location: OneOf::Left(Location {
                        uri: url.to_owned(),
                        range: p.range,
                    }),
                    data: None,
                })
                .collect::<Vec<_>>(),
        );
    });
    Ok(Some(WorkspaceSymbolResponse::Nested(symbols)))
}
