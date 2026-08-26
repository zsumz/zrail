//! Recursive exact field types retain every semantic component or become unsupported.

use syn::spanned::Spanned;
use zrail_core::SourceSpan;

use super::fact::source_span;

#[derive(Clone, Debug)]
pub(crate) enum TypeShapeFact {
    Path {
        written: String,
        span: SourceSpan,
        arguments: Vec<TypeArgumentFact>,
    },
    Tuple(Vec<TypeShapeFact>),
    Reference {
        lifetime: Option<String>,
        mutable: bool,
        element: Box<TypeShapeFact>,
    },
    Slice(Box<TypeShapeFact>),
    Array {
        element: Box<TypeShapeFact>,
        length: ConstShapeFact,
    },
    Pointer {
        mutable: bool,
        element: Box<TypeShapeFact>,
    },
    Never,
    Unsupported(String),
}

#[derive(Clone, Debug)]
pub(crate) enum TypeArgumentFact {
    Type(TypeShapeFact),
    Lifetime(String),
    Const(ConstShapeFact),
}

#[derive(Clone, Debug)]
pub(crate) enum ConstShapeFact {
    Literal(String),
    Path { written: String, span: SourceSpan },
}

impl TypeShapeFact {
    pub(crate) fn nominal_path_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Path { span, .. } => Some(*span),
            Self::Reference { element, .. } => element.nominal_path_span(),
            _ => None,
        }
    }
}

pub(crate) fn type_shape(ty: &syn::Type) -> TypeShapeFact {
    match ty {
        syn::Type::Path(path) if path.qself.is_none() => path_shape(&path.path),
        syn::Type::Tuple(tuple) => {
            TypeShapeFact::Tuple(tuple.elems.iter().map(type_shape).collect())
        }
        syn::Type::Reference(reference) => TypeShapeFact::Reference {
            lifetime: reference.lifetime.as_ref().map(ToString::to_string),
            mutable: reference.mutability.is_some(),
            element: Box::new(type_shape(&reference.elem)),
        },
        syn::Type::Slice(slice) => TypeShapeFact::Slice(Box::new(type_shape(&slice.elem))),
        syn::Type::Array(array) => match const_shape(&array.len) {
            Some(length) => TypeShapeFact::Array {
                element: Box::new(type_shape(&array.elem)),
                length,
            },
            None => unsupported("array length is not an exact literal or path"),
        },
        syn::Type::Ptr(pointer) => TypeShapeFact::Pointer {
            mutable: pointer.mutability.is_some(),
            element: Box::new(type_shape(&pointer.elem)),
        },
        syn::Type::Never(_) => TypeShapeFact::Never,
        syn::Type::Group(group) => type_shape(&group.elem),
        syn::Type::Paren(paren) => type_shape(&paren.elem),
        syn::Type::BareFn(_)
        | syn::Type::ImplTrait(_)
        | syn::Type::Infer(_)
        | syn::Type::Macro(_)
        | syn::Type::TraitObject(_)
        | syn::Type::Verbatim(_) => unsupported("field type form is not statically exact"),
        _ => unsupported("field type form is unsupported"),
    }
}

fn path_shape(path: &syn::Path) -> TypeShapeFact {
    let written = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    let mut arguments = Vec::new();
    for (index, segment) in path.segments.iter().enumerate() {
        match &segment.arguments {
            syn::PathArguments::None => {}
            syn::PathArguments::AngleBracketed(values) if index + 1 == path.segments.len() => {
                for value in &values.args {
                    let argument = match value {
                        syn::GenericArgument::Type(ty) => TypeArgumentFact::Type(type_shape(ty)),
                        syn::GenericArgument::Lifetime(lifetime) => {
                            TypeArgumentFact::Lifetime(lifetime.to_string())
                        }
                        syn::GenericArgument::Const(value) => {
                            let Some(value) = const_shape(value) else {
                                return unsupported(
                                    "generic const is not an exact literal or path",
                                );
                            };
                            TypeArgumentFact::Const(value)
                        }
                        syn::GenericArgument::AssocType(_)
                        | syn::GenericArgument::AssocConst(_)
                        | syn::GenericArgument::Constraint(_)
                        | _ => {
                            return unsupported(
                                "associated or constrained path arguments are unsupported",
                            );
                        }
                    };
                    arguments.push(argument);
                }
            }
            syn::PathArguments::AngleBracketed(_) | syn::PathArguments::Parenthesized(_) => {
                return unsupported("only final-segment angle-bracketed arguments are supported");
            }
        }
    }
    TypeShapeFact::Path {
        written,
        span: source_span(path.span()),
        arguments,
    }
}

fn const_shape(expression: &syn::Expr) -> Option<ConstShapeFact> {
    match expression {
        syn::Expr::Lit(literal) => literal_text(&literal.lit).map(ConstShapeFact::Literal),
        syn::Expr::Path(path) if path.qself.is_none() => Some(ConstShapeFact::Path {
            written: path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
            span: source_span(path.path.span()),
        }),
        syn::Expr::Group(group) => const_shape(&group.expr),
        syn::Expr::Paren(paren) => const_shape(&paren.expr),
        _ => None,
    }
}

fn literal_text(literal: &syn::Lit) -> Option<String> {
    match literal {
        syn::Lit::Int(value) => Some(value.to_string()),
        syn::Lit::Bool(value) => Some(value.value.to_string()),
        syn::Lit::Char(value) => Some(format!("{:?}", value.value())),
        syn::Lit::Byte(value) => Some(value.value().to_string()),
        _ => None,
    }
}

fn unsupported(reason: &str) -> TypeShapeFact {
    TypeShapeFact::Unsupported(reason.into())
}
