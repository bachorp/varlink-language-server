use auto_lsp::default::db::{BaseDatabase, BaseDb};
use auto_lsp::default::server::capabilities::{
    TEXT_DOCUMENT_SYNC, WORKSPACE_PROVIDER, semantic_tokens_provider,
};
use auto_lsp::default::server::file_events::{
    change_text_document, changed_watched_files, open_text_document,
};
use auto_lsp::default::server::workspace_init::WorkspaceInit;
use auto_lsp::lsp_server::{self, Connection};
use auto_lsp::lsp_types::ServerCapabilities;
use auto_lsp::lsp_types::notification::{
    Cancel, DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument,
    DidOpenTextDocument, DidSaveTextDocument, LogTrace, SetTrace,
};
use auto_lsp::lsp_types::request::{
    Completion, DocumentDiagnosticRequest, DocumentSymbolRequest, FoldingRangeRequest, Formatting,
    GotoDefinition, HoverRequest, PrepareRenameRequest, References, Rename, SelectionRangeRequest,
    SemanticTokensFullRequest, WorkspaceDiagnosticRequest, WorkspaceSymbolRequest,
};
use auto_lsp::lsp_types::{self, HoverProviderCapability, OneOf};
use auto_lsp::lsp_types::{DiagnosticOptions, DiagnosticServerCapabilities};
use auto_lsp::server::Session;
use auto_lsp::server::notification_registry::NotificationRegistry;
use auto_lsp::server::options::InitOptions;
use auto_lsp::server::request_registry::RequestRegistry;
use std::error::Error;
use std::panic::RefUnwindSafe;
use varlink_language_server::capabilities::completion::completion;
use varlink_language_server::capabilities::diagnostics::{diagnostics, workspace_diagnostics};
use varlink_language_server::capabilities::folding_range::folding_range;
use varlink_language_server::capabilities::formatting::formatting;
use varlink_language_server::capabilities::goto_definition::goto_definition;
use varlink_language_server::capabilities::hover::hover;
use varlink_language_server::capabilities::references::references;
use varlink_language_server::capabilities::rename::{prepare_rename, rename};
use varlink_language_server::capabilities::selection_range::selection_range;
use varlink_language_server::capabilities::semantic_tokens::{
    SUPPORTED_TYPES, semantic_tokens_full,
};
use varlink_language_server::capabilities::symbols::{document_symbols, workspace_symbols};

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
                text_document_sync: TEXT_DOCUMENT_SYNC.clone(),
                workspace: WORKSPACE_PROVIDER.clone(),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        workspace_diagnostics: true,
                        ..Default::default()
                    },
                )),
                semantic_tokens_provider: semantic_tokens_provider(
                    false,
                    Some(SUPPORTED_TYPES),
                    None,
                ),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(crate::OneOf::Left(true)),
                references_provider: Some(lsp_types::OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                folding_range_provider: Some(lsp_types::FoldingRangeProviderCapability::Simple(
                    true,
                )),
                rename_provider: Some(lsp_types::OneOf::Right(lsp_types::RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: Default::default(),
                })),
                selection_range_provider: Some(
                    lsp_types::SelectionRangeProviderCapability::Simple(true),
                ),
                completion_provider: Some(lsp_types::CompletionOptions {
                    ..Default::default()
                }),
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
    init_results.into_iter().for_each(|result| {
        if let Err(err) = result {
            eprintln!("{}", err);
        }
    });

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
        .on::<WorkspaceDiagnosticRequest, _>(workspace_diagnostics)
        .on::<DocumentSymbolRequest, _>(document_symbols)
        .on::<WorkspaceSymbolRequest, _>(workspace_symbols)
        .on::<SemanticTokensFullRequest, _>(semantic_tokens_full)
        .on::<GotoDefinition, _>(goto_definition)
        .on::<References, _>(references)
        .on::<HoverRequest, _>(hover)
        .on::<Formatting, _>(formatting)
        .on::<FoldingRangeRequest, _>(folding_range)
        .on::<Completion, _>(completion)
        .on::<PrepareRenameRequest, _>(prepare_rename)
        .on::<Rename, _>(rename)
        .on::<SelectionRangeRequest, _>(selection_range)
}
