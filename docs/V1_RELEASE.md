# DayFlow V1.0 Release Summary

## Date: 2026-10-28

## Status: DESKTOP COMPLETE, MOBILE PARTIAL

## Milestones Completed

| Milestone | Description | Status |
|-----------|-------------|--------|
| M0 | Repository and ADR baseline | COMPLETE |
| M1 | Domain and local persistence | COMPLETE |
| M2 | Desktop quick panel | COMPLETE |
| M3 | Task management UI | COMPLETE |
| M4 | Reminder engine | COMPLETE |
| M5 | Release hardening | COMPLETE |
| M6 | Mobile app (Flutter) | PARTIAL — widget placement and physical-device verification remain |
| M7 | Cloud sync | OUT OF SCOPE (V1.1+) |

## Desktop (macOS) — COMPLETE

### Features
- Tray/menu-bar quick panel with show/hide/quit
- Task board: Today Timed, Today Untimed, Legacy, Future sections
- Quick add, edit, reschedule, complete, undo, cancel, restore, soft-delete
- Reminder engine: reconciliation, polling, OS notifications
- Backup/export to SQLite file
- Local SQLite persistence with versioned migrations
- Optimistic versioning, soft deletion
- DST-aware reminder scheduling
- Light/dark theme, accessibility

### Test Results
- **105 Rust tests**: ALL PASS
- **23 Frontend tests**: ALL PASS
- **Format check**: PASS
- **Lint (ESLint + Clippy)**: PASS
- **TypeScript typecheck**: PASS
- **Build**: SUCCESS (macOS app and DMG packaged)

### Build Artifacts
- `apps/desktop/src-tauri/target/release/bundle/macos/DayFlow.app`
- `apps/desktop/src-tauri/target/release/bundle/dmg/DayFlow_0.1.0_aarch64.dmg`

### How to Build
```bash
pnpm install
pnpm build
```

### How to Run
```bash
open apps/desktop/src-tauri/target/release/bundle/macos/DayFlow.app
```

### Data Location
```
~/Library/Application Support/com.dayflow.desktop/dayflow.sqlite
```

### Backup
```
~/Library/Application Support/com.dayflow.desktop/backups/
```

## Mobile (iOS/Android) — PARTIAL

### Implemented
- Local task flows, SQLite persistence, best-effort reminders, and native Android/iOS widget extensions.
- The app publishes task snapshots; widgets must be added manually through the OS widget gallery.
- The app provides platform-specific setup instructions and a control to refresh widget data.

### Remaining Verification
- Place and tap widgets on Android/iOS home screens. The Android launcher
  picker and preview render, but home-screen placement and direct checkbox
  interaction were not completed.
- Tap the iOS 17+ widget App Intent directly on the home screen.
- Verify on physical devices and with release-signed builds.
- Retest packaged macOS shortcut/outside-click behavior with only one DayFlow
  process running.

### How to Build
```bash
cd apps/mobile
flutter pub get
flutter test
flutter build ios --simulator --no-codesign
flutter build apk
```

## Known Limitations

1. **Unsigned macOS builds**: Right-click → Open on first launch.
2. **No auto-update**: Manual download required.
3. **Mobile widgets require user placement**: The app cannot add a widget to the home screen automatically; Android picker preview works, but direct placement/tap and iOS home-screen interaction remain unverified.
4. **Windows/Linux desktop**: NOT RUN (macOS only environment).
5. **Cloud sync**: Not in V1 (planned for V1.1).

## Architecture

```
apps/
├── desktop/          # Tauri 2 + React + TypeScript + Rust
│   ├── src/          # Frontend (React, TypeScript)
│   └── src-tauri/    # Backend (Rust, SQLite)
└── mobile/           # Flutter (Dart)
    ├── lib/          # Dart source
    └── test/         # Dart tests

docs/
├── PRD.md            # Product requirements
├── DOMAIN_MODEL.md   # Domain specification
├── IMPLEMENTATION_PLAN.md
├── adr/              # Architecture Decision Records
└── milestones/       # Milestone reports (M0-M6)
```

## Key Decisions (ADRs)

1. **Desktop**: Tauri 2 + React + TypeScript (ADR 0001)
2. **Storage**: SQLite with rusqlite (ADR 0002, 0004)
3. **Mobile**: Flutter (ADR 0003)
4. **Domain**: Rust domain layer (ADR 0005)
5. **Reminders**: App-resident scheduler, SQLite job table (ADR 0006)
6. **Packaging**: Tauri bundler, no auto-update in V1 (ADR 0007)

## Security

- No cloud dependency in V1
- No analytics, advertising, or telemetry
- No account/login system
- Local-only data storage
- No secrets in code or logs

## Next Steps (V1.1+)

1. Complete mobile app (build, test, verify on iOS/Android)
2. Cloud sync design (Java/Spring Boot + PostgreSQL)
3. Authentication and device registration
4. Conflict resolution and tombstone propagation
5. Auto-update (after security review)
6. Windows/Linux desktop verification

## Follow-up: Android Startup, Mobile UI, and macOS Dismissal (2026-10-30)

- The Android app module now applies its Kotlin Gradle plugin explicitly and
  shares concurrent startup SQLite opens between initial rendering and
  widget/reminder synchronization.
- The Flutter task board and both native widget surfaces now share a compact
  indigo/slate visual treatment with explicit untimed-task labels.
- macOS outside-click blur rechecks no longer wait a fixed 800 ms after the
  click; they are scheduled after the short panel-show guard. Runtime behavior
  remains to be verified.
- These source changes are pending native verification. Android/iOS builds,
  simulator installation, Rust checks, and packaged macOS blur behavior have
  not been rerun for this follow-up.

## Follow-up: Simulator Regression and Widget Preview Fix (2026-10-31)

- Fixed an iOS first-sync race: `MobileTaskSync` now initializes the Widget
  App Group before reading pending widget completion IDs.
- Fixed Android launcher preview inflation: a generic `View` divider was not
  supported by `RemoteViews`; it is now a `TextView`.
- `flutter analyze` passed with no issues; `flutter test` passed all 18 tests.
- Android and iOS integration flows passed for task/widget completion,
  notification denial, and reminder scheduling/cancellation. The added queued
  widget completion test passed on both simulators.
- Android and iOS debug apps rebuilt, installed, and cold-launched on the
  Pixel 7 emulator (API 34) and iPhone 16 Pro simulator (iOS 18.3).
- The Android widget picker lists DayFlow and its preview renders correctly.
  Actual home-screen placement/direct tapping remains unverified.
- `cargo fmt --check`, `cargo test --locked` (105 tests), strict Clippy, and
  release build passed. `pnpm --filter dayflow-desktop build:desktop` produced
  the macOS app and DMG successfully.
- Existing macOS runtime automation previously verified the shortcut and
  outside-click behavior. A fresh retest in this session was inconclusive
  because two DayFlow processes were already running; do not count that attempt
  as a new runtime pass.
- Remaining risks: direct iOS App Intent tapping, direct Android widget
  placement/tapping, physical devices, release signing, and a single-instance
  macOS runtime retest. Flutter also reports future Kotlin Gradle Plugin and
  Swift Package Manager compatibility warnings for dependencies.
