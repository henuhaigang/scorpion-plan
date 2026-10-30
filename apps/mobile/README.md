# DayFlow Mobile

Flutter client for DayFlow. Tasks are stored locally; widgets receive a compact
snapshot from the app and do not access the task database directly.

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
automatically. Open DayFlow once, then:

- **iOS:** Long-press an empty area of the home screen, choose **Edit** >
  **Add Widget**, search for **DayFlow**, choose a size, and add it.
- **Android:** Long-press an empty area of the home screen, open **Widgets**,
  find **DayFlow**, then drag its widget onto the home screen.

The app's **Add home-screen widget** button shows these steps and can republish
the latest task snapshot. Widget refresh timing is controlled by the OS.
iOS 17 and later support completing a task from its widget; the completion is
queued in the App Group and applied to SQLite when DayFlow next becomes active.
App Group setup completes before synchronization reads queued iOS widget
completions, so this path also works on the first synchronization after launch.
Earlier iOS versions open DayFlow when the widget is tapped.

The Android and iOS widgets use the same compact indigo/slate visual language
as the mobile app. The OS controls widget sizing, refresh timing, and home
screen placement.
