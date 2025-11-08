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
    icons: Vec<Icon>,
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
    fn generic_name(&self) -> Option<&str> {
        self.entry.get("Desktop Entry", "GenericName")
            .map(|e|e.first())
            .flatten()
            .map(|e|e.as_str())
    }

    fn description(&self) -> Option<&str> {
        self.entry.get("Desktop Entry", "Comment")
            .map(|e|e.first())
            .flatten()
            .map(|e|e.as_str())
    }

    fn icon_path(&self, size: (u32, u32)) -> Option<&PathBuf> {
        let (width, height) = size;
        let scalable = self.icons.iter().find(|icon| matches!(icon.size, IconSize::Scalable));
        if let Some(scalable) = scalable {
            return Some(&scalable.path);
        }

        let mut best = None;
        
        for icon in self.icons.iter() {
            if let IconSize::Fixed(size) = &icon.size {
                let reaches_min_size = size.width >= width && size.height >= height;
                if !reaches_min_size {
                    continue;
                }
                if best.is_none() {
                    best = Some(icon);
                }
                else if let Some(best_icon) = best {
                    if let IconSize::Fixed(best_size) = &best_icon.size {
                        if size.width <= best_size.width && size.height <= best_size.height {
                            best = Some(icon);
                        }
                    }
                }
            }
        }

        best.map(|e|&e.path)
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
    let icon_value = entry.get("Desktop Entry", "Icon")
        .map(|e| e.first())
        .flatten()
        .map(|e| e.as_str());

    let icons = if let Some(icon_value) = icon_value {
        let mut icons: Vec<Icon> = Vec::new();
        let icon_path = PathBuf::from(icon_value);

        if !icon_path.is_absolute() {
            let data_dirs = get_data_dirs();
            for dir in data_dirs {
                let theme = "hicolor";
                let theme_dir = dir.join("icons").join(&theme);
                let children = theme_dir.read_dir();
                if let Ok(children) = children {
                    for child in children.flatten() {
                        let resolution_path = child.path();
                        let size = parse_icon_size(child.file_name().to_string_lossy());
                        let Some(size) = size else {
                            continue;
                        };
                        let ext = match size {
                            IconSize::Scalable => "svg",
                            IconSize::Fixed(_) => "png",
                        };
                        let possible_path = resolution_path.join(format!("apps/{}.{}", icon_value, ext));
                        if possible_path.exists() {
                            icons.push(Icon {
                                path: possible_path,
                                size: size,
                                theme: Some(theme.to_string()),
                            });
                        }
                    }
                }
            }
        }

        icons
    } else {
        Vec::new()
    };

    Ok(DesktopEntry { entry, path: path.clone(), icons })
}
