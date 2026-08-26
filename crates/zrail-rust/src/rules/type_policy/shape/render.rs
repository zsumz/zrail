//! Recursive canonical rendering for type and const shapes.

use zrail_core::PolicyReachability;

use crate::source::{
    ConstShapeFact, FactNamespace, RustFileFacts, TypeArgumentFact, TypeShapeFact,
};

use super::super::{RuleContext, identity};

pub(crate) fn render_source(
    shape: &TypeShapeFact,
    context: &RuleContext<'_>,
    file: &RustFileFacts,
    reachability: PolicyReachability,
) -> Result<String, String> {
    render(shape, &mut |written, span| {
        let resolved =
            identity::at_span(context, file, span, reachability, Some(FactNamespace::Type));
        if primitive(written) && resolved.exact.is_empty() && !resolved.unresolved {
            return Ok(written.into());
        }
        resolved.one().map(str::to_owned).map_err(str::to_owned)
    })
}

pub(crate) fn render_contract(source: &str) -> Result<String, String> {
    let parsed = syn::parse_str::<syn::Type>(source)
        .map_err(|error| format!("invalid Rust type syntax: {error}"))?;
    let shape = crate::source::type_shape(&parsed);
    render(&shape, &mut |written, _| {
        if primitive(written) || written.contains("::") {
            Ok(identity::normalize(written).into())
        } else {
            Err(format!("unqualified non-primitive path {written:?}"))
        }
    })
}

fn render(
    shape: &TypeShapeFact,
    resolve: &mut impl FnMut(&str, zrail_core::SourceSpan) -> Result<String, String>,
) -> Result<String, String> {
    match shape {
        TypeShapeFact::Path {
            written,
            span,
            arguments,
        } => {
            let mut result = resolve(written, *span)?;
            if !arguments.is_empty() {
                result.push('<');
                result.push_str(
                    &arguments
                        .iter()
                        .map(|argument| render_argument(argument, resolve))
                        .collect::<Result<Vec<_>, _>>()?
                        .join(","),
                );
                result.push('>');
            }
            Ok(result)
        }
        TypeShapeFact::Tuple(values) => {
            let values = values
                .iter()
                .map(|value| render(value, resolve))
                .collect::<Result<Vec<_>, _>>()?;
            let suffix = if values.len() == 1 { "," } else { "" };
            Ok(format!("({}{suffix})", values.join(",")))
        }
        TypeShapeFact::Reference {
            lifetime,
            mutable,
            element,
        } => Ok(format!(
            "&{}{}{}",
            lifetime
                .as_deref()
                .map_or_else(String::new, |value| format!("{value} ")),
            if *mutable { "mut " } else { "" },
            render(element, resolve)?
        )),
        TypeShapeFact::Slice(element) => Ok(format!("[{}]", render(element, resolve)?)),
        TypeShapeFact::Array { element, length } => Ok(format!(
            "[{};{}]",
            render(element, resolve)?,
            render_const(length, resolve)?
        )),
        TypeShapeFact::Pointer { mutable, element } => Ok(format!(
            "*{} {}",
            if *mutable { "mut" } else { "const" },
            render(element, resolve)?
        )),
        TypeShapeFact::Never => Ok("!".into()),
        TypeShapeFact::Unsupported(reason) => Err(reason.clone()),
    }
}

fn render_argument(
    argument: &TypeArgumentFact,
    resolve: &mut impl FnMut(&str, zrail_core::SourceSpan) -> Result<String, String>,
) -> Result<String, String> {
    match argument {
        TypeArgumentFact::Type(value) => render(value, resolve),
        TypeArgumentFact::Lifetime(value) => Ok(value.clone()),
        TypeArgumentFact::Const(value) => render_const(value, resolve),
    }
}

fn render_const(
    value: &ConstShapeFact,
    resolve: &mut impl FnMut(&str, zrail_core::SourceSpan) -> Result<String, String>,
) -> Result<String, String> {
    match value {
        ConstShapeFact::Literal(value) => Ok(value.clone()),
        ConstShapeFact::Path { written, span } => resolve(written, *span),
    }
}

fn primitive(value: &str) -> bool {
    matches!(
        value,
        "bool"
            | "char"
            | "str"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "f32"
            | "f64"
    )
}
