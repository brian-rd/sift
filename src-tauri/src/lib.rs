#[cfg(not(target_os = "windows"))]
compile_error!("Sift is a Windows-only desktop application.");

use rusqlite::{params, Connection};
use serde::Serialize;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{Manager, State};
use windows::{
    core::PWSTR,
    Win32::Security::Authentication::Identity::{GetUserNameExW, NameDisplay},
};

struct AppState {
    db: Mutex<Connection>,
}

#[derive(Debug, thiserror::Error)]
enum SiftError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Database(#[from] rusqlite::Error),
    #[error("{0}")]
    Trash(#[from] trash::Error),
    #[error("{0}")]
    Tauri(#[from] tauri::Error),
    #[error("{0}")]
    Message(String),
}

impl Serialize for SiftError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadFile {
    path: String,
    name: String,
    extension: String,
    size: u64,
    modified_at: u64,
    created_at: u64,
    kind: String,
    suggested_folder: Option<String>,
    matched_rule: Option<String>,
    preview_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanResult {
    folder: String,
    files: Vec<DownloadFile>,
    total_bytes: u64,
    skipped_incomplete: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OperationResult {
    operation_id: i64,
    source: String,
    destination: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrashEntry {
    operation_id: i64,
    original_path: String,
    staged_path: String,
    name: String,
    extension: String,
    size: u64,
    modified_at: u64,
    created_at: u64,
    kind: String,
}

#[derive(Serialize)]
struct PinnedDestination {
    name: String,
    path: String,
}

fn unix_millis(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn classify(extension: &str) -> &'static str {
    match extension {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg" => "image",
        "pdf" => "pdf",
        "zip" | "rar" | "7z" | "tar" | "gz" => "archive",
        "mp4" | "mov" | "mkv" | "avi" | "webm" => "video",
        "mp3" | "wav" | "flac" | "m4a" | "ogg" => "audio",
        "txt" | "md" | "csv" | "json" | "log" | "xml" | "yaml" | "yml" => "text",
        _ => "other",
    }
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(true)
}

fn downloads_dir(folder: Option<String>) -> Result<PathBuf, SiftError> {
    let path = match folder {
        Some(value) if !value.trim().is_empty() => PathBuf::from(value),
        _ => dirs::download_dir()
            .ok_or_else(|| SiftError::Message("Downloads folder was not found".into()))?,
    };
    let canonical = path.canonicalize()?;
    if !canonical.is_dir() {
        return Err(SiftError::Message(
            "The watched path is not a folder".into(),
        ));
    }
    Ok(canonical)
}

#[tauri::command]
fn scan_downloads(folder: Option<String>, app: tauri::AppHandle) -> Result<ScanResult, SiftError> {
    let root = downloads_dir(folder)?;
    app.asset_protocol_scope()
        .allow_directory(&root, false)
        .map_err(|error| SiftError::Message(format!("Could not enable previews: {error}")))?;
    let mut files = Vec::new();
    let mut total_bytes = 0;
    let mut skipped_incomplete = 0;

    for entry in fs::read_dir(&root)? {
        let entry = match entry {
            Ok(value) => value,
            Err(_) => continue,
        };
        let path = entry.path();
        let metadata = match fs::symlink_metadata(&path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() || is_hidden(&path) {
            continue;
        }
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_lowercase();
        if matches!(
            extension.as_str(),
            "crdownload" | "part" | "download" | "tmp"
        ) {
            skipped_incomplete += 1;
            continue;
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("Unknown file")
            .to_owned();
        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        if SystemTime::now()
            .duration_since(modified)
            .unwrap_or_default()
            .as_secs()
            < 5
        {
            skipped_incomplete += 1;
            continue;
        }
        total_bytes += metadata.len();
        files.push(DownloadFile {
            path: path.to_string_lossy().into_owned(),
            name,
            extension: extension.clone(),
            size: metadata.len(),
            modified_at: unix_millis(modified),
            created_at: unix_millis(metadata.created().unwrap_or(modified)),
            kind: classify(&extension).to_owned(),
            suggested_folder: None,
            matched_rule: None,
            preview_url: None,
        });
    }
    files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    Ok(ScanResult {
        folder: root.to_string_lossy().into_owned(),
        files,
        total_bytes,
        skipped_incomplete,
    })
}

#[tauri::command]
fn read_text_preview(path: String, app: tauri::AppHandle) -> Result<String, SiftError> {
    const PREVIEW_LIMIT: u64 = 256 * 1024;
    let file = PathBuf::from(path).canonicalize()?;
    let metadata = file.symlink_metadata()?;
    let extension = file
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !metadata.is_file() || metadata.file_type().is_symlink() || classify(&extension) != "text" {
        return Err(SiftError::Message(
            "Only supported text files can be previewed".into(),
        ));
    }
    if !app.asset_protocol_scope().is_allowed(&file) {
        return Err(SiftError::Message(
            "The file is outside the watched folder".into(),
        ));
    }

    let mut bytes = Vec::new();
    fs::File::open(file)?
        .take(PREVIEW_LIMIT)
        .read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn unique_destination(folder: &Path, file_name: &str) -> PathBuf {
    let original = Path::new(file_name);
    let stem = original
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("file");
    let extension = original.extension().and_then(|value| value.to_str());
    let first = folder.join(file_name);
    if !first.exists() {
        return first;
    }
    for number in 2..10_000 {
        let candidate = match extension {
            Some(ext) => folder.join(format!("{stem} ({number}).{ext}")),
            None => folder.join(format!("{stem} ({number})")),
        };
        if !candidate.exists() {
            return candidate;
        }
    }
    folder.join(format!("{stem}-{}", unix_millis(SystemTime::now())))
}

fn move_file(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    fs::rename(source, destination).or_else(|_| {
        fs::copy(source, destination)?;
        if let Err(error) = fs::remove_file(source) {
            let _ = fs::remove_file(destination);
            return Err(error);
        }
        Ok(())
    })
}

fn is_app_undoable(action: &str) -> bool {
    matches!(action, "move" | "stage_trash" | "trash")
}

fn windows_path_key(path: &Path) -> String {
    let value = path.to_string_lossy().replace('/', "\\");
    let value = value
        .strip_prefix(r"\\?\UNC\")
        .map(|path| format!(r"\\{path}"))
        .or_else(|| value.strip_prefix(r"\\?\").map(str::to_owned))
        .unwrap_or(value);
    value.trim_end_matches('\\').to_lowercase()
}

fn find_recycle_item(
    items: Vec<trash::TrashItem>,
    staged_path: &Path,
    recycle_id: Option<&str>,
) -> Option<trash::TrashItem> {
    if let Some(recycle_id) = recycle_id {
        if let Some(item) = items
            .iter()
            .find(|item| item.id.to_string_lossy() == recycle_id)
        {
            return Some(item.clone());
        }
    }

    let staged_key = windows_path_key(staged_path);
    items
        .into_iter()
        .filter(|item| windows_path_key(&item.original_path()) == staged_key)
        .max_by_key(|item| item.time_deleted)
}

fn restore_recycled_file(
    original_path: &Path,
    staged_path: &Path,
    recycle_id: Option<&str>,
) -> Result<(), SiftError> {
    if original_path.exists() {
        return Err(SiftError::Message(
            "The original path is already occupied".into(),
        ));
    }

    if !staged_path.exists() {
        let item = find_recycle_item(trash::os_limited::list()?, staged_path, recycle_id)
            .ok_or_else(|| {
                SiftError::Message("This file is no longer in the Windows Recycle Bin".into())
            })?;
        trash::os_limited::restore_all([item])?;
    }

    if !staged_path.is_file() {
        return Err(SiftError::Message(
            "Windows did not restore the file from the Recycle Bin".into(),
        ));
    }
    move_file(staged_path, original_path)?;
    Ok(())
}

fn trash_entry(
    operation_id: i64,
    original_path: String,
    staged_path: String,
) -> Option<TrashEntry> {
    let path = PathBuf::from(&staged_path);
    let original = PathBuf::from(&original_path);
    let metadata = path.symlink_metadata().ok()?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return None;
    }
    let name = original.file_name()?.to_string_lossy().into_owned();
    let extension = original
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_lowercase();
    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
    Some(TrashEntry {
        operation_id,
        original_path,
        staged_path,
        name,
        extension: extension.clone(),
        size: metadata.len(),
        modified_at: unix_millis(modified),
        created_at: unix_millis(metadata.created().unwrap_or(modified)),
        kind: classify(&extension).to_owned(),
    })
}

fn record_operation(
    db: &Connection,
    action: &str,
    source: &Path,
    destination: Option<&Path>,
    undoable: bool,
) -> Result<i64, SiftError> {
    db.execute(
        "INSERT INTO operations (action, source, destination, created_at, undoable, undone) VALUES (?1, ?2, ?3, ?4, ?5, 0)",
        params![action, source.to_string_lossy(), destination.map(|path| path.to_string_lossy().into_owned()), unix_millis(SystemTime::now()) as i64, undoable],
    )?;
    Ok(db.last_insert_rowid())
}

fn record_moved_operation(
    state: &AppState,
    action: &str,
    source: &Path,
    destination: &Path,
) -> Result<i64, SiftError> {
    let result = state
        .db
        .lock()
        .map_err(|_| SiftError::Message("History database is unavailable".into()))
        .and_then(|db| record_operation(&db, action, source, Some(destination), true));

    match result {
        Ok(operation_id) => Ok(operation_id),
        Err(error) => {
            move_file(destination, source).map_err(|rollback_error| {
                SiftError::Message(format!(
                    "Could not save the action ({error}), and could not restore the file ({rollback_error})"
                ))
            })?;
            Err(error)
        }
    }
}

#[tauri::command]
fn move_download(
    source: String,
    destination: String,
    state: State<'_, AppState>,
) -> Result<OperationResult, SiftError> {
    let source_path = PathBuf::from(&source).canonicalize()?;
    if !source_path.is_file() || source_path.symlink_metadata()?.file_type().is_symlink() {
        return Err(SiftError::Message("Only regular files can be moved".into()));
    }
    let destination_dir = PathBuf::from(destination).canonicalize()?;
    if !destination_dir.is_dir() {
        return Err(SiftError::Message("Destination is not a folder".into()));
    }
    let file_name = source_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| SiftError::Message("Invalid file name".into()))?;
    let target = unique_destination(&destination_dir, file_name);
    move_file(&source_path, &target)?;
    let operation_id = record_moved_operation(&state, "move", &source_path, &target)?;
    Ok(OperationResult {
        operation_id,
        source,
        destination: Some(target.to_string_lossy().into_owned()),
    })
}

#[tauri::command]
fn trash_download(
    path: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<OperationResult, SiftError> {
    let source = PathBuf::from(&path).canonicalize()?;
    if !source.is_file() || source.symlink_metadata()?.file_type().is_symlink() {
        return Err(SiftError::Message(
            "Only regular files can be moved to Sift Trash".into(),
        ));
    }
    let trash_dir = app.path().app_local_data_dir()?.join("trash");
    fs::create_dir_all(&trash_dir)?;
    let file_name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| SiftError::Message("Invalid file name".into()))?;
    let staged = unique_destination(&trash_dir, file_name);
    move_file(&source, &staged)?;
    let operation_id = record_moved_operation(&state, "stage_trash", &source, &staged)?;
    Ok(OperationResult {
        operation_id,
        source: path,
        destination: Some(staged.to_string_lossy().into_owned()),
    })
}

#[tauri::command]
fn list_trash(state: State<'_, AppState>) -> Result<Vec<TrashEntry>, SiftError> {
    let db = state
        .db
        .lock()
        .map_err(|_| SiftError::Message("History database is unavailable".into()))?;
    let mut statement = db.prepare(
        "SELECT id, source, destination FROM operations
         WHERE action = 'stage_trash' AND undone = 0 AND destination IS NOT NULL
         ORDER BY created_at DESC",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let rows = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, source, destination)| trash_entry(id, source, destination))
        .collect())
}

#[tauri::command]
fn finalize_trash(operation_id: i64, state: State<'_, AppState>) -> Result<(), SiftError> {
    let (action, destination, undone): (String, Option<String>, bool) = {
        let db = state
            .db
            .lock()
            .map_err(|_| SiftError::Message("History database is unavailable".into()))?;
        db.query_row(
            "SELECT action, destination, undone FROM operations WHERE id = ?1",
            [operation_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?
    };
    if action != "stage_trash" || undone {
        return Err(SiftError::Message(
            "This Trash item is no longer pending".into(),
        ));
    }
    let staged = PathBuf::from(
        destination.ok_or_else(|| SiftError::Message("Trash location is missing".into()))?,
    );
    let previous_items = trash::os_limited::list().unwrap_or_default();
    trash::delete(&staged)?;
    let recycle_id = trash::os_limited::list().ok().and_then(|items| {
        let staged_key = windows_path_key(&staged);
        items
            .into_iter()
            .filter(|item| windows_path_key(&item.original_path()) == staged_key)
            .filter(|item| !previous_items.iter().any(|previous| previous.id == item.id))
            .max_by_key(|item| item.time_deleted)
            .map(|item| item.id.to_string_lossy().into_owned())
    });
    let db = state
        .db
        .lock()
        .map_err(|_| SiftError::Message("History database is unavailable".into()))?;
    db.execute(
        "UPDATE operations SET action = 'trash', undoable = 1, recycle_id = ?2 WHERE id = ?1",
        params![operation_id, recycle_id],
    )?;
    Ok(())
}

#[tauri::command]
fn undo_operation(operation_id: i64, state: State<'_, AppState>) -> Result<(), SiftError> {
    let (action, source, destination, recycle_id, undone): (
        String,
        String,
        Option<String>,
        Option<String>,
        bool,
    ) = {
        let db = state
            .db
            .lock()
            .map_err(|_| SiftError::Message("History database is unavailable".into()))?;
        db.query_row(
            "SELECT action, source, destination, recycle_id, undone FROM operations WHERE id = ?1",
            [operation_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )?
    };
    if undone {
        return Err(SiftError::Message("This action was already undone".into()));
    }
    if action == "stage_trash" {
        let from = PathBuf::from(
            destination.ok_or_else(|| SiftError::Message("Trash location is missing".into()))?,
        );
        let to = PathBuf::from(&source);
        if to.exists() {
            return Err(SiftError::Message(
                "The original path is already occupied".into(),
            ));
        }
        move_file(&from, &to)?;
        let db = state
            .db
            .lock()
            .map_err(|_| SiftError::Message("History database is unavailable".into()))?;
        db.execute(
            "UPDATE operations SET undone = 1 WHERE id = ?1",
            [operation_id],
        )?;
        return Ok(());
    }
    if action == "trash" {
        let staged = PathBuf::from(
            destination.ok_or_else(|| SiftError::Message("Trash location is missing".into()))?,
        );
        restore_recycled_file(Path::new(&source), &staged, recycle_id.as_deref())?;
        let db = state
            .db
            .lock()
            .map_err(|_| SiftError::Message("History database is unavailable".into()))?;
        db.execute(
            "UPDATE operations SET undone = 1 WHERE id = ?1",
            [operation_id],
        )?;
        return Ok(());
    }
    if !is_app_undoable(&action) {
        return Err(SiftError::Message("This action cannot be undone".into()));
    }
    let from = PathBuf::from(
        destination.ok_or_else(|| SiftError::Message("Move destination is missing".into()))?,
    );
    let to = PathBuf::from(source);
    if to.exists() {
        return Err(SiftError::Message(
            "The original path is already occupied".into(),
        ));
    }
    move_file(&from, &to)?;
    let db = state
        .db
        .lock()
        .map_err(|_| SiftError::Message("History database is unavailable".into()))?;
    db.execute(
        "UPDATE operations SET undone = 1 WHERE id = ?1",
        [operation_id],
    )?;
    Ok(())
}

#[tauri::command]
fn reveal_download(path: String) -> Result<(), SiftError> {
    let file = PathBuf::from(path).canonicalize()?;
    Command::new("explorer")
        .arg(format!("/select,{}", file.display()))
        .spawn()?;
    Ok(())
}

#[tauri::command]
fn open_download(path: String) -> Result<(), SiftError> {
    let file = PathBuf::from(path).canonicalize()?;
    if !file.is_file() || file.symlink_metadata()?.file_type().is_symlink() {
        return Err(SiftError::Message(
            "Only regular files can be opened".into(),
        ));
    }
    Command::new("explorer").arg(file).spawn()?;
    Ok(())
}

#[tauri::command]
fn user_display_name() -> String {
    let mut length = 0u32;
    unsafe { GetUserNameExW(NameDisplay, None, &mut length) };
    if length > 0 {
        let mut buffer = vec![0u16; length as usize];
        if unsafe { GetUserNameExW(NameDisplay, Some(PWSTR(buffer.as_mut_ptr())), &mut length) } {
            let end = buffer
                .iter()
                .position(|value| *value == 0)
                .unwrap_or(buffer.len());
            if let Some(first_name) = String::from_utf16_lossy(&buffer[..end])
                .split_whitespace()
                .next()
            {
                if !first_name.is_empty() {
                    return first_name.to_owned();
                }
            }
        }
    }
    std::env::var("USERNAME")
        .ok()
        .filter(|name| !name.trim().is_empty())
        .map(|name| {
            let mut characters = name.chars();
            characters
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + characters.as_str())
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

#[tauri::command]
fn open_recycle_bin() -> Result<(), SiftError> {
    Command::new("explorer")
        .arg("shell:RecycleBinFolder")
        .spawn()?;
    Ok(())
}

#[tauri::command]
fn default_destinations() -> Vec<PinnedDestination> {
    [
        ("Documents", dirs::document_dir()),
        ("Pictures", dirs::picture_dir()),
        ("Music", dirs::audio_dir()),
        ("Videos", dirs::video_dir()),
    ]
    .into_iter()
    .filter_map(|(name, path)| {
        path.map(|path| PinnedDestination {
            name: name.to_owned(),
            path: path.to_string_lossy().into_owned(),
        })
    })
    .collect()
}

fn initialise_database(path: &Path) -> Result<Connection, rusqlite::Error> {
    let connection = Connection::open(path)?;
    connection.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS operations (
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           action TEXT NOT NULL,
           source TEXT NOT NULL,
           destination TEXT,
           recycle_id TEXT,
           created_at INTEGER NOT NULL,
           undoable INTEGER NOT NULL DEFAULT 0,
           undone INTEGER NOT NULL DEFAULT 0
         );",
    )?;
    let has_recycle_id = {
        let mut statement = connection.prepare("PRAGMA table_info(operations)")?;
        let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
        let mut found = false;
        for column in columns {
            if column? == "recycle_id" {
                found = true;
                break;
            }
        }
        found
    };
    if !has_recycle_id {
        connection.execute("ALTER TABLE operations ADD COLUMN recycle_id TEXT", [])?;
    }
    connection.execute(
        "UPDATE operations SET undoable = 1 WHERE action = 'trash' AND undone = 0 AND destination IS NOT NULL",
        [],
    )?;
    Ok(connection)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&data_dir)?;
            let db = initialise_database(&data_dir.join("sift.db"))?;
            app.manage(AppState { db: Mutex::new(db) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_downloads,
            read_text_preview,
            move_download,
            trash_download,
            list_trash,
            finalize_trash,
            undo_operation,
            reveal_download,
            open_download,
            open_recycle_bin,
            user_display_name,
            default_destinations
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sift");
}

#[cfg(test)]
mod tests {
    use super::{
        find_recycle_item, is_app_undoable, restore_recycled_file, unix_millis, windows_path_key,
    };
    use std::{fs, path::Path, time::SystemTime};

    #[test]
    fn staged_trash_and_moves_are_undoable_in_sift() {
        assert!(is_app_undoable("stage_trash"));
        assert!(is_app_undoable("move"));
        assert!(is_app_undoable("trash"));
    }

    #[test]
    fn recycle_paths_ignore_extended_prefix_and_case() {
        assert_eq!(
            windows_path_key(Path::new(r"\\?\C:\Users\Brian\Downloads\File.txt")),
            windows_path_key(Path::new(r"c:\users\brian\downloads\file.txt"))
        );
    }

    #[test]
    #[ignore = "temporarily uses the Windows Recycle Bin"]
    fn recycled_file_round_trip_restores_the_original_path() {
        let root = std::env::temp_dir().join(format!(
            "sift-recycle-restore-{}",
            unix_millis(SystemTime::now())
        ));
        fs::create_dir(&root).expect("create recycle restore test folder");
        let staged = root.join("staged.txt");
        let original = root.join("original.txt");
        fs::write(&staged, "Sift recycle restore test").expect("create recycle restore test file");

        trash::delete(&staged).expect("send test file to Recycle Bin");
        let recycled = find_recycle_item(
            trash::os_limited::list().expect("list Recycle Bin"),
            &staged,
            None,
        )
        .expect("find the recycled test file");
        let recycle_id = recycled.id.to_string_lossy().into_owned();

        restore_recycled_file(&original, &staged, Some(&recycle_id))
            .expect("restore test file through Sift");
        assert_eq!(
            fs::read_to_string(&original).expect("read restored test file"),
            "Sift recycle restore test"
        );

        fs::remove_file(original).expect("remove restored test file");
        fs::remove_dir(root).expect("remove recycle restore test folder");
    }
}
