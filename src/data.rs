use std::sync::{RwLock};
use once_cell::sync::Lazy;
use std::{path::PathBuf};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

use crate::entry::Entry;

pub fn get_application_dirs() -> Vec<PathBuf> {
    let dirs_string = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
    
    dirs_string.split(':').map(PathBuf::from).collect()
}

pub fn get_desktop_entry_paths() -> Vec<PathBuf> {
    let dirs = get_application_dirs();

    let desktop_entries: Vec<PathBuf> = dirs.par_iter().map(|dir|{
        let apps_dir = dir.join("applications");
        let mut desktop_entries = Vec::new();
        if apps_dir.exists() && apps_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(apps_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        desktop_entries.push(path);
                    }
                }
            }
        }
        desktop_entries
    }).flatten().collect();

    desktop_entries
}

pub fn get_desktop_entries() -> Vec<DesktopEntry> {
    let paths = get_desktop_entry_paths();

    let entries: Vec<DesktopEntry> = paths.par_iter().filter_map(|path|{
        match parse_desktop_entry(path) {
            Ok(entry) => {
                let app_type = entry.entry.get("Desktop Entry", "Type")
                    .map(|e| e.first())
                    .flatten()
                    .map(|e| e.as_str());

                if app_type != Some("Application") {
                    return None;
                }
                Some(entry)
            },
            Err(_) => {
                None
            }
        }
    }).collect();

    entries
}

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    path: PathBuf,
    entry: freedesktop_entry_parser::Entry,
}

impl Entry for DesktopEntry {
    fn id(&self) -> &str {
        self.path.to_str().unwrap_or_default()
    }
    fn title(&self) -> &str {
        let Some(str) = self.entry.get("Desktop Entry", "Name").map(|e|e.first()).flatten() else {
            return "Unnamed";
        };

        str.as_str()
    }

    fn description(&self) -> Option<&str> {
        self.entry.get("Desktop Entry", "Comment")
            .map(|e|e.first())
            .flatten()
            .map(|e|e.as_str())
    }
}

pub fn parse_desktop_entry(path: &PathBuf) -> Result<DesktopEntry, anyhow::Error> {
    let entry = freedesktop_entry_parser::parse_entry(path)?;
    Ok(DesktopEntry { entry, path: path.clone() })
}

#[derive(Debug)]
pub struct AppData {
    pub entries: Vec<DesktopEntry>,
}

pub static APP_DATA: Lazy<RwLock<AppData>> = Lazy::new(|| RwLock::new(AppData {
    entries: Vec::new(),
}));

pub fn load_app_data() {
    let entries = get_desktop_entries();
    match APP_DATA.write() {
        Ok(mut lock) => {
            lock.entries = entries;
        }
        Err(_) => {}
    }
}
