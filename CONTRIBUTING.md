# Contributing

Thank you for considering a contribution to ToastDesk.

## Development

Use Windows PowerShell from the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-dev.ps1
```

## Build

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-build.ps1
```

## Guidelines

- Keep changes focused.
- Prefer Windows 11 behavior correctness over broad cross-platform abstractions.
- Do not add bundled media assets unless their license is clear and documented.
- Avoid logging notification contents.
