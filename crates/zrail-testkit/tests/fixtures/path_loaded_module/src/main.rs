//! Path-loaded module fixture.

#[path = "proxy.rs"]
mod proxy;

fn main() {
    let _ = proxy::VALUE;
}
