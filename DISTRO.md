# nitrodb — distribution notes

- **Identity:** The fast, lightweight distribution of NEDB: append-only op-log lineage, minimal footprint, built for speed.
- **Relationship to nedb-engine:** identical core today; renamed for npm/PyPI/crates so it publishes as `nitrodb`.
- **Planned divergence:** per-distro *defaults* (no flags required) land in `rust/crates/nitrodb/src/lib.rs` and the Python/JS shims.
- **Builds:** driven by the central `nedb` release workflow via submodule; this repo carries no workflow of its own.
