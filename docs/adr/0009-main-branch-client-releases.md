# ADR 0009: Main-Branch Client Releases

- Status: Accepted
- Date: 2026-11-01

## Context

Each successful commit to `main` should produce retrievable client packages
without committing signing credentials or unexpectedly publishing a stable
release.

## Decision

- GitHub Actions builds macOS DMG, Windows MSI/NSIS, Android APK/AAB, and an
  unsigned iOS Simulator debug app archive on pushes to `main`.
- After all platform jobs pass, Actions publishes the packages as a
  commit-specific GitHub prerelease.
- Android release signing is optional and uses GitHub Secrets. Without the
  upload-keystore secrets, Android output is explicitly marked unsigned; it is
  never signed with the debug key.
- iOS physical-device IPA signing is not configured in this first iteration;
  the uploaded iOS package is simulator-only. Desktop installers remain
  unsigned per ADR 0007.

## Consequences

- A failed platform build or test prevents publishing a partial release.
- To distribute an Android package to users, configure the four documented
  Android signing secrets.
- iOS device distribution and notarized desktop distribution remain separate
  signing setup tasks; the workflow labels unsigned outputs accordingly.
