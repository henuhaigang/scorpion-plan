# Splan Mobile

Flutter client for Splan. Tasks are stored locally; widgets receive a compact
snapshot from the app and do not access the task database directly.

Splan is licensed under the Apache License 2.0; see [LICENSE](../../LICENSE).

## Development and verification

From this directory:

```bash
flutter pub get
dart format --set-exit-if-changed lib test integration_test
flutter analyze
flutter test
```

Install and launch on an available simulator:

```bash
flutter run -d <android-emulator-id>
flutter run -d <ios-simulator-id>
```

The Android host must apply both the Android Gradle plugin and the Kotlin
Android plugin because the activity, widget provider, and notification
receivers include Kotlin sources. Database opening is shared between
simultaneous initial list loading and startup widget/reminder synchronization
so both operations use the same in-flight SQLite connection.

## Add a home-screen widget

Widgets are managed by iOS and Android and are not placed on the home screen
automatically. Open Splan once, then:

- **iOS:** Long-press an empty area of the home screen, choose **Edit** >
  **Add Widget**, search for **Splan**, choose a size, and add it.
- **Android:** Long-press an empty area of the home screen, open **Widgets**,
  find **Splan**, then drag its widget onto the home screen. After placing
  it, long-press the widget and drag its resize handles; smaller heights show
  fewer task rows, while larger sizes reveal more rows and the legacy summary.

The app's **Add home-screen widget** button shows these steps and can republish
the latest task snapshot. Widget refresh timing is controlled by the OS.
iOS 17 and later support completing a task from its widget; the completion is
queued in the App Group and applied to SQLite when Splan next becomes active.
App Group setup completes before synchronization reads queued iOS widget
completions, so this path also works on the first synchronization after launch.
Earlier iOS versions open Splan when the widget is tapped.

The Android and iOS widgets use the same compact indigo/slate visual language
as the mobile app. The OS controls widget sizing, refresh timing, and home
screen placement.

## Latest verification

On 2026-11-09, `flutter analyze` reported no issues and `flutter test`
passed. The mobile home page now refreshes the task board when the local date
changes while the app remains open. A notification-delivery probe was added
under `integration_test/` for device-level reminder verification.

On 2026-11-01, the app was rebuilt, installed, and launched on the Android
emulator (`emulator-5554`) and iPhone 16 Pro simulator
(`EEB96DB5-39B8-4B91-A3CB-1FCB196CDF88`). Directly placing/tapping the widgets
on the simulator home screens is still a separate, unverified OS interaction.
