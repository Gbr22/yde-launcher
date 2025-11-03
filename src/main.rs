use std::rc::Rc;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use gpui::*;
use gpui_component::{ActiveTheme, Root, StyledExt, Theme, VirtualListScrollHandle, v_virtual_list};
use gpui_component::input::{InputEvent, InputState, TextInput};
use gpui_component::Selectable;

mod data;
mod entry;

use crate::data::DesktopEntry;
use crate::entry::Entry;

#[derive(Clone)]
pub struct State {
    selection_index: usize,
    query: String,
    input_focus_handle: Option<FocusHandle>,
    entries: Vec<DesktopEntry>,
    filtered_entries: Vec<DesktopEntry>,
    scroll_handle: VirtualListScrollHandle,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selection_index: 0,
            query: String::new(),
            input_focus_handle: None,
            entries: Vec::new(),
            filtered_entries: Vec::new(),
            scroll_handle: VirtualListScrollHandle::new(),
        }
    }
}

impl Global for State {}

impl State {
    fn update_filtered_entries(&mut self) {
        let matcher = SkimMatcherV2::default();
        let mut vec = self.entries.iter().flat_map(|entry| {
            if self.query.is_empty() {
                Some((0, entry))
            } else {
                let title_match = matcher.fuzzy_match(entry.title(), &self.query);
                let generic_name_match = entry.generic_name().and_then(|gn| matcher.fuzzy_match(gn, &self.query));
                let score = match (title_match, generic_name_match) {
                    (Some(ts), Some(gs)) => Some(ts.max(gs)),
                    (Some(ts), None) => Some(ts),
                    (None, Some(gs)) => Some(gs),
                    (None, None) => None,
                };
                score.map(|score| {
                    (score, entry)
                })
            }
        }).collect::<Vec<_>>();
        vec.sort_by(|a,b|a.1.title().cmp(b.1.title()));
        vec.sort_by(|a, b| b.0.cmp(&a.0));
        self.filtered_entries = vec.iter().map(|(_, entry)| (*entry).clone()).collect();
    }
    fn set_query(&mut self, query: String) {
        self.query = query;
        self.update_filtered_entries();
    }
    fn set_selection_index(&mut self, index: usize) {
        self.selection_index = index;
        if self.selection_index >= self.filtered_entries.len() {
            self.selection_index = self.filtered_entries.len().saturating_sub(1);
        }
        self.update_filtered_entries();
        self.scroll_handle.scroll_to_item(index, ScrollStrategy::Top);
    }
    fn add_selection_index(&mut self, delta: isize) {
        if delta < 0 {
            self.set_selection_index(self.selection_index.saturating_sub(delta.abs() as usize));
        } else {
            self.set_selection_index(self.selection_index.saturating_add(delta as usize));
        }
        self.update_filtered_entries();
    }
    fn get_selected_entry(&self) -> Option<&DesktopEntry> {
        self.filtered_entries.get(self.selection_index)
    }
    fn launch_entry(&self, entry: &impl Entry) {
        let placeholder_regex = regex::Regex::new(r"%\w").expect("Hardcoded regex should compile");
        println!("Launching entry: {:?}", entry);
        let command = entry.launch_command().map(|cmd|{
            placeholder_regex.replace_all(&cmd, "").replace("%%","%")
        });

        if let Some(command) = command {
            let Ok(args) = shell_words::split(&command) else {
                return;
            };
            let args: Vec<String> = args.iter().flat_map(|part|{
                if part == "%%" {
                    Some("%".to_string())
                } else if part.starts_with("%") {
                    None
                } else {
                    Some(part.clone())
                }
            }).collect();
            
            println!("Launching command: {:?}", args);
            
            #[cfg(target_family = "unix")]
            {
                use std::process::Command;
                use std::os::unix::process::CommandExt;
                
                let devnull = std::fs::File::open("/dev/null").unwrap();

                unsafe {
                    Command::new(&args[0])
                        .args(&args[1..])
                        .stdin(devnull.try_clone().unwrap())
                        .stdout(devnull.try_clone().unwrap())
                        .stderr(devnull)
                        .pre_exec(|| {
                            if libc::setsid() == -1 {
                                return Err(std::io::Error::last_os_error());
                            }
                            Ok(())
                        })
                        .spawn()
                        .expect("Failed to launch command");
                };
            }
        }
    }
}

pub struct App {
    input: Entity<InputState>,
}

impl Render for App {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input = &self.input;
        let state = cx.global::<State>();

