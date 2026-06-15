use auto_lsp::{
    anyhow::{self, Ok},
    core::{ast::AstNode, document::Document},
    default::db::{
        BaseDatabase,
        tracked::{ParsedAst, get_ast},
    },
    lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind, Position, Range},
    texter::core::text::Text,
};

use crate::{
    ast::{
        self, Any, Bool, EnumMemberName, Error, ErrorName, Float, Int, InterfaceDeclaration,
        InterfaceName, KeywordError, KeywordInterface, KeywordMethod, KeywordType, Method,
        MethodName, Object, StructField, StructFieldName, Typedef, TypedefName, Typeref,
    },
    util::{get_file_from_db, leaf_at, walk_up},
};

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}

// It would be great to utilize partial formatting for this but this is hard to achieve with Topiary
fn fix_indent(raw: &str, level: usize) -> String {
    if raw.lines().skip(1).all(|line| {
        line.get(..level)
            .map(|prefix| prefix.chars().all(is_whitespace))
            .unwrap_or(false)
    }) {
        raw.lines()
            .enumerate()
            .map(|(i, line)| if i == 0 { line } else { &line[level..] })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        " ".repeat(level) + raw
    }
}

fn get_doc(text: &Text, pos: Position) -> anyhow::Result<String> {
    let c = pos.character as usize;
    if !text
        .get_row(pos.line as usize)
        .ok_or_else(|| {
            anyhow::format_err!("position {}:{} is out of range", pos.line, pos.character)
        })?
        .chars()
        .take(c)
        .all(is_whitespace)
    {
        return Ok("".into());
    }

    let comments_rev: Vec<&str> = (0..pos.line)
        .rev()
        .map(|l| text.get_row(l as usize).unwrap())
        .take_while(|line| {
            line.chars().take(c).all(is_whitespace) && line.chars().nth(c) == Some('#')
        })
        .map(|line| &line[c..])
        .collect();

    if comments_rev.is_empty() {
        return Ok("".into());
    }

    let mut value = "\n\n---\n".to_string();
    for line in comments_rev.iter().rev() {
        // TODO: Use `trim_prefix` when available (https://github.com/rust-lang/rust/issues/142312)
        value.push_str(
            line.trim_start_matches("#")
                .strip_prefix(" ")
                .unwrap_or(line),
        );
        value.push('\n');
    }

    Ok(value)
}

#[derive(Clone, Copy, Debug, Default)]
struct MkHoverOptions {
    doc: bool,
    bracket: bool,
}

fn mk_hover(
    target: &dyn AstNode,
    document: &Document,
    document_bytes: &[u8],
    text: &Text,
    options: MkHoverOptions,
) -> anyhow::Result<(Range, String)> {
    Ok((
        target.get_lsp_range(document).unwrap(),
        format!(
            "```varlink\n{}{}{}\n```{}",
            if options.bracket { "(" } else { "" },
            fix_indent(
                target.get_text(document_bytes).unwrap(),
                target.get_start_position().character as usize
            ),
            if options.bracket { ")" } else { "" },
            if options.doc {
                get_doc(text, target.get_start_position())?
            } else {
                "".into()
            }
        ),
    ))
}

impl StructFieldName {
    fn get_hover<'a>(
        &'a self,
        ast: &'a ParsedAst,
        document: &Document,
        document_bytes: &'a [u8],
        text: &Text,
    ) -> anyhow::Result<(Range, String)> {
        mk_hover(
            walk_up::<StructField>(ast, self).unwrap(),
            document,
            document_bytes,
            text,
            MkHoverOptions {
                doc: true,
                bracket: true,
                ..Default::default()
            },
        )
    }
}

impl EnumMemberName {
    fn get_hover<'a>(
        &'a self,
        _nodes: &'a [Box<dyn AstNode>],
        document: &Document,
        document_bytes: &'a [u8],
        text: &Text,
    ) -> anyhow::Result<(Range, String)> {
        mk_hover(
            self,
            document,
            document_bytes,
            text,
            MkHoverOptions {
                doc: true,
                bracket: true,
                ..Default::default()
            },
        )
    }
}

impl InterfaceName {
    fn get_hover<'a>(
        &'a self,
        ast: &'a ParsedAst,
        document: &Document,
        document_bytes: &'a [u8],
        text: &Text,
    ) -> anyhow::Result<(Range, String)> {
        mk_hover(
            walk_up::<InterfaceDeclaration>(ast, self).unwrap(),
            document,
            document_bytes,
            text,
            MkHoverOptions {
                doc: true,
                ..Default::default()
            },
        )
    }
}

