/**
 * Core Application Logic.
 *
 * Orchestrates the Tauri backend, managing plugin initialization
 * and command registration for frontend integration.
 *
 * BACKEND ARCHITECTURE:
 * ┌───────────────┐      ┌───────────────┐      ┌───────────────┐
 * │  Tauri Core   │ ◄─── │ Command Disp  │ ◄─── │ IPC (Invoke)  │
 * └───────┬───────┘      └───────┬───────┘      └───────┬───────┘
 *         │                      │                      │
 * ┌───────▼───────┐      ┌───────▼───────┐      ┌───────▼───────┐
 * │ Plugin Init   │      │ Clipboard Mgr │      │ Opener Plugin │
 * └───────────────┘      └───────────────┘      └───────────────┘
 *
 * INTEGRATION STRATEGY:
 * - Commands are registered via generate_handler for secure IPC.
 * - ClipboardManager plugin provides cross-platform text buffer access.
 * - Error states are bubbled up as Result values to the frontend.
 *
 * CONTRACTS & GUARANTEES:
 * - Clipboard access is thread-safe via AppHandle.
 * - Command execution is non-blocking on the main thread where possible.
 *
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 */

#[derive(serde::Serialize, serde::Deserialize)]
#[allow(non_snake_case)]
struct KaomojiEntry {
	Character: String,
	Tags: Vec<String>,
	Category: String,
}

#[tauri::command]
#[allow(non_snake_case)]
fn Greet(name: &str) -> String {
	format!("Hello, {}! from crablang!", name)
}

#[tauri::command]
#[allow(non_snake_case)]
fn CopyToClipboard(app: tauri::AppHandle, text: String) -> Result<(), String> {
	use tauri_plugin_clipboard_manager::ClipboardExt;
	app.clipboard().write_text(text).map_err(|e| e.to_string())
}

#[allow(non_snake_case)]
fn BackupCorrupt(filePath: &std::path::Path) {
	use std::time::{SystemTime, UNIX_EPOCH};
	if let Ok(ts) = SystemTime::now().duration_since(UNIX_EPOCH) {
		let fileName = filePath
			.file_name()
			.and_then(|s| s.to_str())
			.unwrap_or("kaomojis.user.json");

		let backup = filePath.with_file_name(format!("{fileName}.corrupt.{}.bak", ts.as_secs()));
		
		// ROBUST BACKUP STRATEGY: Try rename, fallback to copy
		if let Err(renameErr) = std::fs::rename(filePath, &backup) {
			if let Ok(content) = std::fs::read(filePath) {
				if let Ok(_) = std::fs::write(&backup, content) {
					if let Err(removeErr) = std::fs::remove_file(filePath) {
						eprintln!("Kaomoji file corrupt; backed up via copy to {:?}, but failed to remove original: {}", backup, removeErr);
					} else {
						eprintln!("Kaomoji file corrupt; backed up via copy to {:?}", backup);
					}
					return;
				}
			}
			eprintln!("Kaomoji file corrupt; failed to back up: {renameErr}. Path: {:?}", filePath);
		} else {
			eprintln!("Kaomoji file corrupt; backed up to {:?}", backup);
		}
	}
}

/**
 * ATOMIC REPLACEMENT STRATEGY
 * 
 * Ensures that file writes are individual and complete. 
 * On Windows, uses ReplaceFileW for atomicity when replacing existing files.
 * Falls back to MoveFileExW with durability flags.
 */
