# ADR 0001: Desktop Framework

- Status: Proposed
- Decision candidate: Tauri 2 + React + TypeScript
- Alternative: Electron + React + TypeScript

## Context
DayFlow's defining desktop feature is a compact tray/menu-bar quick panel with reliable toggle, blur-hide, draft preservation, multi-monitor positioning, and no accidental app exit.

## Decision
Start with Tauri 2 because the product values a lightweight desktop footprint and the team can use React/TypeScript for UI while isolating native behavior in Rust. This is a candidate decision, not a claim of proven platform parity.

## Milestone evidence
- 2026-10-26 (M1): macOS aarch64 packaged app builds via `pnpm run ci`; Rust domain, SQLite persistence, and application use cases are implemented. Tray/menu-bar runtime behavior remains unverified and is still required before release.
- 2026-10-27 (M2): macOS aarch64 packaged app builds via `pnpm run ci`; tray icon, panel show/hide/toggle, close-to-hide, blur-hide state coordination, draft persistence commands, and the React quick panel are implemented. The packaged app was launched locally as `DayFlow.app`; visual multi-display and Windows tray behavior still require platform-specific verification before release.
- 2026-10-30: macOS panel show uses Tauri's `window.center()` and is toggled by `⌘⇧Space`; click-away and Escape dismissal remain unchanged, and the panel is not always-on-top. Rust tests, strict Clippy, and macOS app/DMG builds pass; exact on-screen centering and interactive shortcut behavior had not yet been runtime-verified.
- 2026-10-30: Full-screen VS Code testing initially showed that the panel either failed to appear without activation or switched away from the full-screen Space when focused. The packaged follow-up uses accessory activation and a non-activating floating window. Runtime automation then showed the overlay above full-screen VS Code; opening the add-task form by mouse left Code frontmost and full-screen, and the form remained interactive. This supersedes the initial failure for shortcut display and panel interaction. A physical click on the menu-bar icon, outside-click/blur behavior, Dock/Cmd-Tab visibility, and multi-display behavior remain unverified.
- 2026-10-30: A screen capture and Accessibility API measurements found that the packaged panel was not vertically centered on the full display. The show path now calculates its position from the current monitor's physical bounds; a macOS-only unit test covers the coordinate calculation. A Clippy compile attempt caught borrowed monitor coordinates, which were corrected by copying the `Copy` geometry values. The corrected Rust source and unit test have not yet been rebuilt or run, and the repositioning has not been runtime-verified.
- 2026-10-30: The existing 800 ms blur recheck combined with a 750 ms show guard could delay dismissal after the first outside click. The blur recheck now runs after the remaining short 150 ms show guard (or a 40 ms minimum interval), so a single outside-click focus loss can hide the panel promptly. Rust tests and packaged macOS runtime behavior still need verification for this change.

## Required evidence before release
- macOS packaged app: physical menu-bar click, shortcut toggle, full-screen non-activating overlay, centered panel after the latest positioning change, single-click blur-hide, focus, and multi-display behavior.
- Windows packaged app: tray icon, toggle, blur-hide, taskbar/display scaling, multi-display.
- Repeated clicks and blur/tray race tests.
- Startup, explicit quit and draft preservation.
- Compare actual defects and implementation cost against Electron if a required behavior is unreliable.

## Reversal criteria
If a core requirement cannot be implemented reliably with supported Tauri APIs without brittle platform-specific workarounds, create an ADR comparing Electron before proceeding further.
