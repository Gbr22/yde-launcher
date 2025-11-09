use gpui::SharedString;

#[derive(Debug, Clone)]
pub struct Entry {
    id: String,
    title: SharedString,
    generic_name: Option<SharedString>,
    description: Option<SharedString>,
    icon: Option<String>,
    launch_command: Option<String>,
    is_terminal: bool,
    user_confirm: bool,
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            id: String::new(),
            title: SharedString::new("Unnamed"),
            generic_name: None,
            description: None,
            icon: None,
            launch_command: None,
            is_terminal: false,
            user_confirm: false,
        }
    }
}

pub struct EntryBuilder {
    entry: Entry,
}

impl EntryBuilder {
    pub fn new() -> Self {
        EntryBuilder {
            entry: Entry::default(),
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.entry.id = id.to_string();
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.entry.title = SharedString::from(title.into());
        self
    }

    pub fn generic_name(mut self, generic_name: Option<impl Into<String>>) -> Self {
        self.entry.generic_name = generic_name.map(|e|SharedString::from(e.into()));
        self
    }

    pub fn description(mut self, description: Option<impl Into<String>>) -> Self {
        self.entry.description = description.map(|e|SharedString::from(e.into()));
        self
    }

    pub fn icon(mut self, icon: Option<impl Into<String>>) -> Self {
        self.entry.icon = icon.map(|e|e.into());
        self
    }

    pub fn launch_command(mut self, launch_command: Option<impl Into<String>>) -> Self {
        self.entry.launch_command = launch_command.map(|e|e.into());
        self
    }

    pub fn is_terminal(mut self, is_terminal: bool) -> Self {
        self.entry.is_terminal = is_terminal;
        self
    }

    pub fn user_confirm(mut self, must_confirm: bool) -> Self {
        self.entry.user_confirm = must_confirm;
        self
    }

    pub fn build(self) -> Entry {
        self.entry
    }
}

impl Entry {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn title(&self) -> SharedString {
        self.title.clone()
    }
    pub fn generic_name(&self) -> Option<SharedString> {
        self.generic_name.clone()
    }
    pub fn description(&self) -> Option<SharedString> {
        self.description.clone()
    }
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }
    pub fn launch_command(&self) -> Option<&str> {
        self.launch_command.as_deref()
    }
    pub fn is_terminal(&self) -> bool {
        self.is_terminal
    }
    pub fn user_confirm(&self) -> bool {
        self.user_confirm
    }
    pub fn builder() -> EntryBuilder {
        EntryBuilder::new()
    }
}