impl TypedefName {
    fn get_hover<'a>(
        &'a self,
        ast: &'a ParsedAst,
        document: &Document,
        document_bytes: &'a [u8],
        text: &Text,
    ) -> anyhow::Result<(Range, String)> {
        mk_hover(
            walk_up::<Typedef>(ast, self).unwrap(),
            document,
            document_bytes,
            text,
            MkHoverOptions {
                doc: true,
                ..Default::default()
            },
        )
    }
}

impl ErrorName {
    fn get_hover<'a>(
        &'a self,
        ast: &'a ParsedAst,
        document: &Document,
        document_bytes: &'a [u8],
        text: &Text,
    ) -> anyhow::Result<(Range, String)> {
        mk_hover(
            walk_up::<Error>(ast, self).unwrap(),
            document,
            document_bytes,
            text,
            MkHoverOptions {
                doc: true,
                ..Default::default()
            },
        )
    }
}

impl MethodName {
    fn get_hover<'a>(
        &'a self,
        ast: &'a ParsedAst,
        document: &Document,
        document_bytes: &'a [u8],
        text: &Text,
    ) -> anyhow::Result<(Range, String)> {
        mk_hover(
            walk_up::<Method>(ast, self).unwrap(),
            document,
            document_bytes,
            text,
            MkHoverOptions {
                doc: true,
                ..Default::default()
            },
        )
    }
}

macro_rules! implement_hover {
    ($ty:ty) => {
        impl $ty {
            fn get_hover<'a>(
                &'a self,
                _ast: &'a ParsedAst,
                document: &Document,
                document_bytes: &'a [u8],
                text: &Text,
            ) -> anyhow::Result<(Range, String)> {
                mk_hover(
                    self,
                    document,
                    document_bytes,
                    text,
                    MkHoverOptions::default(),
                )
            }
        }
    };
}

implement_hover!(Bool);
implement_hover!(Int);
implement_hover!(Float);
implement_hover!(ast::String);
implement_hover!(Object);
implement_hover!(Any);
implement_hover!(KeywordInterface);
implement_hover!(KeywordError);
implement_hover!(KeywordMethod);
implement_hover!(KeywordType);

impl Typeref {
    fn get_hover<'a>(
        &'a self,
        ast: &'a ParsedAst,
        document: &Document,
        document_bytes: &'a [u8],
        text: &Text,
    ) -> anyhow::Result<(Range, String)> {
        let name = self.get_text(document_bytes).unwrap();
        let def = {
            let mut def = None;
            for node in ast.iter() {
                if let Some(typedef) = node.lower().downcast_ref::<Typedef>()
                    && typedef.name.cast(ast).get_text(document_bytes).unwrap() == name
                {
                    if def.is_some() {
                        def = None;
                        break;
                    }

                    def = Some(typedef);
                }
            }

            def
        };

        if let Some(def) = def {
            def.name
                .cast(ast)
                .get_hover(ast, document, document_bytes, text)
        } else {
            Ok((
                self.get_lsp_range(document).unwrap(),
                format!(
                    "```varlink\n{}\n```",
                    self.get_text(document_bytes).unwrap()
                ),
            ))
        }
    }
}

pub fn hover(db: &impl BaseDatabase, params: HoverParams) -> anyhow::Result<Option<Hover>> {
    let file = get_file_from_db(&params.text_document_position_params.text_document.uri, db)?;
    let ast = get_ast(db, file);
    let document = file.document(db);
    let document_bytes = document.as_bytes();
    let text = &document.texter;

    let Some(leaf) = leaf_at(ast, document, params.text_document_position_params.position) else {
        return Ok(None);
    };
    let leaf = leaf.lower();

    let hover = {
        if let Some(x) = walk_up::<InterfaceName>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<TypedefName>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<ErrorName>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<MethodName>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<StructFieldName>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<EnumMemberName>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<Bool>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<Int>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<Float>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<ast::String>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<Object>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<Any>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<KeywordInterface>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<KeywordError>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<KeywordMethod>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<KeywordType>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else if let Some(x) = walk_up::<Typeref>(ast, leaf) {
            Some(x.get_hover(ast, document, document_bytes, text))
        } else {
            None
        }
    };

    Ok(hover.transpose()?.map(|(range, value)| Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        }),
        range: Some(range),
    }))
}
