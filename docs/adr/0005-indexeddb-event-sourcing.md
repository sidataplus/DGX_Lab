# ADR 0005: Browser-local event log and snapshots

**Status:** Accepted, wiring pending

## Decision
Persist commands/system events with periodic world snapshots in IndexedDB. Export integrity-protected `.dgxlab` bundles. Avoid native SQLite in the first release.

## Consequences
Static web parity and minimal Tauri privileges. Complex relational analytics are deferred.
