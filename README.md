# FreeSynergy Activities — remember what I or we did

**A timeline of your own work, without anyone watching you.**

FreeSynergy Activities (earlier History) notices what happens across your FreeSynergy programs and keeps a short line for each event: "edited this page", "finished that task". You can scroll it, filter it, collapse a busy hour into one line, and forget parts of it.

What is different here: it keeps a summary, never the content. A diary text stays a wiki page, and Activities only points to it. The history is local to you. Sensitive things never arrive in it. Forgetting is real for the past: matching entries are removed and a record remembers only salted hashes of what you forgot, so a forgotten project name is not stored in readable form.

## What it can do today

Latest tag: `v0.0.2` (program with a window, server and frontend; the main view is a marked preview with example data).

- **A window and a server (new in 0.0.2).** `fs-activities` makes sure the server runs and opens the frontend `simple`: name, icon, what works today and what comes. Everything that does not work yet says "coming soon"; the filters "By me" and "By others" say openly "The source does not yet deliver who it was" instead of being greyed out silently. A missing server or one that does not fit is said in the window, never a grey window (the reason text of a missing server comes from the transport library and is English only for now: known, provisional). The window opens on a main view in the look of its models: filters on the left, search above, the days as an agenda; its rows are example data marked "Preview" ("Example A/B/C"), and they vanish as soon as the server offers real activities. No real activities yet: the core is not wired to the window. Try `fs-activities --version`, `fs-activities --help`.


- A timeline of activities from other programs, newest first, with filters and paging.
- A digest that groups runs of events on the same object.
- Forget entries by filter, for the past only.
- Reading from the FreeSynergy hub (adapter `fs-activities-hub`, tested against a real hub); in version 0.0.2 the adapter is left out of the workspace build, because the newest `fs-registry` tag (v0.0.2) still pins old libraries (see DESIGN.md "Gaps").
- Never reads relationships or survey answers.
- Tests: 49 (the hub tests are not run in 0.0.2, the adapter is excluded), with real stores and a real hub.
- Checked by an independent reviewer over three rounds.

## What is not there yet

- The timeline in the window is a marked preview with example data; only the filter by program, the search and showing or hiding the example work; the server answers only what it offers (nothing yet) and the contract version. The server keeps running after the window closes until it is stopped or replaced (no lease).
- Platform: Linux (Windows and macOS follow; the transport uses Unix sockets).
- Who did something is empty for now, and the time is when the entry arrived, not necessarily when it happened (see DESIGN.md).
- Forgetting covers this index only. The hub's own log is not touched.
- No sharing and no group view. No import of telemetry, no time tracking.
- A hub connection that dies is not reconnected by itself.
- Contract: draft (`history.capnp`, not locked). The basics are released with this draft contract. This frontend and server are its first consumers.
- Gaps in the window, named: no timeline widget in fs-components yet (the lists are text lists for now, provisional), no date-range picker (provisional), no context menus, keyboard shortcuts or tree; see DESIGN.md "Gaps".

## Part of FreeSynergy

Activities belongs to the project-management suite, and to anyone who wants to remember. FreeSynergy is a family of programs that are free for everyone, forever, and that keep your data yours. The documentation, the manifesto and the overall plan live in [fs-documentation](https://github.com/FreeSynergy/fs-documentation); the reading version is at <https://freesynergy.github.io/fs-documentation/>. Start with the [manifesto](https://freesynergy.github.io/fs-documentation/manifesto/) if you want to know why we do this.


## For developers

- Workspace crates: `fs-activities-core` (core, no GUI), `fs-activities-hub` (reads activities from the hub; not built in 0.0.2, see above), `fs-activities-server` (the contract over a local socket), `fs-activities-client` (the contract as a client), `fs-activities-simple` (the frontend `simple`, a process of its own, depends only on the client and fs-components), `fs-activities-bin` (the command `fs-activities`: starter and `serve`).
- Engine and frontend are separate processes (ADR 0021, 0094 draft); the manifest of the frontend is `packaging/frontends/simple.metainfo.xml`.
- The contract is `history.capnp` in [fs-interfaces](https://github.com/FreeSynergy/fs-interfaces) (draft). Activities owns it; the capability key stays `history` (ADR 0087).
- Build and test: `cargo build --workspace` and `cargo test --workspace` (the window crates need the system libraries of Blinc; the tests do not open a window).
- Design and decisions: [DESIGN.md](DESIGN.md).
- Licence: MIT OR Apache-2.0.

## Auf Deutsch

**Eine Zeitleiste deiner eigenen Arbeit, ohne dass dich jemand beobachtet.** FreeSynergy Activities (früher History) merkt, was in deinen FreeSynergy-Programmen geschieht, und behält eine kurze Zeile je Ereignis. Es behält eine Zusammenfassung, nie den Inhalt; der Index gehört dir und bleibt lokal.

**Was es heute kann.** Letzter Tag `v0.0.2` (Programm mit Fenster, Server und Frontend; die Hauptansicht ist eine gekennzeichnete Vorschau mit Beispieldaten). Der Kern (Zeitleiste, Filter, Zusammenfassungen, Vergessen, Hub-Anschluss) liegt als Bibliothek vor. Neu ist ein Fenster mit Server: `fs-activities` startet den Server und das Frontend `simple` mit Name, Symbol, „Was es heute kann“ und „Was kommt“. Was noch nicht geht, sagt „kommt noch“; die Filter „Von mir“ und „Von anderen“ sagen offen „Quelle liefert noch nicht, wer es war“. Fehlt der Server oder passt er nicht, steht das im Fenster (der Grund eines fehlenden Servers kommt aus der Transport-Bibliothek und ist vorläufig nur englisch). Das Fenster öffnet auf einer Hauptansicht im Aussehen der Vorbilder: links Filter, oben Suche, die Tage als Agenda; ihre Zeilen sind als „Vorschau“ gekennzeichnete Beispieldaten („Beispiel A/B/C“) und verschwinden, sobald der Server echte Aktivitäten anbietet. Echte Aktivitäten gibt es noch nicht, der Kern ist noch nicht am Fenster angeschlossen; bedienbar sind nur der Filter nach Programm, die Suche und das Ein- und Ausblenden des Beispiels.

**Plattform.** Linux (Windows und macOS folgen).

**Für Entwickler.** Frontend und Engine sind getrennte Prozesse; das Frontend hängt nur am Client und an fs-components. Bauen: `cargo build --workspace`, `cargo test --workspace`. Vertrag: Entwurf (`history.capnp`, nicht gesperrt). Die Grundfassung erscheint mit diesem Entwurf. Lizenz: MIT OR Apache-2.0.
