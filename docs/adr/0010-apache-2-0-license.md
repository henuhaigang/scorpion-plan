# ADR 0010: Apache-2.0 Open Source License

- Status: Accepted
- Date: 2026-11-09

## Context

Splan needs an explicit open source license so users and contributors can rely
on the permitted uses, redistribution terms, patent grant, and liability
limits. The project already documents local-first behavior, privacy
constraints, and release packaging, but it did not previously declare a
license in the repository or package metadata.

## Decision

- Splan is distributed under the Apache License 2.0.
- The repository root `LICENSE` file contains the full Apache-2.0 license
  text.
- The root `package.json`, desktop `package.json`, desktop `Cargo.toml`, and
  mobile `pubspec.yaml` declare the Apache-2.0 license where the package
  format supports it.
- README and project documentation identify Apache-2.0 as the project
  license.
- Third-party dependencies retain their own licenses; this ADR licenses Splan
  project code and documentation, not its dependencies.

## Consequences

- Users may use, modify, and redistribute Splan subject to Apache-2.0.
- Contributors are expected to provide contributions under Apache-2.0 unless
  the repository later adopts a more specific contributor process.
- License metadata must be kept aligned when packaging or publishing client
  artifacts.
- Any future proprietary components, trademarks, or separately licensed
  assets require explicit documentation and user-facing notice.
