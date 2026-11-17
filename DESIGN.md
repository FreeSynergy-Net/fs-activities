# fs-activities – Design

`fs-activities` observes what happens across programs (ADR 0020). This repo holds the **program core**, no GUI. Contract:
`history.capnp` in `fs-interfaces` v0.0.27 (draft 0.1, not locked). Program docs: `fs-documentation/docs/en/programs/apps/activities/`.

The repo and crates were renamed from `fs-history` to `fs-activity` and then to `fs-activities` (ADR 0087, plural rule), but types and contract keep the name "history" (`HistoryRepo`, `History`, `history.capnp`, capability IDs `history.*`, `PROGRAM = "history"`, MIME `vnd.freesynergy.history-*`): these names are part of the wire contract in `fs-interfaces` and other programs depend on them, so behaviour towards the outside stays the same and changing them needs its own decision.

## State

Version 0.0.2 (the version of the tag; earlier texts said 0.1.1). A capnp server and a window exist as basics (see "The basics" below); the core is not wired to the server yet.

| Area | State |
|---|---|
| Entries: derived index, unlogged, no payload; timeline, get, filters, keyset paging | done |
| Digest | done |
| Forget with tombstones, past only, salted filter texts | done |
| Observer over the port `ActivitySource`; hub adapter `fs-activities-hub` against a real hub | done; at most 16 activities per pull until the hub's next tag |
| `Relationship` and `SurveyResponse` never read | done |
| Actor | empty; `originalActor` stays empty until the person releases it (strategy, in `hub.capnp`) |
| Original time | receive time for now; `originalAt` comes with the next fs-registry tag |
| A dead hub session | not reconnected; the host starts a new one |
| capnp server | basics only: `capabilities` and `supportReport`, see "The basics" |
| Tests | 49, real stores and a real hub, the 80 MiB test included |

Checked by an independent reviewer over three rounds; the last found one small error, fixed in 0.1.1 with a test.

## Decisions (the owner may change them; all provisional)

### Does it have its own data?
No content of its own, but it needs an index. The hub only offers a cursor-based `pull`: no query by time, person or project, and a view would have to scan the whole log each time. So entries are a **derived index**: a summary of each activity, never its `payload`. A diary's text stays a wiki entry (ADR 0031). So it stays a small program, and the views are its frontends.

### An entry
One `StoredObject` per delivered activity; envelope type `Activity`, `context` = the object's context, `published` = `Entry.at`, id = the hub activity id made into an `ObjectId` where the types allow it, otherwise an index field `activity`. Index fields: `activity`, `program`, `kind`, `object`, `objectType`, `person`. Content: the summary fields. Written with `UnloggedObjects`, because the index is derived: logging it would double every activity and send it towards the hub again. Query through `ObjectQuery`, ordered `PublishedDesc` with a tie break on id so paging is stable.

### Who and when: limits of today's hub
The hub's `Activity.actor` is the program name and `at` is when the hub applied the push. So `Entry.actor` is empty, `at` is the receive time, and `atIsReceived` is true. An offline week arrives all at once and "what did I edit in someone else's object" cannot be answered. **Requested from thread 3:** append the original actor and time to `hub.capnp` `Activity`. When they arrive, the source fills them in; the contract already has the fields. `person` in a filter matches `actor`, or `attributedTo` where `actor` is empty.

### Log-only verbs
`Invite`, `Accept` and the like carry only an id in their envelope. Name, type, context and `attributedTo` are filled in from the object's latest entry, if there is one.

### Privacy
* Local to this person. No sharing and no group view in this version: it comes with the sharing interface.
* `Relationship` is never read (sprint rule, role privacy). `reads` is an explicit default list without it, and a constructor refuses a list that names it. Survey answers (`SurveyResponse`) are not read either.
* Sensitive objects never arrive (ADR 0029). The hub reports an object that became sensitive as a `Delete`, so **on `Delete` all entries of that object lose `name` and `context`**, and `objectType` stays. A delete therefore also removes the name from the history; provisional, because the two cases cannot be told apart.
* `forget(filter)` removes the matching entries and stores a tombstone: the forget time, the removed activity ids, the removed object ids, and the filter. **Forgetting covers the past only.** An activity counts as forgotten if its id was removed, or if its `at` is at or before the forget time and it matches the filter or names a removed object; later activities are recorded again. While `at` is the hub's receive time, a late push gets a later time and comes back; that goes away once the hub delivers the original time.
* Filter texts in a tombstone are salted hashes (SHA-256, a random salt per tombstone), so a forgotten project name is not kept readable. Stated limit: a guessed value can still be tested. Forgetting happens in this index only; the hub's log is not touched, and the interface must say so. An empty filter is refused, and all-zero values count as unset.
* `Update` after `Delete` with no new `Create` stays blank. An `Undo` is not handled, since the hub's log has none.
* Types are compared normalised (case, `as:` prefix). A log-only verb of unknown type is kept only from a source that filters types itself (`ActivitySource::filters_types`, true for the hub).

