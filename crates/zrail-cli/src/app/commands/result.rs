//! Command output and process status.

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandResult {
    pub(crate) text: String,
    pub(crate) exit_code: i32,
}

impl CommandResult {
    pub(crate) fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            exit_code: 0,
        }
    }

    pub(crate) fn status(text: impl Into<String>, exit_code: i32) -> Self {
        Self {
            text: text.into(),
            exit_code,
        }
    }
}
