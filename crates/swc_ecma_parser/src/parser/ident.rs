//! 12.1 Identifiers
use either::Either;
use swc_atoms::atom;
use swc_ecma_lexer::token::{IdentLike, Keyword};

use super::*;

impl<I: Tokens> Parser<I> {
    pub(super) fn parse_maybe_private_name(&mut self) -> PResult<Either<PrivateName, IdentName>> {
        let is_private = is!(self, '#');

        if is_private {
            self.parse_private_name().map(Either::Left)
        } else {
            self.parse_ident_name().map(Either::Right)
        }
    }

    pub(super) fn parse_private_name(&mut self) -> PResult<PrivateName> {
        let start = cur_pos!(self);
        assert_and_bump!(self, '#');

        let hash_end = self.input.prev_span().hi;
        if self.input.cur_pos() - hash_end != BytePos(0) {
            syntax_error!(
                self,
                span!(self, start),
                SyntaxError::SpaceBetweenHashAndIdent
            );
        }

        let id = self.parse_ident_name()?;
        Ok(PrivateName {
            span: span!(self, start),
            name: id.sym,
        })
    }

    /// IdentifierReference
    pub(super) fn parse_ident_ref(&mut self) -> PResult<Ident> {
        let ctx = self.ctx();

        self.parse_ident(
            !ctx.contains(Context::InGenerator),
            !ctx.contains(Context::InAsync),
        )
    }

    /// LabelIdentifier
    pub(super) fn parse_label_ident(&mut self) -> PResult<Ident> {
        let ctx = self.ctx();

        self.parse_ident(
            !ctx.contains(Context::InGenerator),
            !ctx.contains(Context::InAsync),
        )
    }

    /// Use this when spec says "IdentifierName".
    /// This allows idents like `catch`.
    pub(super) fn parse_ident_name(&mut self) -> PResult<IdentName> {
        let in_type = self.ctx().contains(Context::InType);

        let start = cur_pos!(self);

        let w = match cur!(self, true) {
            Token::Word(..) => match bump!(self) {
                Token::Word(w) => w.into(),
                _ => unreachable!(),
            },

            Token::JSXName { .. } if in_type => match bump!(self) {
                Token::JSXName { name } => name,
                _ => unreachable!(),
            },

            _ => syntax_error!(self, SyntaxError::ExpectedIdent),
        };

        Ok(IdentName::new(w, span!(self, start)))
    }

    // https://tc39.es/ecma262/#prod-ModuleExportName
    pub(super) fn parse_module_export_name(&mut self) -> PResult<ModuleExportName> {
        let module_export_name = match cur!(self, false) {
            Ok(&Token::Str { .. }) => match self.parse_lit()? {
                Lit::Str(str_lit) => ModuleExportName::Str(str_lit),
                _ => unreachable!(),
            },
            Ok(&Token::Word(..)) => ModuleExportName::Ident(self.parse_ident_name()?.into()),
            _ => {
                unexpected!(self, "identifier or string");
            }
        };
        Ok(module_export_name)
    }

    /// Identifier
    ///
    /// In strict mode, "yield" is SyntaxError if matched.
    pub(super) fn parse_ident(&mut self, incl_yield: bool, incl_await: bool) -> PResult<Ident> {
        trace_cur!(self, parse_ident);

        let start = cur_pos!(self);

        let word = self.parse_with(|p| {
            let w = match cur!(p, true) {
                &Token::Word(..) => match bump!(p) {
                    Token::Word(w) => w,
                    _ => unreachable!(),
                },
                _ => syntax_error!(p, SyntaxError::ExpectedIdent),
            };

            // Spec:
            // It is a Syntax Error if this phrase is contained in strict mode code and the
            // StringValue of IdentifierName is: "implements", "interface", "let",
            // "package", "private", "protected", "public", "static", or "yield".
            match w {
                Word::Ident(ref name @ ident_like!("enum")) => {
                    p.emit_err(
                        p.input.prev_span(),
                        SyntaxError::InvalidIdentInStrict(name.clone().into()),
                    );
                }
                Word::Keyword(name @ Keyword::Yield) | Word::Keyword(name @ Keyword::Let) => {
                    p.emit_strict_mode_err(
                        p.input.prev_span(),
                        SyntaxError::InvalidIdentInStrict(name.into_atom()),
                    );
                }

                Word::Ident(
                    ref name @ ident_like!("static")
                    | ref name @ ident_like!("implements")
                    | ref name @ ident_like!("interface")
                    | ref name @ ident_like!("package")
                    | ref name @ ident_like!("private")
                    | ref name @ ident_like!("protected")
                    | ref name @ ident_like!("public"),
                ) => {
                    p.emit_strict_mode_err(
                        p.input.prev_span(),
                        SyntaxError::InvalidIdentInStrict(name.clone().into()),
                    );
                }
                _ => {}
            }

            // Spec:
            // It is a Syntax Error if StringValue of IdentifierName is the same String
            // value as the StringValue of any ReservedWord except for yield or await.
            match w {
                Word::Keyword(Keyword::Await) if p.ctx().contains(Context::InDeclare) => Ok(atom!("await")),

                Word::Keyword(Keyword::Await) if p.ctx().contains(Context::InStaticBlock) => {
                    syntax_error!(p, p.input.prev_span(), SyntaxError::ExpectedIdent)
                }

                // It is a Syntax Error if the goal symbol of the syntactic grammar is Module
                // and the StringValue of IdentifierName is "await".
                Word::Keyword(Keyword::Await) if p.ctx().contains(Context::Module) | p.ctx().contains(Context::InAsync) => {
                    syntax_error!(p, p.input.prev_span(), SyntaxError::InvalidIdentInAsync)
                }
                Word::Keyword(Keyword::This) if p.input.syntax().typescript() => Ok(atom!("this")),
                Word::Keyword(Keyword::Let) => Ok(atom!("let")),
                Word::Ident(ident) => {
                    if p.ctx().contains(Context::InClassField)
                        && matches!(&ident, IdentLike::Other(arguments) if atom!("arguments").eq(arguments))
                    {
                        p.emit_err(p.input.prev_span(), SyntaxError::ArgumentsInClassField)
                    }
                    Ok(ident.into())
                }
                Word::Keyword(Keyword::Yield) if incl_yield => Ok(atom!("yield")),
                Word::Keyword(Keyword::Await) if incl_await => Ok(atom!("await")),
                Word::Keyword(..) | Word::Null | Word::True | Word::False => {
                    syntax_error!(p, p.input.prev_span(), SyntaxError::ExpectedIdent)
                }
            }
        })?;

        Ok(Ident::new_no_ctxt(word, span!(self, start)))
    }
}
