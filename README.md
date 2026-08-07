# Sift

Sift is a Windows-only Downloads organiser built around two workflows: safe automatic rules for predictable files and a rapid, keyboard-first Sift queue for everything else.

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

The desktop build scans the selected folder without changing it. Moves use collision-safe names, Trash is staged in Sift for review, and operations are recorded in a local SQLite database.

## Sifting files

The default shortcuts form a four-direction layout:

- Up keeps the file in the watched folder.
- Down stages it in Trash for review.
- Left undoes the previous action and returns to that file.
- Right files it away, using a suggestion, a quick-destination menu, or the Windows folder picker.

Keep, Trash, and File Away advance immediately. Held keys are ignored, shortcuts can be rebound in Settings, and pinned destinations remain visible with number-key shortcuts. Sift follows the Windows light or dark theme by default.

The Windows app previews supported images, PDFs, videos, audio, text, and rendered Markdown locally. Preview access is restricted to the selected watched folder and text previews are limited to 256 KB. The preview toolbar can either reveal a file in Explorer or open it in its default Windows app.

## Safety model

- Scans skip hidden files, directories, symbolic links, and incomplete downloads.
- Rule runs require a file-by-file preview and explicit confirmation.
- A move never overwrites an existing file.
- Trash first moves files into Sift's private staging folder. Leaving Sift or returning to Overview opens a review list where files can be restored individually or in a selection, or sent on to the Windows Recycle Bin.
- Once a file is sent to the Windows Recycle Bin, Sift no longer offers Undo for it. History opens the Recycle Bin so Windows can handle any final restoration manually.
- Moves are recorded and can be reversed while the original path remains free.
