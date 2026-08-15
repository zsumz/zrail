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

struct Harness;

#[cfg(test)]
fn compile_fixture() {
    mod function_support;
    include!("function.rs");
}

#[cfg(test)]
const FIXTURE: () = { include!("const.rs") };

#[cfg(test)]
impl Harness {
    fn inherited() {
        include!("impl.rs");
    }
}

impl Harness {
    #[cfg(test)]
    fn method() {
        include!("method.rs");
    }
}

#[cfg(test)]
mod file_context;
