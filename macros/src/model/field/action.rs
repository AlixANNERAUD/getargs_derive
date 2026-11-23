use super::RawField;
use crate::model::attr::{require_value, set_once};
use quote::ToTokens;
use syn::meta::ParseNestedMeta;

const SUPPORTED_ACTIONS: &str = "`SetTrue`, `SetFalse`, `Count`, `Set`";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlagAction {
    SetTrue,
    SetFalse,
    Count,
}

/// The `action` key: one of the flag actions, or `Set`, which makes a `bool` field an option
/// taking `true` or `false` instead of a flag.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Action {
    Flag(FlagAction),
    Set,
}

impl Action {
    pub(super) fn name(self) -> &'static str {
        match self {
            Action::Flag(FlagAction::SetTrue) => "SetTrue",
            Action::Flag(FlagAction::SetFalse) => "SetFalse",
            Action::Flag(FlagAction::Count) => "Count",
            Action::Set => "Set",
        }
    }
}

fn parse_action(meta: &ParseNestedMeta) -> syn::Result<(Action, syn::Path)> {
    let path: syn::Path = require_value(meta, "action", "action = Count")?.parse()?;
    let plain = path.leading_colon.is_none()
        && path
            .segments
            .iter()
            .all(|segment| segment.arguments.is_none());
    let segments: Vec<String> = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    let name = match segments.as_slice() {
        [name] if plain => name.as_str(),
        [prefix, name] if plain && prefix == "ArgAction" => name.as_str(),
        _ => "",
    };
    let action = match name {
        "SetTrue" => Action::Flag(FlagAction::SetTrue),
        "SetFalse" => Action::Flag(FlagAction::SetFalse),
        "Count" => Action::Flag(FlagAction::Count),
        "Set" => Action::Set,
        _ => {
            let text = path.to_token_stream().to_string().replace(' ', "");
            return Err(syn::Error::new_spanned(
                &path,
                format!("unsupported action `{text}`; supported actions are: {SUPPORTED_ACTIONS}"),
            ));
        }
    };
    Ok((action, path))
}

pub(super) fn action(meta: &ParseNestedMeta, raw: &mut RawField) -> syn::Result<()> {
    let value = parse_action(meta)?;
    set_once(&mut raw.action, value, meta)
}
