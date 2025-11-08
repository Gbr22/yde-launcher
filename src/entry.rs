use std::{fmt::Debug};

pub trait Entry: Debug {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
    fn generic_name(&self) -> Option<&str>;
    fn description(&self) -> Option<&str>;
    fn icon(&self) -> Option<&str>;
    fn launch_command(&self) -> Option<&str>;
}
