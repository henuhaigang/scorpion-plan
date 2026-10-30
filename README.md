# DayFlow

DayFlow is a local-first task and schedule app for macOS, iOS, and Android.
The desktop client uses Tauri 2, React, TypeScript, Rust, and SQLite. The
mobile clients use Flutter, Dart, SQLite, and native home-screen widgets.
Core task management works offline; cloud sync is out of scope for V1.

## Repository

```text
apps/
  desktop/     Tauri desktop app
  mobile/      Flutter iOS and Android app
docs/
  adr/         Architecture decisions
  milestones/  Implementation and validation reports
```

Product and engineering requirements are documented in
[docs/PRD.md](docs/PRD.md), [docs/DOMAIN_MODEL.md](docs/DOMAIN_MODEL.md),
[docs/DATA_AND_SYNC.md](docs/DATA_AND_SYNC.md),
[docs/REMINDER_RULES.md](docs/REMINDER_RULES.md),
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md),
[docs/ACCEPTANCE_TESTS.md](docs/ACCEPTANCE_TESTS.md), and
[docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md).

## Prerequisites

- Node.js and pnpm for the desktop frontend.
- Rust and the platform build tools for the Tauri desktop app.
- Flutter/Dart for mobile analysis and tests.
- Android SDK/emulator for Android builds and simulator installation.
- Xcode and CocoaPods for iOS simulator builds.

## Desktop development

From the repository root:

```bash
pnpm install
pnpm run lint
pnpm run build
pnpm run test
```

Run the Tauri desktop app in development mode:

```bash
pnpm --filter dayflow-desktop tauri dev
```

Package the desktop app for the current host platform:

```bash
pnpm --filter dayflow-desktop build:desktop
```

The desktop frontend and Rust host have separate checks. `pnpm run lint`
includes Rust Clippy; use the root scripts to run the full configured suite.

## Mobile development

From `apps/mobile`:

```bash
flutter pub get
dart format --set-exit-if-changed lib test integration_test
flutter analyze
flutter test
```

Build and install on available simulators:

```bash
flutter run -d <android-emulator-id>
flutter run -d <ios-simulator-id>
```

Widgets are managed by the operating system and must be added by the user:
long-press the home screen and select DayFlow from the widget gallery. See
[apps/mobile/README.md](apps/mobile/README.md) for platform behavior and
widget setup.

## Current status

Desktop and mobile task flows, local SQLite persistence, reminder handling,
and native widget targets are implemented. Mobile simulator, physical-device,
widget placement, and packaged desktop behavior still have platform-specific
verification gaps. Do not treat a source change or a frontend-only test as a
successful native build. Detailed evidence and remaining risks are in
[docs/V1_RELEASE.md](docs/V1_RELEASE.md) and
[docs/milestones/M6.md](docs/milestones/M6.md).

## Product principles

- Stable UUIDs, soft deletion, and optimistic version checks for task writes.
- A task without a scheduled time stays untimed; it is never converted to
  midnight.
- Task persistence succeeds independently of OS reminder scheduling.
- Timestamps are UTC instants; schedule dates and wall-clock times remain
  separate.
- No account, analytics, ads, or cloud requirement in V1.