#[allow(non_snake_case)]
fn AtomicSave(filePath: &std::path::Path, content: &str) -> Result<(), String> {
	use std::{fs, io::Write, time::{SystemTime, UNIX_EPOCH}};

	// Unique temp name to prevent race conditions
	let ts = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|d| d.as_nanos())
		.unwrap_or(0);
	let pid = std::process::id();
	let fileName = filePath
		.file_name()
		.and_then(|s| s.to_str())
		.unwrap_or("kaomojis.user.json");

	let tmpName = format!("{}.{}.{}.tmp", fileName, pid, ts);
	let tmpPath = filePath.with_file_name(tmpName);
	
	// DURABILITY STRATEGY: Write + Flush
	{
		let mut f = fs::File::create(&tmpPath).map_err(|e| e.to_string())?;
		f.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
		f.sync_all().map_err(|e| e.to_string())?;
	}

	#[cfg(windows)]
	let saveResult = {
		use std::os::windows::ffi::OsStrExt;
		use windows_sys::Win32::Storage::FileSystem::{
			ReplaceFileW, MoveFileExW,
			MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
			REPLACEFILE_WRITE_THROUGH,
		};
		
		let destU16: Vec<u16> = filePath.as_os_str().encode_wide().chain(Some(0)).collect();
		let tmpU16: Vec<u16> = tmpPath.as_os_str().encode_wide().chain(Some(0)).collect();
		
		unsafe {
			if filePath.exists() {
				// Atomic replace for existing dest
				let ok = ReplaceFileW(
					destU16.as_ptr(),
					tmpU16.as_ptr(),
					std::ptr::null(),
					REPLACEFILE_WRITE_THROUGH,
					std::ptr::null_mut(),
					std::ptr::null_mut(),
				);
				if ok == 0 {
					let err = std::io::Error::last_os_error();
					// Fallback: MoveFileExW with replace existing + write through
					let ok2 = MoveFileExW(
						tmpU16.as_ptr(),
						destU16.as_ptr(),
						MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
					);
					if ok2 == 0 {
						let err2 = std::io::Error::last_os_error();
						Err(format!("ReplaceFileW failed ({err}); MoveFileExW failed ({err2})"))
					} else {
						Ok(())
					}
				} else {
					Ok(())
				}
			} else {
				// New dest: MoveFileExW also fine
				let ok = MoveFileExW(
					tmpU16.as_ptr(),
					destU16.as_ptr(),
					MOVEFILE_WRITE_THROUGH,
				);
				if ok == 0 {
					let err = std::io::Error::last_os_error();
					Err(format!("MoveFileExW failed ({err})"))
				} else {
					Ok(())
				}
			}
		}
	};

	#[cfg(not(windows))]
	let saveResult = {
		// Unix-like systems: rename is atomic and replaces destination by default
		fs::rename(&tmpPath, filePath).map_err(|e| e.to_string()).and_then(|_| {
			// STRATEGY: Directory Durability (Metadata Sync)
			if let Some(parent) = filePath.parent() {
				if let Ok(dir) = fs::File::open(parent) {
					let _ = dir.sync_all(); // Best effort metadata flush
				}
			}
			Ok(())
		})
	};

	// CLEANUP: Always attempt to remove the unique temp file
	let _ = fs::remove_file(&tmpPath);

	saveResult
}

#[tauri::command]
#[allow(non_snake_case)]
fn LoadUserKaomojis(app: tauri::AppHandle) -> Result<Vec<KaomojiEntry>, String> {
	use tauri::Manager;
	use std::fs;

	// BINDING CONTRACT: Ensure dataDir exists for consistency
	let dataDir = app.path().app_data_dir().map_err(|e| e.to_string())?;
	if !dataDir.exists() {
		fs::create_dir_all(&dataDir).map_err(|e| e.to_string())?;
	}

	let filePath = dataDir.join("kaomojis.user.json");
	if !filePath.exists() {
		return Ok(Vec::new());
	}

	let content = fs::read_to_string(&filePath).map_err(|e| e.to_string())?;
	
	// SAFE CORRUPTION HANDLING
	let list: Vec<KaomojiEntry> = match serde_json::from_str(&content) {
		Ok(v) => v,
		Err(_) => {
			BackupCorrupt(&filePath);
			Vec::new()
		}
	};
	
	Ok(list)
}

