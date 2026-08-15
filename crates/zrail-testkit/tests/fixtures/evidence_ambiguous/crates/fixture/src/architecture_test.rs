//! Duplicate simple names cannot masquerade as exact evidence.

mod first {
    #[test]
    fn qualification_graph_is_live() {}
}

mod second {
    #[test]
    fn qualification_graph_is_live() {}
}
