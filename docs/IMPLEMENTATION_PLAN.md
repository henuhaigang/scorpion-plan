# Implementation and Validation Status

The original milestone order is retained for traceability. Implementation through M6 is complete; remaining work is platform verification and release follow-up, not starting the mobile implementation.

Splan is licensed under Apache-2.0. The root `LICENSE` file and package metadata are the authoritative license declarations.

| Milestone | Scope | Status | Evidence |
|---|---|---|---|
| M0 | Repository and ADR baseline | Complete | `docs/adr/` |
| M1 | Domain and local persistence | Complete | Rust and Dart unit tests; versioned SQLite migrations |
| M2 | Desktop quick panel | Complete on macOS | Packaged runtime evidence in `docs/milestones/M3.md` and `docs/V1_RELEASE.md` |
| M3 | Task management UI | Complete | `docs/milestones/M3.md` |
| M4 | Reminder engine | Implemented; OS delivery is best-effort | `docs/milestones/M4.md` and `docs/milestones/M6.md` |
| M5 | Release hardening | Implemented; signing and cross-platform checks remain | `docs/milestones/M5.md` and `docs/V1_RELEASE.md` |
| M6 | Flutter mobile app, reminders, and widgets | Implemented; date rollover refresh added; platform verification partial | `docs/milestones/M6.md` |
| M7 | Cloud sync | Out of scope for V1 | Requires a separately approved design |

## Remaining V1 Verification

- macOS hour-only time editing is verified through the packaged app self-test; see `docs/V1_RELEASE.md`.
- Desktop and mobile date-rollover board refresh is implemented and covered by desktop unit tests; physical-device overnight behavior remains a platform verification item.
- Place and directly interact with Android and iOS home-screen widgets; simulator launcher catalog/preview and shared action paths do not prove this OS-level flow.
- Validate physical devices, release signing, and Windows desktop behavior when those environments are available.
- Keep each unrun platform scenario marked NOT RUN; do not infer runtime behavior from a source build or unit test.

## Engineering Gates

For code changes, run the focused regression test first, then the relevant formatter, lint, unit/integration tests, and build. Report exact commands and observed results, including unavailable toolchains and platform checks. Preserve the domain, storage, reminder, and privacy constraints in `AGENTS.md` and the specifications linked from `README.md`.