#[tauri::command]
#[allow(non_snake_case)]
fn SaveKaomoji(app: tauri::AppHandle, mut newEntry: KaomojiEntry) -> Result<(), String> {
	use tauri::Manager;
	use std::fs;

	// STRATEGY: Deep Sanitization
	newEntry.Character = newEntry.Character.trim().to_string();
	if newEntry.Character.is_empty() {
		return Err("Kaomoji character cannot be empty".to_string());
	}

	newEntry.Tags = newEntry
		.Tags
		.into_iter()
		.map(|t| t.trim().to_lowercase())
		.filter(|t| !t.is_empty())
		.collect();

	newEntry.Category = newEntry.Category.trim().to_string();

	let dataDir = app.path().app_data_dir().map_err(|e| e.to_string())?;
	if !dataDir.exists() {
		fs::create_dir_all(&dataDir).map_err(|e| e.to_string())?;
	}

	let filePath = dataDir.join("kaomojis.user.json");
	let mut list: Vec<KaomojiEntry> = if filePath.exists() {
		let content = fs::read_to_string(&filePath).map_err(|e| e.to_string())?;
		
		match serde_json::from_str(&content) {
			Ok(v) => v,
			Err(_) => {
				BackupCorrupt(&filePath);
				Vec::new()
			}
		}
	} else {
		Vec::new()
	};

	// Uart-style Upsert: Update if exists, else push
	let exists = list.iter_mut().find(|k| k.Character == newEntry.Character);
	match exists {
		Some(item) => {
			item.Tags = newEntry.Tags;
			item.Category = newEntry.Category;
		}
		None => list.push(newEntry),
	}

	let updatedContent = serde_json::to_string_pretty(&list).map_err(|e| e.to_string())?;
	AtomicSave(&filePath, &updatedContent)?;

	Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
fn LoadRecents(app: tauri::AppHandle) -> Result<Vec<KaomojiEntry>, String> {
	use tauri::Manager;
	use std::fs;

	let dataDir = app.path().app_data_dir().map_err(|e| e.to_string())?;
	if !dataDir.exists() {
		fs::create_dir_all(&dataDir).map_err(|e| e.to_string())?;
	}

	let filePath = dataDir.join("kaomojis.recents.json");
	if !filePath.exists() {
		return Ok(Vec::new());
	}

	let content = fs::read_to_string(&filePath).map_err(|e| e.to_string())?;
	
	let list: Vec<KaomojiEntry> = match serde_json::from_str(&content) {
		Ok(v) => v,
		Err(_) => {
			BackupCorrupt(&filePath);
			Vec::new()
		}
	};
	
	Ok(list)
}

#[tauri::command]
#[allow(non_snake_case)]
fn SaveRecent(app: tauri::AppHandle, entry: KaomojiEntry) -> Result<(), String> {
	use tauri::Manager;
	use std::fs;

	const MAX_RECENTS: usize = 20;

	let dataDir = app.path().app_data_dir().map_err(|e| e.to_string())?;
	if !dataDir.exists() {
		fs::create_dir_all(&dataDir).map_err(|e| e.to_string())?;
	}

	let filePath = dataDir.join("kaomojis.recents.json");
	let mut list: Vec<KaomojiEntry> = if filePath.exists() {
		let content = fs::read_to_string(&filePath).map_err(|e| e.to_string())?;
		
		match serde_json::from_str(&content) {
			Ok(v) => v,
			Err(_) => {
				BackupCorrupt(&filePath);
				Vec::new()
			}
		}
	} else {
		Vec::new()
	};

	// Remove any existing instance of this kaomoji (by Character)
	list.retain(|k| k.Character != entry.Character);
	
	// Add to front of list
	list.insert(0, entry);
	
	// Trim to max size
	list.truncate(MAX_RECENTS);

	let updatedContent = serde_json::to_string_pretty(&list).map_err(|e| e.to_string())?;
	AtomicSave(&filePath, &updatedContent)?;

	Ok(())
}

#[tauri::command]
#[allow(non_snake_case)]
fn LoadFavorites(app: tauri::AppHandle) -> Result<Vec<KaomojiEntry>, String> {
	use tauri::Manager;
	use std::fs;

	let dataDir = app.path().app_data_dir().map_err(|e| e.to_string())?;
	if !dataDir.exists() {
		fs::create_dir_all(&dataDir).map_err(|e| e.to_string())?;
	}

	let filePath = dataDir.join("kaomojis.favorites.json");
	if !filePath.exists() {
		return Ok(Vec::new());
	}

	let content = fs::read_to_string(&filePath).map_err(|e| e.to_string())?;
	
	let list: Vec<KaomojiEntry> = match serde_json::from_str(&content) {
		Ok(v) => v,
		Err(_) => {
			BackupCorrupt(&filePath);
			Vec::new()
		}
	};
	
	Ok(list)
}

#[tauri::command]
#[allow(non_snake_case)]
fn ToggleFavorite(app: tauri::AppHandle, entry: KaomojiEntry) -> Result<bool, String> {
	use tauri::Manager;
	use std::fs;

	let dataDir = app.path().app_data_dir().map_err(|e| e.to_string())?;
	if !dataDir.exists() {
		fs::create_dir_all(&dataDir).map_err(|e| e.to_string())?;
	}

	let filePath = dataDir.join("kaomojis.favorites.json");
	let mut list: Vec<KaomojiEntry> = if filePath.exists() {
		let content = fs::read_to_string(&filePath).map_err(|e| e.to_string())?;
		
		match serde_json::from_str(&content) {
			Ok(v) => v,
			Err(_) => {
				BackupCorrupt(&filePath);
				Vec::new()
			}
		}
	} else {
		Vec::new()
	};

	// Check if entry exists (by Character)
	let existingIndex = list.iter().position(|k| k.Character == entry.Character);
	
	let isFavorite = if let Some(idx) = existingIndex {
		// Remove from favorites
		list.remove(idx);
		false
	} else {
		// Add to favorites
		list.push(entry);
		true
	};

	let updatedContent = serde_json::to_string_pretty(&list).map_err(|e| e.to_string())?;
	AtomicSave(&filePath, &updatedContent)?;

	Ok(isFavorite)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(non_snake_case)]
pub fn Run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_clipboard_manager::init())
		.plugin(tauri_plugin_opener::init())
		.invoke_handler(tauri::generate_handler![Greet, CopyToClipboard, SaveKaomoji, LoadUserKaomojis, LoadRecents, SaveRecent, LoadFavorites, ToggleFavorite])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
