//! Deliberate inline test.

pub(crate) fn value() -> u64 { 5 }

#[cfg(test)]
mod tests {
    #[test]
    fn proof() { assert_eq!(super::value(), 5); }
}
