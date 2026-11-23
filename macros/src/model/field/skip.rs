use super::RawField;
use crate::model::attr::set_once;
use proc_macro2::{Group, Span};
use syn::{Attribute, Expr, Token, meta::ParseNestedMeta, spanned::Spanned};

pub(super) fn skip(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let value = if meta.input.peek(Token![=]) {
        Some(meta.value()?.parse::<Expr>()?)
    } else if meta.input.peek(syn::token::Paren) {
        return Err(
            meta.error("`skip` expects no value or an expression, e.g. `skip` or `skip = 4`")
        );
    } else {
        None
    };
    set_once(&mut raw.skip, (value, meta.path.span()), meta)
}

/// Finds `skip` in an `#[arg(...)]` attribute that is not parsed as an argument, to explain
/// why it cannot be there instead of reporting the attribute as a whole.
pub(super) fn find_skip(attr: &Attribute) -> Option<Span> {
    let mut span = None;
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("skip") {
            span.get_or_insert(meta.path.span());
        }
        if meta.input.peek(Token![=]) {
            meta.value()?.parse::<Expr>()?;
        } else if meta.input.peek(syn::token::Paren) {
            meta.input.parse::<Group>()?;
        }
        Ok(())
    });
    span
}
