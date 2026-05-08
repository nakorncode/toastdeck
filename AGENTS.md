# ToastDesk Agent Instructions

## Product Direction

- This project is for a Windows 11 desktop app that shows persistent toast-style notifications on screen.
- The product name is `ToastDesk`. Do not use old prototype names such as `ToastDeck-A` in user-facing UI, executable/project identity, AppUserModelID, shortcuts, settings paths, or startup registration.
- Avoid demo-only wording in user-facing UI unless a screen is explicitly a development/test tool.
- Bundle only production-safe audio assets with clear source/license notes; do not ship third-party game audio even if it appears in OpenCode notifier examples.
- The core UX goal is to avoid missed notifications caused by the normal Windows Notification Center flow.
- Toast cards should appear in an obvious on-screen location, stay visible above normal windows, and remain until the user takes an explicit action on that card.
- Treat this as a custom always-on-top notification surface, not as a thin wrapper around the Windows notification center.

## Planning First

- Do not jump into a full implementation from one broad prompt.
- Work in small, verifiable slices: project scaffold, window/toast prototype, persistence model, action handling, tray/background behavior, installer packaging, then integration surfaces.
- Before choosing a stack or architecture, compare options against Windows API access, always-on-top window behavior, UI quality, app size, packaging, maintainability, and local developer workflow.
- Record durable technical decisions in this file when they become settled.

## Current Technical Decisions

- Use C#/.NET WPF for the first prototype because Windows desktop APIs, always-on-top windows, tray behavior, focus handling, and installer paths are more mature there.
- Treat upcoming work as production hardening: stable app identity, predictable settings migration, graceful startup/shutdown, installer readiness, and fewer demo-only labels.
- Rust is still a possible future option for lower-level components, but do not switch the main app stack until the WPF prototype proves or fails the core Windows behavior.
- The previous ToastDesk attempt had runtime and reliability problems, so keep this project incremental and validate platform behavior in small slices.
- For unpackaged WPF desktop notifications, register an AppUserModelID Start Menu shortcut before sending Windows toasts.
- Prefer polling `UserNotificationListener.GetNotificationsAsync` for this prototype; subscribing to `NotificationChanged` can fail in this unpackaged desktop process.
- The app should eventually ship as an easy Windows installable artifact, preferably `.exe` and/or `.msi`.
- Release automation should publish public Windows assets from version tags: setup `.exe`, MSI, portable ZIP, and SHA256 checksums.

## Windows Behavior Requirements

- Target Windows 11 first.
- Keep notification windows always on top without stealing focus unnecessarily.
- Support persistent visible cards that are dismissed only by explicit user action.
- Plan for multiple simultaneous toasts, stable positioning, screen/work-area awareness, and safe behavior across restarts.
- Avoid depending on users opening the Windows Notification Center to notice important messages.

## Development Guidance

- Prefer the smallest clean change for each step.
- Use local prototypes to validate risky platform behavior before building larger abstractions.
- Use `scripts/run-dev.ps1` as the simple local run entrypoint and `scripts/run-build.ps1` as the simple build entrypoint.
- Current dev and release builds share the same singleton mutex, AppUserModelID, Start Menu shortcut, settings path, and startup Run key. Do not assume they can run side by side until a separate dev identity is intentionally added.
- Do not introduce unrelated product scope such as cloud sync, accounts, or cross-platform support unless explicitly requested.
- Do not run builds, tests, browser checks, or packaging commands by default after normal edits unless the user asks or the change is high-risk.
