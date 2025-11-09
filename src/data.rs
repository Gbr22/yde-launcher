use std::{path::PathBuf};
use gpui::SharedString;
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

                let hidden = entry.entry.get("Desktop Entry", "Hidden")
                    .map(|e| e.iter().filter(|t| t.as_str() == "true").collect::<Vec<_>>().len() > 0).unwrap_or(false);

                if hidden {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedIconSize {
    pub width: u32,
    pub height: u32,
    pub scale: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconSize {
    Scalable,
    Fixed(FixedIconSize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Icon {
    pub theme: Option<String>,
    pub path: PathBuf,
    pub size: IconSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry {
    path: PathBuf,
    entry: freedesktop_entry_parser::Entry,
    title: SharedString,
    generic_name: Option<SharedString>,
    description: Option<SharedString>,
}

impl Entry for DesktopEntry {
    fn id(&self) -> &str {
        self.path.to_str().unwrap_or_default()
    }
    fn title(&self) -> SharedString {
        self.title.clone()
    }
    fn generic_name(&self) -> Option<SharedString> {
        self.generic_name.clone()
    }

    fn description(&self) -> Option<SharedString> {
        match &self.description {
            Some(desc) => return Some(desc.clone()),
            None => {
                return self.generic_name.clone();
            }
        }
    }

    fn icon(&self) -> Option<&str> {
        self.entry.get("Desktop Entry", "Icon")
            .map(|e| e.first())
            .flatten()
            .map(|e| e.as_str())
    }

    fn launch_command(&self) -> Option<&str> {
        self.entry.get("Desktop Entry", "Exec")
            .map(|e| e.first())
            .flatten()
            .map(|e| e.as_str())
    }
}

pub fn parse_icon_size(icon_size: impl AsRef<str>) -> Option<IconSize> {
    let icon_size = icon_size.as_ref();
    if icon_size == "scalable" {
        return Some(IconSize::Scalable);
    }

    let parts: Vec<&str> = icon_size.split('@').collect();
    let size = parts.get(0).unwrap_or(&"").to_string();
    let scale = parts.get(1).unwrap_or(&"").to_string();
    let scale = if scale.is_empty() { 1 } else {
        if let Ok(scale) = scale.parse::<u32>() {
            scale
        } else {
            return None;
        }
    };
    let parts = size.split('x').collect::<Vec<&str>>();
    if parts.len() != 2 {
        return None;
    }

    let (width, height) = (parts[0], parts[1]);
    let (width, height) = (width.parse::<u32>(), height.parse::<u32>());
    if let (Ok(width), Ok(height)) = (width, height) {
        return Some(IconSize::Fixed(FixedIconSize { width, height, scale }));
    }

    None
}

pub fn parse_desktop_entry(path: &PathBuf) -> Result<DesktopEntry, anyhow::Error> {
    let entry = freedesktop_entry_parser::parse_entry(path)?;

    let title = {
        entry.get("Desktop Entry", "Name")
            .map(|e|e.first())
            .flatten()
            .map(SharedString::from)
            .unwrap_or(SharedString::new("Unnamed"))
    };

    let generic_name = {
        entry.get("Desktop Entry", "GenericName")
            .map(|e|e.first())
            .flatten()
            .map(SharedString::from)
    };

    let description = {
        entry.get("Desktop Entry", "Comment")
            .map(|e|e.first())
            .flatten()
            .map(SharedString::from)
    };

    Ok(DesktopEntry { entry, path: path.clone(), title, generic_name, description })
}
