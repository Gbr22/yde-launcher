use crate::entry::Entry;

pub fn get_builtin_actions() -> Vec<Entry> {
    let mut actions: Vec<Entry> = Vec::new();

    actions.push(Entry::builder()
        .id("yde-launcher:action:logout")
        .title("Logout")
        .description(Some("Log out of the currently active session"))
        .icon(Some("system-log-out"))
        .launch_command(Some("sh -c 'loginctl terminate-session $XDG_SESSION_ID'"))
        .is_terminal(false)
        .build());

    actions.push(Entry::builder()
        .id("yde-launcher:action:shutdown")
        .title("Shutdown")
        .description(Some("Shut down the system"))
        .icon(Some("system-shutdown"))
        .launch_command(Some("shutdown now"))
        .is_terminal(false)
        .build());

    actions.push(Entry::builder()
        .id("yde-launcher:action:restart")
        .title("Restart")
        .description(Some("Restart the system"))
        .icon(Some("system-reboot"))
        .launch_command(Some("shutdown -r now"))
        .is_terminal(false)
        .build());

    actions
}