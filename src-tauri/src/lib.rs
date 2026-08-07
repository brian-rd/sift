#[cfg(not(target_os = "windows"))]
compile_error!("Sift is a Windows-only desktop application.");

use rusqlite::{params, Connection};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{Manager, State};

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

fn unix_millis(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
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
        _ => dirs::download_dir().ok_or_else(|| SiftError::Message("Downloads folder was not found".into()))?,
    };
    let canonical = path.canonicalize()?;
    if !canonical.is_dir() {
        return Err(SiftError::Message("The watched path is not a folder".into()));
    }
    Ok(canonical)
}

#[tauri::command]
fn scan_downloads(folder: Option<String>) -> Result<ScanResult, SiftError> {
    let root = downloads_dir(folder)?;
    let mut files = Vec::new();
    let mut total_bytes = 0;
    let mut skipped_incomplete = 0;

    for entry in fs::read_dir(&root)? {
        let entry = match entry { Ok(value) => value, Err(_) => continue };
        let path = entry.path();
        let metadata = match fs::symlink_metadata(&path) { Ok(value) => value, Err(_) => continue };
        if !metadata.is_file() || metadata.file_type().is_symlink() || is_hidden(&path) { continue; }
        let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("").to_lowercase();
        if matches!(extension.as_str(), "crdownload" | "part" | "download" | "tmp") {
            skipped_incomplete += 1;
            continue;
        }
        let name = path.file_name().and_then(|value| value.to_str()).unwrap_or("Unknown file").to_owned();
        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        if SystemTime::now().duration_since(modified).unwrap_or_default().as_secs() < 5 {
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
            kind: classify(&extension).to_owned(),
            suggested_folder: None,
            matched_rule: None,
            preview_url: None,
        });
    }
    files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    Ok(ScanResult { folder: root.to_string_lossy().into_owned(), files, total_bytes, skipped_incomplete })
}

fn unique_destination(folder: &Path, file_name: &str) -> PathBuf {
    let original = Path::new(file_name);
    let stem = original.file_stem().and_then(|value| value.to_str()).unwrap_or("file");
    let extension = original.extension().and_then(|value| value.to_str());
    let first = folder.join(file_name);
    if !first.exists() { return first; }
    for number in 2..10_000 {
        let candidate = match extension {
            Some(ext) => folder.join(format!("{stem} ({number}).{ext}")),
            None => folder.join(format!("{stem} ({number})")),
        };
        if !candidate.exists() { return candidate; }
    }
    folder.join(format!("{stem}-{}", unix_millis(SystemTime::now())))
}

fn record_operation(db: &Connection, action: &str, source: &Path, destination: Option<&Path>, undoable: bool) -> Result<i64, SiftError> {
    db.execute(
        "INSERT INTO operations (action, source, destination, created_at, undoable, undone) VALUES (?1, ?2, ?3, ?4, ?5, 0)",
        params![action, source.to_string_lossy(), destination.map(|path| path.to_string_lossy().into_owned()), unix_millis(SystemTime::now()) as i64, undoable],
    )?;
    Ok(db.last_insert_rowid())
}

#[tauri::command]
fn move_download(source: String, destination: String, state: State<'_, AppState>) -> Result<OperationResult, SiftError> {
    let source_path = PathBuf::from(&source).canonicalize()?;
    if !source_path.is_file() || source_path.symlink_metadata()?.file_type().is_symlink() {
        return Err(SiftError::Message("Only regular files can be moved".into()));
    }
    let destination_dir = PathBuf::from(destination).canonicalize()?;
    if !destination_dir.is_dir() { return Err(SiftError::Message("Destination is not a folder".into())); }
    let file_name = source_path.file_name().and_then(|value| value.to_str()).ok_or_else(|| SiftError::Message("Invalid file name".into()))?;
    let target = unique_destination(&destination_dir, file_name);
    fs::rename(&source_path, &target).or_else(|_| {
        fs::copy(&source_path, &target)?;
        fs::remove_file(&source_path)
    })?;
    let db = state.db.lock().map_err(|_| SiftError::Message("History database is unavailable".into()))?;
    let operation_id = record_operation(&db, "move", &source_path, Some(&target), true)?;
    Ok(OperationResult { operation_id, source, destination: Some(target.to_string_lossy().into_owned()) })
}

#[tauri::command]
fn trash_download(path: String, state: State<'_, AppState>) -> Result<OperationResult, SiftError> {
    let source = PathBuf::from(&path).canonicalize()?;
    if !source.is_file() || source.symlink_metadata()?.file_type().is_symlink() {
        return Err(SiftError::Message("Only regular files can be sent to Trash".into()));
    }
    trash::delete(&source)?;
    let db = state.db.lock().map_err(|_| SiftError::Message("History database is unavailable".into()))?;
    let operation_id = record_operation(&db, "trash", &source, None, true)?;
    Ok(OperationResult { operation_id, source: path, destination: None })
}

#[tauri::command]
fn undo_operation(operation_id: i64, state: State<'_, AppState>) -> Result<(), SiftError> {
    let db = state.db.lock().map_err(|_| SiftError::Message("History database is unavailable".into()))?;
    let (action, source, destination, undone): (String, String, Option<String>, bool) = db.query_row(
        "SELECT action, source, destination, undone FROM operations WHERE id = ?1",
        [operation_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?;
    if undone { return Err(SiftError::Message("This action was already undone".into())); }
    if action == "trash" {
        let source_path = PathBuf::from(&source);
        let item = trash::os_limited::list()?
            .into_iter()
            .filter(|item| item.original_path() == source_path)
            .max_by_key(|item| item.time_deleted)
            .ok_or_else(|| SiftError::Message("The file is no longer in the Recycle Bin".into()))?;
        trash::os_limited::restore_all([item])?;
        db.execute("UPDATE operations SET undone = 1 WHERE id = ?1", [operation_id])?;
        return Ok(());
    }
    if action != "move" { return Err(SiftError::Message("This action cannot be undone".into())); }
    let from = PathBuf::from(destination.ok_or_else(|| SiftError::Message("Move destination is missing".into()))?);
    let to = PathBuf::from(source);
    if to.exists() { return Err(SiftError::Message("The original path is already occupied".into())); }
    fs::rename(from, to)?;
    db.execute("UPDATE operations SET undone = 1 WHERE id = ?1", [operation_id])?;
    Ok(())
}

#[tauri::command]
fn reveal_download(path: String) -> Result<(), SiftError> {
    let file = PathBuf::from(path).canonicalize()?;
    Command::new("explorer").arg(format!("/select,{}", file.display())).spawn()?;
    Ok(())
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
           created_at INTEGER NOT NULL,
           undoable INTEGER NOT NULL DEFAULT 0,
           undone INTEGER NOT NULL DEFAULT 0
         );
         CREATE TABLE IF NOT EXISTS rules (
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           name TEXT NOT NULL,
           condition_type TEXT NOT NULL,
           condition_value TEXT NOT NULL,
           action_type TEXT NOT NULL,
           destination TEXT,
           priority INTEGER NOT NULL,
           enabled INTEGER NOT NULL DEFAULT 1
         );"
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
        .invoke_handler(tauri::generate_handler![scan_downloads, move_download, trash_download, undo_operation, reveal_download])
        .run(tauri::generate_context!())
        .expect("error while running Sift");
}
