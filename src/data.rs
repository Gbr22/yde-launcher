use std::sync::{RwLock};
use once_cell::sync::Lazy;
use std::{path::PathBuf};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

use crate::entry::Entry;

pub fn get_data_dirs() -> Vec<PathBuf> {
    let dirs_string = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
    
    dirs_string.split(':').map(PathBuf::from).collect()
}

pub fn get_desktop_entry_paths() -> Vec<PathBuf> {
    let dirs = get_data_dirs();

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
                let no_display = entry.entry.get("Desktop Entry", "NoDisplay")
                    .map(|e| e.iter().filter(|t| t.as_str() == "true").collect::<Vec<_>>().len() > 0).unwrap_or(false);

                if no_display {
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
    icon: Option<PathBuf>,
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
    
    fn icon_path(&self) -> Option<&PathBuf> {
        self.icon.as_ref()
    }
}

pub fn parse_desktop_entry(path: &PathBuf) -> Result<DesktopEntry, anyhow::Error> {
    let entry = freedesktop_entry_parser::parse_entry(path)?;
    let icon_value = entry.get("Desktop Entry", "Icon")
        .map(|e| e.first())
        .flatten()
        .map(|e| e.as_str());

    let icon_path: Option<PathBuf> = icon_value.map(|icon_value| {
        let icon_path = PathBuf::from(icon_value);

        if icon_path.is_absolute() {
            Some(icon_path)
        } else {
            let data_dirs = get_data_dirs();
            for dir in data_dirs {
                let resolutions = vec![
                    "256x256",
                    "128x128",
                    "64x64",
                    "48x48",
                    "32x32",
                    "24x24",
                    "16x16",
                ];
                for resolution in resolutions {
                    let possible_path = dir.join(format!("icons/hicolor/{}/apps/{}.png", resolution, icon_value));
                    if possible_path.exists() {
                        return Some(possible_path);
                    }
                }
            }
            None
        }
    }).flatten();

    Ok(DesktopEntry { entry, path: path.clone(), icon: icon_path })
}
