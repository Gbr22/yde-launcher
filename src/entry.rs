use std::{fmt::Debug};

use gpui::SharedString;

pub trait Entry: Debug {
    fn id(&self) -> &str;
    fn title(&self) -> SharedString;
    fn generic_name(&self) -> Option<SharedString>;
    fn description(&self) -> Option<SharedString>;
    fn icon(&self) -> Option<&str>;
    fn launch_command(&self) -> Option<&str>;
}
