# Sift

Sift is a Windows-only Downloads organiser built around two workflows: safe automatic rules for predictable files and a rapid, keyboard-first triage queue for everything else.

## Run the interface

```powershell
corepack pnpm install
corepack pnpm dev
```

The browser build opens in preview mode with representative files. No local files are touched.

## Run the desktop app

On Windows, install the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/), then run:

```powershell
corepack pnpm tauri dev
```

The desktop build scans the selected folder without changing it. Moves use collision-safe names, Trash uses the operating system Recycle Bin, and operations are recorded in a local SQLite database.

## Safety model

- Scans skip hidden files, directories, symbolic links, and incomplete downloads.
- Rule runs require a file-by-file preview and explicit confirmation.
- A move never overwrites an existing file.
- Trash is never permanent. Restore trashed items through the operating system Recycle Bin.
- Moves are recorded and can be reversed while the original path remains free.
