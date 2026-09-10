"""Independent expected mutation identities for the frozen retired-backend report."""

import hashlib

MODULES = "src/reactor/mod.rs"
BACKEND = "src/reactor/backend.rs"
CONSTRUCTION = "src/reactor/host/construction.rs"
NAMES = ["broker_set", "plaintext", "poller", "resource", "tcp", "timer", "tls"]


def matrix(baseline):
    result = {}

    def add(case, path=None, source=None, diagnostic=None, quantity=0):
        inputs = dict(baseline)
        if path:
            if source is None:
                del inputs[path]
            else:
                payload = source.encode() if isinstance(source, str) else source
                inputs[path] = hashlib.sha256(payload).hexdigest()
        result[case] = (inputs, [list(diagnostic)] if diagnostic else [], quantity)

    add("valid")
    for name in NAMES:
        for case, source, quantity in [
            ("external", f"mod {name};", 1), ("inline", f"pub mod {name} {{}}", 1),
            ("inactive", f"#[cfg(any())] mod {name};", 1),
            ("duplicate", f"mod {name}; mod {name};", 2),
            ("nested", f"mod child {{ mod {name}; }}", 0), ("raw", f"mod r#{name};", 0),
            ("opaque", f"macro_rules! hidden {{ () => {{ mod {name}; }}; }}", 0),
            ("comment", f"// mod {name};", 0),
        ]:
            error = ("rust:inventory:kd-retired-modules", "RUST-INVENTORY-001") if quantity else None
            add(f"module-{name}-{case}", MODULES, source, error, quantity)
        for case, suffix, rejected in [
            ("direct", "old.rs", True), ("nested", "a/b/old.rs", True),
            ("uppercase", "old.RS", False), ("extensionless", ".rs", False),
            ("nested-extensionless", "a/.rs", False), ("hidden", ".old.rs", True),
            ("text", "old.txt", False),
        ]:
            error = (f"repository:file:kd-retired-tree-{name}", "REP-FILE-001") if rejected else None
            add(f"tree-{name}-{case}", f"src/reactor/{name}/{suffix}", "// retired physical source\n", error)
        add(f"tree-{name}-empty-directory")
    for case, source, quantity in [
        ("unit", "enum ReactorBackend { Legacy }", 1),
        ("tuple", "enum ReactorBackend { Legacy(u8) }", 1),
        ("named", "enum ReactorBackend { Legacy { x: u8 } }", 1),
        ("inactive", "enum ReactorBackend { #[cfg(any())] Legacy }", 1),
        ("duplicate", "enum ReactorBackend { Legacy, Legacy }", 2),
        ("different-enum", "enum Other { Legacy }", 0),
        ("nested", "mod child { enum ReactorBackend { Legacy } }", 0),
        ("raw", "enum ReactorBackend { r#Legacy }", 0), ("absent-enum", "", 0),
        ("comment", "// enum ReactorBackend { Legacy }", 0),
    ]:
        error = ("rust:inventory:kd-retired-backend", "RUST-INVENTORY-001") if quantity else None
        add(f"backend-{case}", BACKEND, source, error, quantity)
    for token in ["new_legacy", "LegacyBackend"]:
        for case, source, rejected in [
            ("comment", f"// {token}", True), ("string", f'const TEXT: &str = "{token}";', True),
            ("inactive", f"#[cfg(any())] fn {token}() {{}}", True),
            ("substring", f"// prefix{token}suffix", True),
            ("different-case", f"// {token.upper()}", False),
        ]:
            error = (f"repository:file:kd-retired-construction-{token}", "REP-FILE-004") if rejected else None
            add(f"construction-{token}-{case}", CONSTRUCTION, source, error)
    for name, path in [("modules", MODULES), ("backend", BACKEND), ("construction", CONSTRUCTION)]:
        add(f"input-{name}-missing", path, None, ("repository:file:kd-retired-inputs", "REP-FILE-002"))
        error = (("repository:files", "REP-FILE-006") if path == CONSTRUCTION
                 else (f"rust:inventory:kd-retired-{name}", "RUST-INVENTORY-002"))
        add(f"input-{name}-utf8", path, b"\xff", error)
        if path != CONSTRUCTION:
            for case, source in [("malformed", "fn broken("), ("fragment", "1 + 2")]:
                add(f"input-{name}-{case}", path, source, error)
    add("wrong-file-decoy", "src/unrelated.rs", "mod tls; enum ReactorBackend { Legacy } // LegacyBackend new_legacy")
    assert len(result) == 144
    return result
