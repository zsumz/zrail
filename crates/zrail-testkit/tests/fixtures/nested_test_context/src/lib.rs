//! Nested test-only source edges.

#[cfg(test)]
mod tests {
    mod support;
    include!("included.rs");

    mod outer {
        mod inner {
            mod support;
        }
    }
}