### Ingest
Port `ActivitySource` (Strategy): `pull(limit) -> Batch { items: Vec<SourceActivity>, last, more }`, `ack(last)`. `SourceActivity` is a plain value with the summary fields, so the core does not know the hub. `Observer::run_once` (Template method): pull, drop what tombstones say, write, ack only after the write; idempotent by activity id, so redelivery changes nothing. `Observer::catch_up` loops while `more`. The payload is not read or kept.

### Digest
A pure function: entries newest first → grouped when object and program are equal and the gap between neighbours is at most `windowMinutes`. Paged by keyset. A page ends where no run of one object and program crosses. Over 10 000 entries a run is cut, and `DigestEntry.continues` says so.

### Feed
`added` and `forgotten` through `tokio::sync::broadcast`, filtered per subscription; dropping the guard unsubscribes. In-process only, the RPC server comes from thread 3.

### Not built
Telemetry import (OpenTelemetry not read), time tracking derived from history, sharing and group views. They have no contract method yet.

## The basics (0.0.2, workflow `grundfassung`)

Engine and frontend are separate processes (ADR 0021, ADR 0094 draft). Pattern map (ADR 0043):

| Crate | Role | Pattern |
|---|---|---|
| `fs-activities-server` | `history.capnp` 0.1 over a local socket (`fs-rpc` v0.0.2); `capabilities` (a `CapabilityList`: no ids, the contract version `0.1`, named by the server because the core has none) and `supportReport`; the rest is `unimplemented` | Adapter |
| `fs-activities-client` | `Hello::fetch`: handshake (`Control.version`, replace a server that differs), `capabilities`, contract check | Template method |
| `fs-activities-simple` | frontend `simple`, a process; depends only on the client and fs-components v0.0.25; window = name, icon, what works today, what comes (all locked, "coming soon") | Adapter, Builder |
| `fs-activities-bin` | command `fs-activities`: starter (server, `--check`, fallback to `simple`), `serve`, `--version`, `--help` | Facade, Strategy |

Contract rule (ADR 0094 draft, item 4): same main level, minor level of the server not lower; a frontend exits with 3 and a message when it does not fit; the starter says so and opens the standard frontend. Not done, on purpose: no wiring of the core to the server (the window shows no timeline), no lease (the server runs until stopped or replaced). Contract: draft (`history.capnp`, not locked). The basics are released with this draft contract. The name `activities.capnp` does not exist in `fs-interfaces` v0.0.27: the contract file is `history.capnp` (interface `History`, capability key `history`).

Decisions for the window (owner, 2026-11-17): the filter "By me / By others" is wanted; as long as the hub does not deliver who did it, it says openly "The source does not yet deliver who it was" (menu entry with the reason, never silently greyed out). The feed of the own timeline is Atom (the standard), opt-in, stage "later".

### Gaps (components that do not exist yet; the nearest existing one stands in, marked "provisional")

- Timeline widget (day groups, loading more): provisional a text list of the shared text view.
- Date-range picker: provisional none; the period filter is listed under "what comes".
- Command model, shortcuts, context menu, tree for summaries (PLAN-components-menue M1 to M5): provisional locked menu entries with a reason.
- Calendar heat map (later): none.
- Error text of a missing server comes from `fs-rpc` in English: known, provisional.
- Hub: `fs-activities-hub` is excluded from the workspace (`exclude` in `Cargo.toml`): `fs-registry-client` v0.0.2, the newest tag, still pins fs-interfaces v0.0.1 and fs-objects v0.0.1 (two sources per library, check_lock_sources red). It returns to the workspace with a registry tag that pins the current libraries; the basics do not use it.
