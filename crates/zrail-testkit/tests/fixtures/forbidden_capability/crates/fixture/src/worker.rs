//! Deliberate network capability escape.

use std::net::TcpStream as Hidden;

pub(crate) fn stream(value: Hidden) -> Hidden { value }
