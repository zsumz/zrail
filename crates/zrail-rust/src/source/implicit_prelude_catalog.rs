//! Rust edition preludes map ordinary names to canonical library identities.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PreludeItemKind {
    Type,
    Value,
    TupleConstructor,
    UnitConstructor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PreludeEntry {
    pub(super) canonical: &'static str,
    pub(super) kind: PreludeItemKind,
}

pub(super) fn core(name: &str, edition: &str) -> Option<PreludeEntry> {
    let name = name.strip_prefix("r#").unwrap_or(name);
    let entry = match name {
        "Copy" => ty("core::marker::Copy"),
        "Send" => ty("core::marker::Send"),
        "Sized" => ty("core::marker::Sized"),
        "Sync" => ty("core::marker::Sync"),
        "Unpin" => ty("core::marker::Unpin"),
        "Drop" => ty("core::ops::Drop"),
        "Fn" => ty("core::ops::Fn"),
        "FnMut" => ty("core::ops::FnMut"),
        "FnOnce" => ty("core::ops::FnOnce"),
        "AsyncFn" => ty("core::ops::AsyncFn"),
        "AsyncFnMut" => ty("core::ops::AsyncFnMut"),
        "AsyncFnOnce" => ty("core::ops::AsyncFnOnce"),
        "drop" => value("core::mem::drop"),
        "align_of" => value("core::mem::align_of"),
        "align_of_val" => value("core::mem::align_of_val"),
        "size_of" => value("core::mem::size_of"),
        "size_of_val" => value("core::mem::size_of_val"),
        "Clone" => ty("core::clone::Clone"),
        "Eq" => ty("core::cmp::Eq"),
        "Ord" => ty("core::cmp::Ord"),
        "PartialEq" => ty("core::cmp::PartialEq"),
        "PartialOrd" => ty("core::cmp::PartialOrd"),
        "AsMut" => ty("core::convert::AsMut"),
        "AsRef" => ty("core::convert::AsRef"),
        "From" => ty("core::convert::From"),
        "Into" => ty("core::convert::Into"),
        "Default" => ty("core::default::Default"),
        "DoubleEndedIterator" => ty("core::iter::DoubleEndedIterator"),
        "ExactSizeIterator" => ty("core::iter::ExactSizeIterator"),
        "Extend" => ty("core::iter::Extend"),
        "IntoIterator" => ty("core::iter::IntoIterator"),
        "Iterator" => ty("core::iter::Iterator"),
        "Option" => ty("core::option::Option"),
        "Some" => tuple("core::option::Option::Some"),
        "None" => unit("core::option::Option::None"),
        "Result" => ty("core::result::Result"),
        "Ok" => tuple("core::result::Result::Ok"),
        "Err" => tuple("core::result::Result::Err"),
        "FromIterator" if edition_at_least_2021(edition) => ty("core::iter::FromIterator"),
        "TryFrom" if edition_at_least_2021(edition) => ty("core::convert::TryFrom"),
        "TryInto" if edition_at_least_2021(edition) => ty("core::convert::TryInto"),
        "Future" if edition == "2024" => ty("core::future::Future"),
        "IntoFuture" if edition == "2024" => ty("core::future::IntoFuture"),
        _ => return None,
    };
    Some(entry)
}

pub(super) fn std_only(name: &str) -> Option<PreludeEntry> {
    let entry = match name.strip_prefix("r#").unwrap_or(name) {
        "ToOwned" => ty("std::borrow::ToOwned"),
        "Box" => ty("std::boxed::Box"),
        "String" => ty("std::string::String"),
        "ToString" => ty("std::string::ToString"),
        "Vec" => ty("std::vec::Vec"),
        _ => return None,
    };
    Some(entry)
}

fn edition_at_least_2021(edition: &str) -> bool {
    matches!(edition.as_bytes(), b"2021" | b"2024")
}

const fn ty(canonical: &'static str) -> PreludeEntry {
    PreludeEntry {
        canonical,
        kind: PreludeItemKind::Type,
    }
}

const fn value(canonical: &'static str) -> PreludeEntry {
    PreludeEntry {
        canonical,
        kind: PreludeItemKind::Value,
    }
}

const fn tuple(canonical: &'static str) -> PreludeEntry {
    PreludeEntry {
        canonical,
        kind: PreludeItemKind::TupleConstructor,
    }
}

const fn unit(canonical: &'static str) -> PreludeEntry {
    PreludeEntry {
        canonical,
        kind: PreludeItemKind::UnitConstructor,
    }
}
