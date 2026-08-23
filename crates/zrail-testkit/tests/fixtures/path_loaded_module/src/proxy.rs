//! Module loaded through an exact path.

mod child;

#[cfg(test)]
mod proxy_test;

#[cfg(test)]
include!("included.rs");

pub(crate) const VALUE: u64 = child::VALUE;
