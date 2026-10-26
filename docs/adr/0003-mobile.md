# ADR 0003: Mobile Platform

- Status: Proposed
- Decision candidate: Flutter for iOS and Android
- Rationale: one mobile UI codebase, suitable for task management and widgets/notifications through platform plugins and native integration.
- Risks: widgets and notification scheduling are OS-controlled; plugin support and platform restrictions must be verified separately on iOS and Android.
- Rule: mobile implementation starts after domain contracts and desktop local core stabilize.
