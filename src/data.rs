use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::path::PathBuf;

use crate::entry::Entry;

pub fn get_data_dirs() -> Vec<PathBuf> {
    let dirs_string = std::env::var("XDG_DATA_DIRS").unwrap_or_default();

    dirs_string.split(':').map(PathBuf::from).collect()
}

pub fn get_desktop_entry_paths() -> Vec<PathBuf> {
    let dirs = get_data_dirs();

    let desktop_entries: Vec<PathBuf> = dirs
        .par_iter()
        .map(|dir| {
            let apps_dir = dir.join("applications");
            let mut desktop_entries = Vec::new();
            if apps_dir.exists()
                && apps_dir.is_dir()
                && let Ok(entries) = std::fs::read_dir(apps_dir)
            {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        desktop_entries.push(path);
                    }
                }
            }
            desktop_entries
        })
        .flatten()
        .collect();

    desktop_entries
}

pub fn get_desktop_entries() -> Vec<Entry> {
    let paths = get_desktop_entry_paths();

    let entries: Vec<Entry> = paths
        .par_iter()
        .filter_map(|path| match parse_desktop_entry(path) {
            Ok(entry) => {
                let no_display = entry
                    .entry
                    .get("Desktop Entry", "NoDisplay")
                    .map(|e| {
                        !e.iter()
                            .filter(|t| t.as_str() == "true")
                            .collect::<Vec<_>>()
                            .is_empty()
                    })
                    .unwrap_or(false);

                if no_display {
                    return None;
                }

                let hidden = entry
                    .entry
                    .get("Desktop Entry", "Hidden")
                    .map(|e| {
                        !e.iter()
                            .filter(|t| t.as_str() == "true")
                            .collect::<Vec<_>>()
                            .is_empty()
                    })
                    .unwrap_or(false);

                if hidden {
                    return None;
                }

                Some(entry.into())
            }
            Err(_) => None,
        })
        .collect();

    entries
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry {
    path: PathBuf,
    entry: freedesktop_entry_parser::Entry,
}

impl From<DesktopEntry> for Entry {
    fn from(val: DesktopEntry) -> Self {
        let title = {
            val.entry
                .get("Desktop Entry", "Name")
                .and_then(|e| e.first())
                .map(|e| e.as_str())
                .unwrap_or("Unnamed")
        };

        let generic_name = {
            val.entry
                .get("Desktop Entry", "GenericName")
                .and_then(|e| e.first())
                .map(|e| e.as_str())
        };

        let description = {
            val.entry
                .get("Desktop Entry", "Comment")
                .and_then(|e| e.first())
                .map(|e| e.as_str())
        };

        let is_terminal = val
            .entry
            .get("Desktop Entry", "Terminal")
            .map(|e| {
                !e.iter()
                    .filter(|t| t.as_str() == "true")
                    .collect::<Vec<_>>()
                    .is_empty()
            })
            .unwrap_or(false);

        let id = val.path.to_str().unwrap_or_default().to_string();

        let icon = {
            val.entry
                .get("Desktop Entry", "Icon")
                .and_then(|e| e.first())
                .map(|e| e.to_string())
        };

        let launch_command = {
            val.entry
                .get("Desktop Entry", "Exec")
                .and_then(|e| e.first())
                .map(|e| e.to_string())
        };

        Entry::builder()
            .id(&id)
            .title(title)
            .generic_name(generic_name)
            .description(description)
            .icon(icon)
            .launch_command(launch_command)
            .is_terminal(is_terminal)
            .user_confirm(false)
            .build()
    }
}

pub fn parse_desktop_entry(path: &PathBuf) -> Result<DesktopEntry, anyhow::Error> {
    let entry = freedesktop_entry_parser::parse_entry(path)?;

    Ok(DesktopEntry {
        entry,
        path: path.clone(),
    })
}
