# DayFlow V1.0 Release Summary

## Date: 2026-11-01

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
- **107 Rust tests**: ALL PASS
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

## Follow-up: macOS Outside-Click Dismissal and Local Install (2026-11-01)

- Removed tray-leave dismissal and macOS blur-only dismissal. The non-activating
  panel now hides on outside mouse-down events; the click is not consumed.
- The click-away monitor ignores clicks while the pointer is over the tray so
  the normal tray click remains a toggle.
- `cargo fmt --check`, all 107 Rust tests, and strict Clippy passed. The Tauri
  release app and DMG built successfully.
- Installed the ad-hoc-signed release app at `~/Applications/DayFlow.app`.
  Packaged runtime checks verified `⌘⇧Space` opens the centered panel, moving
  the pointer away from the menu-bar corner keeps it open, and one external
  click hides it while DayFlow continues running.
- Android (`emulator-5554`) and iOS (iPhone 16 Pro simulator) apps were
  rebuilt, installed, and launched. `flutter analyze` and all 18 Flutter unit
  tests passed.
- Remaining: direct widget placement/tapping on both home screens, physical
  device/release-signing verification, and cross-platform desktop behavior.

## Main-Branch Client Release Workflow

- `.github/workflows/release.yml` builds and publishes a GitHub prerelease for
  each push to `main`. It packages a macOS DMG,
  Windows MSI/NSIS installers, Android APK/AAB, and an unsigned iOS Simulator
  debug app archive.
- Android artifacts are unsigned unless `ANDROID_KEYSTORE_BASE64`,
  `ANDROID_STORE_PASSWORD`, `ANDROID_KEY_ALIAS`, and `ANDROID_KEY_PASSWORD`
  are configured as GitHub Secrets. The workflow never signs release APKs
  with the debug key.
- The iOS output is simulator-only; an installable iPhone IPA needs Apple
  distribution signing and provisioning, which this workflow does not claim
  to provide. Desktop packages are also unsigned, consistent with ADR 0007.
- Each prerelease is tagged `main-<run number>-<commit SHA>` and includes the
  artifacts from all four platform jobs. Any failed build or test prevents
  publishing.
