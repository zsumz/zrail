//! Byte-exact original route-source assertions, with reviewed text inputs injected.

pub(super) fn check(route_sources: &[String], facade: &str) {
    for forbidden in ["ConnectionSet", "Poller", "SingleBroker", "BrokerSet"] {
        assert!(
            route_sources
                .iter()
                .all(|source| !source.contains(forbidden)),
            "route modules acquired forbidden capability {forbidden}"
        );
    }
    assert_eq!(facade.matches("connections: DirectSetOwner<T>").count(), 1);
}
