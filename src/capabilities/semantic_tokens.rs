use auto_lsp::{
    anyhow,
    core::{ast::AstNode, dispatch_once, document::Document, semantic_tokens_builder::SemanticTokensBuilder},
    default::db::{BaseDatabase, tracked::get_ast},
    define_semantic_token_types,
    lsp_types::{SemanticTokenType, SemanticTokensParams, SemanticTokensResult},
};

use crate::{
    ast::{
        Any, Arrow, Bool, Comment, EnumMemberName, ErrorName, Float, Int, InterfaceName, KeywordError, KeywordInterface, KeywordMethod, KeywordType, MethodName, Object, String, StructFieldName, TypedefName, Typeref
    },
    util::{get_token_index, get_file_from_db}
};

define_semantic_token_types![
    standard {
        COMMENT,
        DECORATOR,
        ENUM_MEMBER,
        EVENT,
        INTERFACE,
        KEYWORD,
        METHOD,
        NAMESPACE,
        PROPERTY,
        TYPE,
    }

    custom {}
];

pub fn semantic_tokens_full(
    db: &impl BaseDatabase,
    params: SemanticTokensParams,
) -> anyhow::Result<Option<SemanticTokensResult>> {
    let file = get_file_from_db(&params.text_document.uri, db)?;
    let ast = get_ast(db, file);
    let document = file.document(db);

    let mut builder = SemanticTokensBuilder::new("".into());

    ast.iter().for_each(|node| {
        dispatch_once!(
            node.lower(),
            [
                Any => push_semantic_token(&mut builder, document, TYPE),
                Arrow => push_semantic_token(&mut builder, document, DECORATOR),
                Bool => push_semantic_token(&mut builder, document, TYPE),
                Comment => push_semantic_token(&mut builder, document, COMMENT),
                EnumMemberName => push_semantic_token(&mut builder, document, ENUM_MEMBER),
                ErrorName => push_semantic_token(&mut builder, document, EVENT),
                Float => push_semantic_token(&mut builder, document, TYPE),
                Int => push_semantic_token(&mut builder, document, TYPE),
                InterfaceName => push_semantic_token(&mut builder, document, NAMESPACE),
                KeywordError => push_semantic_token(&mut builder, document, KEYWORD),
                KeywordInterface => push_semantic_token(&mut builder, document, INTERFACE),
                KeywordMethod => push_semantic_token(&mut builder, document, KEYWORD),
                KeywordType => push_semantic_token(&mut builder, document, KEYWORD),
                MethodName => push_semantic_token(&mut builder, document, METHOD),
                Object => push_semantic_token(&mut builder, document, TYPE),
                String => push_semantic_token(&mut builder, document, TYPE),
                StructFieldName => push_semantic_token(&mut builder, document, PROPERTY),
                TypedefName => push_semantic_token(&mut builder, document, TYPE),
                Typeref => push_semantic_token(&mut builder, document, TYPE)
            ]
        );
    });

    Ok(Some(SemanticTokensResult::Tokens(builder.build())))
}

trait SemanticToken {
    fn push_semantic_token(&self, builder: &mut SemanticTokensBuilder, document: &Document, type_: SemanticTokenType);
}
impl<T: AstNode> SemanticToken for T {
    fn push_semantic_token(&self, builder: &mut SemanticTokensBuilder, document: &Document, type_: SemanticTokenType) {
        builder.push(self.get_lsp_range(document).unwrap(), get_token_index(type_), 0);
    }
}
