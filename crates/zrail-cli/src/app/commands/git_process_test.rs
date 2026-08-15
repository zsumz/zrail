//! Git subprocess output remains memory-bounded.

use std::io::Cursor;

use super::{drain_bounded, read_bounded};

#[test]
fn bounded_reads_stop_after_the_first_excess_byte() {
    let captured = read_bounded(Cursor::new(vec![b'x'; 32]), 16).expect("read output");

    assert!(captured.overflowed);
    assert_eq!(captured.bytes.len(), 17);
}

#[test]
fn bounded_drains_discard_excess_bytes() {
    let captured = drain_bounded(Cursor::new(vec![b'x'; 32]), 16).expect("drain output");

    assert!(captured.overflowed);
    assert_eq!(captured.bytes.len(), 16);
}
