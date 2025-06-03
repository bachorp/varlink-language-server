#![allow(deprecated)]

use crate::ast::{Error, InterfaceDeclaration, Method, Typedef};
use auto_lsp::core::ast::AstNode;
use auto_lsp::core::dispatch;
use auto_lsp::core::document::Document;
use auto_lsp::core::document_symbols_builder::DocumentSymbolsBuilder;
use auto_lsp::default::db::BaseDatabase;
use auto_lsp::default::db::tracked::{ParsedAst, get_ast};
use auto_lsp::lsp_types::{DocumentSymbolParams, DocumentSymbolResponse};
use auto_lsp::{anyhow, lsp_types};

pub fn document_symbols(
    db: &impl BaseDatabase,
    params: DocumentSymbolParams,
) -> anyhow::Result<Option<DocumentSymbolResponse>> {
    let uri = params.text_document.uri;

    let file = db
        .get_file(&uri)
        .ok_or_else(|| anyhow::format_err!("File not found in workspace"))?;

    let doc = file.document(db);
    let mut builder = DocumentSymbolsBuilder::default();

    let ast = get_ast(db, file);
    ast.iter().try_for_each(|node| {
        dispatch!(
            node.lower(),
            [
                InterfaceDeclaration => build_document_symbols(&doc, ast, &mut builder),
                Error => build_document_symbols(&doc, ast, &mut builder),
                Typedef => build_document_symbols(&doc, ast, &mut builder),
                Method => build_document_symbols(&doc, ast, &mut builder)
            ]
        );
        anyhow::Ok(())
    })?;
    Ok(Some(DocumentSymbolResponse::Nested(builder.finalize())))
}

impl InterfaceDeclaration {
    pub(crate) fn build_document_symbols(
        &self,
        doc: &Document,
        ast: &ParsedAst,
        builder: &mut DocumentSymbolsBuilder,
    ) -> anyhow::Result<()> {
        let name = self.name.cast(ast);
        builder.push_symbol(lsp_types::DocumentSymbol {
            name: name.get_text(doc.as_bytes())?.to_string(),
            kind: lsp_types::SymbolKind::NAMESPACE,
            range: self.get_lsp_range(),
            selection_range: name.get_lsp_range(),
            tags: None,
            detail: None,
            deprecated: None,
            children: None,
        });
        Ok(())
    }
}

impl Error {
    pub(crate) fn build_document_symbols(
        &self,
        doc: &Document,
        ast: &ParsedAst,
        builder: &mut DocumentSymbolsBuilder,
    ) -> anyhow::Result<()> {
        let name = self.name.cast(ast);
        builder.push_symbol(lsp_types::DocumentSymbol {
            name: name.get_text(doc.as_bytes())?.to_string(),
            kind: lsp_types::SymbolKind::EVENT,
            range: self.get_lsp_range(),
            selection_range: name.get_lsp_range(),
            tags: None,
            detail: None,
            deprecated: None,
            children: None,
        });
        Ok(())
    }
}

impl Method {
    pub(crate) fn build_document_symbols(
        &self,
        doc: &Document,
        ast: &ParsedAst,
        builder: &mut DocumentSymbolsBuilder,
    ) -> anyhow::Result<()> {
        let name = self.name.cast(ast);
        builder.push_symbol(lsp_types::DocumentSymbol {
            name: name.get_text(doc.as_bytes())?.to_string(),
            kind: lsp_types::SymbolKind::METHOD,
            range: self.get_lsp_range(),
            selection_range: name.get_lsp_range(),
            tags: None,
            detail: None,
            deprecated: None,
            children: None,
        });
        Ok(())
    }
}

impl Typedef {
    pub(crate) fn build_document_symbols(
        &self,
        doc: &Document,
        ast: &ParsedAst,
        builder: &mut DocumentSymbolsBuilder,
    ) -> anyhow::Result<()> {
        let name = self.name.cast(ast);
        builder.push_symbol(lsp_types::DocumentSymbol {
            name: name.get_text(doc.as_bytes())?.to_string(),
            kind: lsp_types::SymbolKind::CLASS,
            range: self.get_lsp_range(),
            selection_range: name.get_lsp_range(),
            tags: None,
            detail: None,
            deprecated: None,
            children: None,
        });
        Ok(())
    }
}