        div()
            .v_flex()
            .size_full()
            .content_stretch()
            .child(
                div()
                .overflow_hidden()
                .child(
                    TextInput::new(input)
                    .selected(true)
                    .appearance(false)
                )
            )
            .child(
                div()
                .overflow_hidden()
                .flex_1()
                .child(
                    div()
                    .w_full()
                )
                .child(
                    v_virtual_list(
                        cx.entity().clone(),
                        "entry-list",
                        Rc::new(vec![size(px(9999.), px(48.)); state.filtered_entries.len()]),
                        |_view, visible_range, _window, cx| {
                            visible_range
                                .map(|ix| {
                                    let state = cx.global::<State>();
                                    let is_selected = state.selection_index == ix;
                                    let entry = state.filtered_entries.get(ix);

                                    let Some(entry) = entry else {
                                        return div();
                                    };

                                    let img_src = entry.icon_path().and_then(|path| path.to_str().map(|s| s.to_string()));
                                    let img_el = match img_src {
                                        Some(src) => vec![
                                            img(src)
                                            .w_full()
                                            .h_full()
                                            .bg(rgb(0x00FF00))
                                            .object_fit(ObjectFit::Contain)
                                        ],
                                        None => vec![],
                                    };

                                    div()
                                        .flex()
                                        .flex_row()
                                        .overflow_hidden()
                                        .h(px(48.))
                                        .w_full()
                                        .bg(if is_selected { cx.theme().accent } else { cx.theme().background })
                                        .child(
                                            div()
                                            .flex_basis(px(48.))
                                            .flex_none()
                                            .h_full()
                                            .children(img_el)
                                        )
                                        .child(
                                            div()
                                            .flex_1()
                                            .child(
                                                entry.title().to_string()
                                            )
                                            .child(
                                                entry.description().unwrap_or_default().to_string()
                                            )
                                        )
                                })
                                .collect()
                        },
                    )
                    .track_scroll(&state.scroll_handle))
            )
    }
}

const APP_NAME: &str = "YDE Launcher";

impl App {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx|
            InputState::new(window, cx)
                .placeholder("Search")
        );
        let focus_handle = input.focus_handle(cx);
        State::update_global(cx, |state, cx| {
            state.input_focus_handle = Some(focus_handle);
        });

        cx.subscribe_in(&input, window, |view, state, event, window, cx|{
            match event {
                InputEvent::Change => {
                    let text = state.read(cx).value();
                    State::update_global(cx, |state, _cx|{
                        state.set_query(text.to_string());
                    });
                }
                InputEvent::PressEnter { secondary: _ } => {
                    let state = cx.global::<State>();
                    if let Some(entry) = state.get_selected_entry() {
                        state.launch_entry(entry);
                    }
                    cx.quit();
                }
                _ => {}
            };
        }).detach();

        cx.observe_global::<State>(|_state, cx| {
            cx.notify();
        }).detach();

        App { input }
    }
}

fn main() {
    let app = Application::new();
    let entries = data::get_desktop_entries();
    let mut state = State::default();
    state.entries = entries.clone();
    state.update_filtered_entries();

    app.run(move |cx| {
        gpui_component::init(cx);
        Theme::global_mut(cx).window_border = hsla(0., 0., 0., 0.);
        cx.set_global(state);
        cx.observe_keystrokes(|event, _window, cx| {
            if event.keystroke.key == "up" {
                State::update_global(cx, |state, _cx| {
                    state.add_selection_index(-1);
                });
            }
            if event.keystroke.key == "down" {
                State::update_global(cx, |state, _cx| {
                    state.add_selection_index(1);
                });
            }
            if event.keystroke.key == "pageup" {
                State::update_global(cx, |state, _cx| {
                    state.set_selection_index(0);
                });
            }
            if event.keystroke.key == "pagedown" {
                State::update_global(cx, |state, _cx| {
                    state.set_selection_index(state.filtered_entries.len().saturating_sub(1));
                });
            }
            if event.keystroke.key == "escape" {
                cx.quit();
            }
        }).detach();

        cx.spawn(async move |cx| {
            let window = cx.open_window(WindowOptions {
                window_decorations: Some(WindowDecorations::Client),
                window_bounds: Some(WindowBounds::Windowed(bounds(point(px(0.), px(0.)), size(px(500.), px(400.))))),
                is_resizable: false,
                is_movable: false,
                kind: WindowKind::Floating,
                titlebar: Some(TitlebarOptions {
                    title: Some(APP_NAME.into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                ..Default::default()
            }, |window, cx| {
                let view = cx.new(|cx| {
                    App::new(window, cx)
                });

                cx.new(|cx| {
                    Root::new(view.into(), window, cx)
                })
            })?;

            window
                .update(cx, |_, window, cx| {
                    window.activate_window();
                    let state = cx.global::<State>();
                    let input_handle = &state.input_focus_handle;
                    if let Some(input_handle) = input_handle {
                        input_handle.focus(window);
                    };
                    window.set_window_title(APP_NAME);
                })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
