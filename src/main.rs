use auto_lsp::default::db::{BaseDatabase, BaseDb};
use auto_lsp::default::server::capabilities::semantic_tokens_provider;
use auto_lsp::default::server::file_events::{
    change_text_document, changed_watched_files, open_text_document,
};
use auto_lsp::default::server::workspace_init::WorkspaceInit;
use auto_lsp::lsp_server::{self, Connection};
use auto_lsp::lsp_types::notification::{
    Cancel, DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument,
    DidOpenTextDocument, DidSaveTextDocument, LogTrace, SetTrace,
};
use auto_lsp::lsp_types::request::{
    DocumentDiagnosticRequest, DocumentSymbolRequest, Formatting, GotoDefinition,
    SemanticTokensFullRequest,
};
use auto_lsp::lsp_types::{self, OneOf};
use auto_lsp::lsp_types::{DiagnosticOptions, DiagnosticServerCapabilities};
use auto_lsp::lsp_types::{
    ServerCapabilities, WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};
use auto_lsp::server::Session;
use auto_lsp::server::notification_registry::NotificationRegistry;
use auto_lsp::server::options::InitOptions;
use auto_lsp::server::request_registry::RequestRegistry;
use std::error::Error;
use std::panic::RefUnwindSafe;
use varlink_language_server::capabilities::diagnostics::diagnostics;
use varlink_language_server::capabilities::document_symbols::document_symbols;
use varlink_language_server::capabilities::formatting::formatting;
use varlink_language_server::capabilities::goto_definition::goto_definition;
use varlink_language_server::capabilities::semantic_tokens::{
    SUPPORTED_TYPES, semantic_tokens_full,
};

use varlink_language_server::ast::Interface;

auto_lsp::configure_parsers!(
    PARSERS,
    "varlink" => {
        language: tree_sitter_varlink::LANGUAGE,
        ast_root: Interface
    }
);

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let (connection, io_threads) = Connection::stdio();
    let db = BaseDb::default();

    let (mut session, params) = Session::create(
        InitOptions {
            parsers: &PARSERS,
            capabilities: ServerCapabilities {
                text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Kind(
                    lsp_types::TextDocumentSyncKind::INCREMENTAL,
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(false),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        ..Default::default()
                    },
                )),
                semantic_tokens_provider: semantic_tokens_provider(
                    false,
                    Some(SUPPORTED_TYPES),
                    None,
                ),
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(crate::OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: None,
        },
        connection,
        db,
    )?;

    let mut request_registry = RequestRegistry::<BaseDb>::default();
    let mut notification_registry = NotificationRegistry::<BaseDb>::default();

    let init_results = session.init_workspace(params)?;
    if !init_results.is_empty() {
        init_results.into_iter().for_each(|result| {
            if let Err(err) = result {
                eprintln!("{}", err);
            }
        });
    };

    session.main_loop(
        on_requests(&mut request_registry),
        on_notifications(&mut notification_registry),
    )?;
    io_threads.join()?;

    Ok(())
}

fn on_notifications<Db: BaseDatabase + Clone + RefUnwindSafe>(
    registry: &mut NotificationRegistry<Db>,
) -> &mut NotificationRegistry<Db> {
    registry
        .on_mut::<DidOpenTextDocument, _>(|s, p| Ok(open_text_document(s, p)?))
        .on_mut::<DidChangeTextDocument, _>(|s, p| Ok(change_text_document(s, p)?))
        .on_mut::<DidChangeWatchedFiles, _>(|s, p| Ok(changed_watched_files(s, p)?))
        .on_mut::<Cancel, _>(|s, p| {
            let id: lsp_server::RequestId = match p.id {
                lsp_types::NumberOrString::Number(id) => id.into(),
                lsp_types::NumberOrString::String(id) => id.into(),
            };
            if let Some(response) = s.req_queue.incoming.cancel(id) {
                s.connection.sender.send(response.into())?;
            }
            Ok(())
        })
        .on::<DidSaveTextDocument, _>(|_s, _p| Ok(()))
        .on::<DidCloseTextDocument, _>(|_s, _p| Ok(()))
        .on::<SetTrace, _>(|_s, _p| Ok(()))
        .on::<LogTrace, _>(|_s, _p| Ok(()))
}

fn on_requests<Db: BaseDatabase + Clone + RefUnwindSafe>(
    registry: &mut RequestRegistry<Db>,
) -> &mut RequestRegistry<Db> {
    registry
        .on::<DocumentDiagnosticRequest, _>(diagnostics)
        .on::<DocumentSymbolRequest, _>(document_symbols)
        .on::<SemanticTokensFullRequest, _>(semantic_tokens_full)
        .on::<GotoDefinition, _>(goto_definition)
        .on::<Formatting, _>(formatting)
}
