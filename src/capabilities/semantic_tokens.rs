use auto_lsp::{
    anyhow,
    core::{ast::AstNode, dispatch_once, semantic_tokens_builder::SemanticTokensBuilder},
    default::db::{BaseDatabase, tracked::get_ast},
    define_semantic_token_types,
    lsp_types::{SemanticTokenType, SemanticTokensParams, SemanticTokensResult},
};

use crate::{
    ast::{
        Any, Arrow, Bool, Comment, EnumMemberName, ErrorName, Float, Int, InterfaceName, KeywordError, KeywordInterface, KeywordMethod, KeywordType, MethodName, Object, String, StructFieldName, TypedefName, Typeref
    },
    capabilities::util::get_token_index,
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
    let file = crate::capabilities::util::get_file_from_db(&params.text_document.uri, db)?;
    let ast = get_ast(db, file);

    let mut builder = SemanticTokensBuilder::new("".into());

    ast.iter().for_each(|node| {
        dispatch_once!(
            node.lower(),
            [
                Any => push_semantic_token(&mut builder, TYPE),
                Arrow => push_semantic_token(&mut builder, DECORATOR),
                Bool => push_semantic_token(&mut builder, TYPE),
                Comment => push_semantic_token(&mut builder, COMMENT),
                EnumMemberName => push_semantic_token(&mut builder, ENUM_MEMBER),
                ErrorName => push_semantic_token(&mut builder, EVENT),
                Float => push_semantic_token(&mut builder, TYPE),
                Int => push_semantic_token(&mut builder, TYPE),
                InterfaceName => push_semantic_token(&mut builder, NAMESPACE),
                KeywordError => push_semantic_token(&mut builder, KEYWORD),
                KeywordInterface => push_semantic_token(&mut builder, INTERFACE),
                KeywordMethod => push_semantic_token(&mut builder, KEYWORD),
                KeywordType => push_semantic_token(&mut builder, KEYWORD),
                MethodName => push_semantic_token(&mut builder, METHOD),
                Object => push_semantic_token(&mut builder, TYPE),
                String => push_semantic_token(&mut builder, TYPE),
                StructFieldName => push_semantic_token(&mut builder, PROPERTY),
                TypedefName => push_semantic_token(&mut builder, TYPE),
                Typeref => push_semantic_token(&mut builder, TYPE)
            ]
        );
    });

    Ok(Some(SemanticTokensResult::Tokens(builder.build())))
}

trait SemanticToken {
    fn push_semantic_token(&self, builder: &mut SemanticTokensBuilder, type_: SemanticTokenType);
}
impl<T: AstNode> SemanticToken for T {
    fn push_semantic_token(&self, builder: &mut SemanticTokensBuilder, type_: SemanticTokenType) {
        builder.push(self.get_lsp_range(), get_token_index(type_), 0);
    }
}
