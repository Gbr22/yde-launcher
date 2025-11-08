use std::{fmt::Debug, path::PathBuf, usize};

pub trait Entry: Debug {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
    fn generic_name(&self) -> Option<&str>;
    fn description(&self) -> Option<&str>;
    fn icon_path(&self, size: (u32, u32)) -> Option<&PathBuf>;
    fn launch_command(&self) -> Option<&str>;
}
