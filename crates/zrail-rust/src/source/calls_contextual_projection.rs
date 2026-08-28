//! Qualified paths may become generic only in an including lexical context.

use syn::{ExprPath, Type};

use super::super::operation_model::subject::WrittenOperationSubject;

pub(super) fn text(callee: &ExprPath) -> Option<String> {
    let qself = callee.qself.as_ref()?;
    if qself.position > 0 {
        return None;
    }
    let Type::Path(self_type) = qself.ty.as_ref() else {
        return None;
    };
    (self_type.qself.is_none() && self_type.path.segments.len() > 1)
        .then(|| WrittenOperationSubject::from_expression(callee).written())
}
