# ADR 0007: App Packaging and Distribution (V1)

- Status: Accepted
- Date: 2026-10-26
- Deciders: DayFlow engineering (agent-assisted)

## Context

V1 must ship installable desktop builds for macOS (arm64 first) and Windows,
with a documented, reproducible build path. `IMPLEMENTATION_PLAN.md` (M5)
forbids auto-update without a security review.

## Decision

1. Use the **Tauri 2 bundler** (`tauri build`):
   - macOS: `.app` (always) and `.dmg` (distribution artifact).
   - Windows: `.msi` and NSIS `.exe` installers.
2. **No auto-update in V1.** Update delivery is a Milestone 5+ decision that
   requires a security review of the update channel and signing.
3. **Signing**: V1 builds are unsigned/ad-hoc. macOS Gatekeeper behavior for
   unsigned apps is documented in the README; Developer ID + notarization is a
   release-gate item, not a code item.
4. Bundle identifier: `com.dayflow.desktop`.
5. Local data location is documented (Tauri app data dir) with a backup/export
   strategy defined in Milestone 5.

## Consequences

- `pnpm build` at the repo root must produce the artifacts above on each OS;
  CI runs the macOS leg, the Windows leg is NOT RUN on macOS-only machines and
  is reported as such.
- Unsigned macOS builds require user action on first launch (documented).

## Reversal criteria

Adopt a different bundler only if Tauri bundling produces unreproducible or
broken installers on a target OS after documented attempts.
