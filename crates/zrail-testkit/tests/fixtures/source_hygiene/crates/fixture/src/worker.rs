//! Deliberate hygiene violations.

#[allow(dead_code)]
pub(crate) fn value(input: Option<u64>) -> u64 {
    let value = input.unwrap();
    if value == 0 { panic!("zero"); }
    value
}
