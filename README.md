<div align="center">

# NEDB

**A versioned, self-compressing, time-traveling embedded database.**

Replay-protected · idempotent · relational · filterable · sortable · searchable · provable.
One Rust core → ships to **PyPI** and **npm** from a single source.

**[Website & docs → eth-interchained.github.io/nedb](https://eth-interchained.github.io/nedb/)**

</div>

---

## Why NEDB

Redis is fast because it's in-memory and simple — but relations are hand-rolled, history is gone the moment you overwrite, and every call pays a network hop. NEDB keeps the speed and adds the things real systems actually need:

- **Faster-than-Redis latency where it's honest to claim it** — NEDB runs **embedded, in-process**, so point reads pay *no socket hop*. The networked server (`nedbd`, RESP-compatible) competes on the Rust core's merits.
- **Replay protection + idempotency in the core, not the app.** Every write carries a strictly-monotonic per-client nonce and an optional idempotency key. Retries are no-ops; stale/out-of-order ops are rejected. This is built into one **hash-chained, append-only log**.
- **Time-travel.** Read the database *exactly as it existed* at any past sequence — `AS OF seq`. Debugging, audit, MVCC snapshots, and deterministic replay all fall out of the same log.
- **Durable persistence, Redis-style.** Point a database at a path and every op is appended to the hash-chained log on disk (and `fsync`'d); it reloads by replaying that log on open. It's exactly Redis's AOF model — except the append-only log is the *same tamper-evident chain* the engine already trusts, so `verify()` and `AS OF` hold across restarts and the log is never rewritten.
- **First-class relations.** Adjacency-list graph edges with O(1) traversal — *and the graph time-travels too*.
- **Filter / sort / search.** Equality, ordered, and full-text inverted indexes, maintained incrementally.
- **git-style files with maximum compression.** Content-defined chunking + content-addressed dedup + temperature tiers (fast warm codec, max-ratio cold archival). Every file version has a Merkle root you can **anchor on-chain**.

> **The keystone:** one nonce-enforced append-only log is the substrate for idempotency, replay protection, crash recovery, MVCC, *and* time-travel — simultaneously.

---

## Quickstart (Python reference engine — runs today, zero build)

```bash
git clone https://github.com/Eth-Interchained/nedb && cd nedb
pip install -e .                 # pure-Python reference; no toolchain needed
python3 examples/demo.py         # see every feature
python3 tests/test_nedb.py       # 11/11 invariants
```

```python
from nedb import NEDB

db = NEDB("./mydata")            # durable: append-only log on disk, reloads on open
# db = NEDB()                    # (no path = purely in-memory)
db.create_index("users", "status", "eq")
db.create_index("users", "age", "ordered")
db.create_index("users", "bio", "search")

db.put("users", "alice", {"name": "Alice", "age": 31, "status": "active",
                          "city": "Austin", "bio": "rust systems hacker"})

# Idempotent, replay-protected write (safe to retry forever):
db.put("orders", "o1", {"total": 42}, client="checkout", nonce=7, idem="charge-o1")

# NQL — filter + sort
db.query('FROM users WHERE age >= 25 AND status = "active" ORDER BY age DESC')

# Full-text search
db.query('FROM users SEARCH "rust"')

# Relations + graph traversal
db.link("users:alice", "follows", "users:bob")
db.q("users").where("_id", "=", "alice").traverse("follows").run()

# Time-travel
s = db.seq
db.put("users", "alice", {"name": "Alice", "city": "Lisbon", "age": 31, "status": "active"})
db.get("users", "alice", as_of=s)["city"]      # -> "Austin"

# git-style files with Cascade compression + provable history
v1 = db.put_file("notes.txt", open("notes.txt","rb").read())
db.file_root("notes.txt", v1)                  # Merkle root — anchorable on ITC

# Durable + provable across restarts
db.close()
db = NEDB("./mydata")                          # replays the log on open
assert db.verify()                             # the hash chain is intact
db.get("users", "alice", as_of=s)["city"]      # AS OF still works -> "Austin"
```

---

## Persistence

NEDB persists the way Redis does — by writing the operations, not by dumping pages — because the engine's whole thesis is that **state is a pure function of the log**.

- `NEDB(path)` opens a **durable** database in a directory. Every op is appended to `log.aof` (one JSON line) and `fsync`'d; index configuration is snapshotted to `meta.json`. On open, NEDB replays the log to rebuild state.
- `NEDB()` with no path is **in-memory** (unchanged).
- The append-only log is the **same hash-chained, tamper-evident chain** that powers idempotency, replay protection, and time-travel — so `verify()`, `AS OF`, relations, and the anchorable head all survive a restart. The log is **never rewritten**, so the chain (and its commitment) stays provable.

```python
db = NEDB("./mydata")
db.put("users", "alice", {"name": "Alice", "status": "active"})
db.close()                       # flush + fsync

again = NEDB("./mydata")         # replays log.aof
assert again.verify()            # chain intact across the restart
again.get("users", "alice")      # -> {"name": "Alice", ...}
```

> Snapshotting (an RDB-style fast-load checkpoint that keeps the AOF intact) and Rust-core parity are tracked on the roadmap.

---

## nedbd — run NEDB as a server

For client/server setups (multiple apps, a remote admin UI like NEDB Studio, or just keeping the database in its own process), `pip install nedb-engine` ships a daemon. It runs the engine as a long-lived process and serves an HTTP/JSON API; each named database is a durable `NEDB(path)` held open in memory. Connect to it the way you'd connect to Redis or Postgres — over a URL.

```bash
nedbd                       # http://127.0.0.1:7070, data in ./nedb-data
# config via env: NEDBD_HOST, NEDBD_PORT, NEDBD_DATA, NEDBD_TOKEN (optional bearer auth)
```

```bash
# create a database (optionally seeded with indexes / rows / links)
curl -X POST localhost:7070/v1/databases -d '{"name":"shop","init":{
  "indexes":[["users","status","eq"]],
  "seed":{"users":[{"id":"u1","name":"Ada","status":"active"}]}}}'

# query it (real NQL, real engine)
curl -X POST localhost:7070/v1/databases/shop/query -d '{"nql":"FROM users WHERE status = \"active\""}'

# write, verify, time-travel — all server-side on the durable log
curl -X POST localhost:7070/v1/databases/shop/put   -d '{"coll":"users","id":"u2","doc":{"name":"Bo"}}'
curl       localhost:7070/v1/databases/shop/verify
```

API: `GET /health` · `GET|POST /v1/databases` · `GET|DELETE /v1/databases/<name>` · `POST …/query` · `POST …/put` · `POST …/index` · `POST …/link` · `DELETE …/rows/<coll>/<id>` · `GET …/verify` · `GET …/log`. Databases persist across daemon restarts (the engine replays its append-only log on open).

---

## NQL — the NEDB Query Language

One small grammar; the Rust parser is the single source of truth so Python and Node share identical semantics. A fluent builder compiles to the same plan.

```
FROM <collection>
  [ AS OF <seq> ]
  [ WHERE <field> <op> <value> (AND ...)* ]      op ∈ = != < <= > >=
  [ SEARCH "<text>" ]
  [ ORDER BY <field> [ASC|DESC] ]
  [ TRAVERSE <relation> ]
  [ LIMIT <n> ]
```

---

## What's measured (reference engine, pure Python, 2 vCPU)

| Operation | Result |
|---|---|
| GET (embedded, in-process) | **~1.2M ops/s** (~800 ns/op) |
| SET (logged + indexed) | ~77K ops/s |
| Indexed query latency | ~75 µs |
| File compression — warm (zlib stand-in) | **39.9×** |
| File compression — cold (LZMA archival) | **88.9×** |
| Cross-version dedup | 20 of 22 chunks reused on edit |

The reference engine proves the **architecture**. The Rust core (`rust/`) is the speed target — see `bench/bench_redis.py` for the embedded-vs-Redis harness.

---

## Architecture

```
            ┌──────────────────────────────────────────────┐
  put/del → │  OpLog  (append-only · BLAKE3 hash chain ·    │ ← single source of truth
  link      │          per-client nonce · idempotency keys) │
            └───────────────┬──────────────────────────────┘
            deterministic fold │ (state = pure function of the log)
        ┌──────────────┬───────┴────────┬───────────────────┐
        ▼              ▼                ▼                   ▼
   MVCC store     Relations         Indexes            BlobStore (Cascade)
   (time-travel)  (graph, AS OF)    eq/ordered/search  CDC+dedup+tiers, Merkle roots
```

PyPI ships a **universal pure-Python wheel** (`pip install nedb-engine` works on every platform/Python, and includes the `nedbd` server) — the engine, persistence, and daemon are all pure Python. npm ships **napi-rs** native addons. Native PyO3 acceleration for PyPI is additive/roadmap (the public API is identical with or without it). A RESP-compatible `nedbd` wire protocol and a WASM build are also on the roadmap.

Full design: [`docs/SPEC.md`](docs/SPEC.md).

---

## Repo layout

```
nedb/            pure-Python reference engine (this is what `pip install` ships today)
rust/            production core — nedb-core + nedb-py (PyO3) + nedb-node (napi-rs)
examples/demo.py end-to-end walkthrough
tests/           invariant tests
bench/           embedded micro-bench + Redis head-to-head harness
docs/SPEC.md     architecture specification
.github/         release CI → PyPI + npm on tag
```

## Roadmap

- [x] Reference engine: log, MVCC, relations, indexes, NQL, Cascade, Merkle
- [x] Durable persistence: append-only log (AOF) on disk + replay-on-open; `verify()` / `AS OF` survive restarts
- [ ] RDB-style snapshot checkpoint (fast load) that keeps the AOF chain intact
- [ ] Rust core parity (persistence in `nedb._native`) + criterion benches + `cargo test`
- [x] Universal pure-Python wheel + sdist on PyPI (installs everywhere; ships the `nedbd` command); napi-rs binaries on npm
- [ ] Additive native PyO3 acceleration wheels for PyPI (optional speed; same API)
- [x] `nedbd` server: HTTP/JSON daemon — durable, multi-database; `pip install` ships the `nedbd` command
- [ ] `nedbd`: RESP-compatible wire protocol + native protocol
- [ ] Similarity-picked deltas + schema-aware columnar transforms
- [ ] On-chain (ITC) root anchoring; WASM build

## NEDB Studio

The agentic, prompt-to-database GUI for NEDB — natural language → schema, NQL, seed data, and Python/Node snippets — lives in its own repo: **[Eth-Interchained/nedb-studio](https://github.com/Eth-Interchained/nedb-studio)** (Portal-powered, GPLv3).

## License

Apache-2.0 · © INTERCHAINED, LLC — [interchained.org](https://interchained.org). Built with [AiAssist](https://aiassist.net).
