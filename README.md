<h1 align="center">⚡ nitroDB</h1>
<p align="center"><b>The embedded database that goes brrr.</b></p>
<p align="center"><i>Stupid-fast, append-only, zero-server storage you drop into your app and forget about.</i></p>

---

```
63,400 writes/sec        1,340,000 point reads/sec        one fsync per batch
```

nitroDB is a **tiny, blazing-fast, in-process** datastore. No daemon. No schema migrations. No 400 MB of dependencies. Just `import` it and start flooding it with data — it keeps up.

## Why people can't shut up about it

- 🏎️ **Fast where it counts.** Append-only + group-commit: one `fsync` per *batch*, not per record. Bigger batches get **cheaper per item**. Throughput goes *up* under load, not down.
- 🪶 **Featherweight.** In-process, near-zero footprint. Boots instantly, runs in memory, persists when you say so.
- 🧷 **Durable, not fragile.** Every write is hash-chained and the log replays clean on open — crash-safe without the ceremony.
- 🚀 **Scales with your ambition.** Same engine family as **CryptoDB** — flip on the content-addressed DAG the day you need time-travel and tamper-evidence. Start fast, grow powerful.

## Install (10 seconds, tops)

```bash
npm install nitrodb     #  pip install nitrodb     #  cargo add nitrodb
```

```js
import { NedbCore } from "nitrodb";
const db = new NedbCore();
console.time("100k");
for (let i = 0; i < 100_000; i++) db.put("events", String(i), JSON.stringify({ t: Date.now(), i }));
db.flush();              // one durable group-commit for the whole burst
console.timeEnd("100k"); // …go on, time it.
```

## Built for

Edge & embedded · high-throughput event logging · local-first apps · game state · CLIs · **anywhere "just make it fast" beats "stand up a server."**

<p align="center"><b>If nitroDB just saved your afternoon, ⭐ it and tell a friend.</b></p>

---

<sub>nitroDB is a distribution of the <b>NEDB</b> engine, tuned for speed + simplicity (benchmarks measured on commodity hardware — yours will vary). Engine development: <a href="https://github.com/Eth-Interchained/nedb">Eth-Interchained/nedb</a>. © Interchained LLC.</sub>
