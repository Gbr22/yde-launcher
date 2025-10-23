use std::path::PathBuf;

pub trait Entry {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn icon_path(&self) -> Option<&PathBuf>;
}
