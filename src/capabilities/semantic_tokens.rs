use auto_lsp::{
    anyhow,
    core::{ast::AstNode, dispatch, semantic_tokens_builder::SemanticTokensBuilder},
    default::db::{
        BaseDatabase,
        file::File,
        tracked::{ParsedAst, get_ast},
    },
    define_semantic_token_types,
    lsp_types::{SemanticTokensParams, SemanticTokensResult},
};

use crate::ast::{
    Arrow, Bool, Comment, Enum, Error, Float, Int, InterfaceDeclaration, Method, Object,
    StructField, Typedef, Typeref,
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

    let mut builder = SemanticTokensBuilder::new("".into());

    let ast = get_ast(db, file);
    ast.iter().try_for_each(|node| {
        dispatch!(
            node.lower(),
            [
                Comment => build_semantic_tokens(db, file, ast, &mut builder),
                Method => build_semantic_tokens(db, file, ast, &mut builder),
                StructField =>build_semantic_tokens(db, file, ast, &mut builder),
                Enum => build_semantic_tokens(db, file, ast, &mut builder),
                Typedef => build_semantic_tokens(db, file, ast, &mut builder),
                Error => build_semantic_tokens(db, file, ast, &mut builder),
                Typeref => build_semantic_tokens(db, file, ast, &mut builder),
                Bool =>build_semantic_tokens(db, file, ast, &mut builder),
                Int => build_semantic_tokens(db, file, ast, &mut builder),
                Float => build_semantic_tokens(db, file, ast, &mut builder),
                crate::ast::String => build_semantic_tokens(db, file, ast, &mut builder),
                Object => build_semantic_tokens(db, file, ast, &mut builder),
                InterfaceDeclaration => build_semantic_tokens(db, file, ast, &mut builder),
                Arrow => build_semantic_tokens(db, file, ast, &mut builder)
            ]
        );
        anyhow::Ok(())
    })?;
    Ok(Some(SemanticTokensResult::Tokens(builder.build())))
}

impl Comment {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        _ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        builder.push(
            self.get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == COMMENT).unwrap() as u32,
            0,
        );
        Ok(())
    }
}

impl Method {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        builder.push(
            self.keyword.cast(ast).get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == KEYWORD).unwrap() as u32,
            0,
        );
        builder.push(
            self.name.cast(ast).get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == METHOD).unwrap() as u32,
            0,
        );
        Ok(())
    }
}

impl StructField {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        Ok(builder.push(
            self.name.cast(ast).get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == PROPERTY).unwrap() as u32,
            0,
        ))
    }
}

impl Enum {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        Ok(self.member.iter().for_each(|m| {
            builder.push(
                m.cast(ast).get_lsp_range(),
                SUPPORTED_TYPES
                    .iter()
                    .position(|x| *x == ENUM_MEMBER)
                    .unwrap() as u32,
                0,
            )
        }))
    }
}

impl Typedef {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        builder.push(
            self.keyword.cast(ast).get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == KEYWORD).unwrap() as u32,
            0,
        );
        builder.push(
            self.name.cast(ast).get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == TYPE).unwrap() as u32,
            0,
        );
        Ok(())
    }
}

impl Error {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        builder.push(
            self.keyword.cast(ast).get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == KEYWORD).unwrap() as u32,
            0,
        );
        builder.push(
            self.name.cast(ast).get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == EVENT).unwrap() as u32,
            0,
        );
        Ok(())
    }
}

impl Typeref {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        _ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        builder.push(
            self.get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == TYPE).unwrap() as u32,
            0,
        );
        Ok(())
    }
}

impl Bool {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        _ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        builder.push(
            self.get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == TYPE).unwrap() as u32,
            0,
        );
        Ok(())
    }
}

impl Int {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        _ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        builder.push(
            self.get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == TYPE).unwrap() as u32,
            0,
        );
        Ok(())
    }
}

impl Float {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        _ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        builder.push(
            self.get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == TYPE).unwrap() as u32,
            0,
        );
        Ok(())
    }
}

impl crate::ast::String {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        _ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        builder.push(
            self.get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == TYPE).unwrap() as u32,
            0,
        );
        Ok(())
    }
}

impl Object {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        _ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        builder.push(
            self.get_lsp_range(),
            SUPPORTED_TYPES.iter().position(|x| *x == TYPE).unwrap() as u32,
            0,
        );
        Ok(())
    }
}

impl InterfaceDeclaration {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        builder.push(
            self.keyword.cast(ast).get_lsp_range(),
            SUPPORTED_TYPES
                .iter()
                .position(|x| *x == INTERFACE)
                .unwrap() as u32,
            0,
        );
        builder.push(
            self.name.cast(ast).get_lsp_range(),
            SUPPORTED_TYPES
                .iter()
                .position(|x| *x == NAMESPACE)
                .unwrap() as u32,
            0,
        );
        Ok(())
    }
}

impl Arrow {
    fn build_semantic_tokens(
        &self,
        _db: &impl BaseDatabase,
        _file: File,
        _ast: &ParsedAst,
        builder: &mut SemanticTokensBuilder,
    ) -> anyhow::Result<()> {
        builder.push(
            self.get_lsp_range(),
            SUPPORTED_TYPES
                .iter()
                .position(|x| *x == DECORATOR)
                .unwrap() as u32,
            0,
        );
        Ok(())
    }
}
