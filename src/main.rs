use std::collections::HashMap;
use std::path::{PathBuf};
use std::rc::Rc;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollHandleOffsetable;
use gpui_component::{Kbd, Root, Theme, VirtualListScrollHandle, v_virtual_list};
use gpui_component::input::{InputEvent, InputState, TextInput};
use gpui_component::Selectable;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

mod data;
mod entry;
mod utils;
mod actions;

use crate::data::DesktopEntry;
use crate::entry::Entry;
use crate::utils::WithAlpha;

#[derive(Clone)]
pub struct State {
    selection_index: usize,
    query: String,
    input_focus_handle: Option<FocusHandle>,
    entries: Vec<Entry>,
    filtered_entries: Vec<Entry>,
    scroll_handle: VirtualListScrollHandle,
    scroll_view_bounds: Option<Bounds<Pixels>>,
    icon_map: HashMap<String, PathBuf>,
    confirming_entry: Option<Entry>,
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
            icon_map: HashMap::new(),
            scroll_view_bounds: None,
            confirming_entry: None,
        }
    }
}

impl Global for State {}

impl State {
    fn find_icons(&mut self) {
        let theme_name = linicon_theme::get_icon_theme();
        let entries = &self.entries;
        self.icon_map.clear();
        let iter: Vec<(String, PathBuf)> = entries.par_iter().map(|entry|{
            let icon = entry.icon();
            if let Some(icon) = icon {
                let path_buf = PathBuf::from(icon);
                if path_buf.is_absolute() {
                    return Some((icon.to_string(), path_buf));
                }

                if let Some(theme_name) = &theme_name {
                    let icon_path = freedesktop_icons::lookup(icon)
                        .with_theme(&theme_name)
                        .force_svg()
                        .with_size(64)
                        .find();
                    if let Some(icon_path) = icon_path {
                        return Some((icon.to_string(), icon_path));
                    }
                }

                let icon_path = freedesktop_icons::lookup(icon)
                    .with_theme("hicolor")
                    .force_svg()
                    .with_size(64)
                    .find();
                if let Some(icon_path) = icon_path {
                    return Some((icon.to_string(), icon_path));
                }
            }
            return None;
        }).flatten().collect();
        for (icon_name, icon_path) in iter {
            self.icon_map.insert(icon_name, icon_path);
        }
    }
    fn refresh_entries(&mut self) {
        self.entries = vec![];
        self.entries.extend(data::get_desktop_entries());
        self.entries.extend(actions::get_builtin_actions());

        self.update_filtered_entries();
        self.find_icons();
    }
    fn update_filtered_entries(&mut self) {
        let matcher = SkimMatcherV2::default();
        let mut vec = self.entries.iter().flat_map(|entry| {
            if self.query.is_empty() {
                Some((0, entry))
            } else {
                let title_match = matcher.fuzzy_match(entry.title().as_str(), &self.query);
                let generic_name_match = entry.generic_name().and_then(|gn| matcher.fuzzy_match(gn.as_str(), &self.query));
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
        vec.sort_by(|a,b|a.1.title().as_str().cmp(b.1.title().as_str()));
        vec.sort_by(|a, b| b.0.cmp(&a.0));
        self.filtered_entries = vec.iter().map(|(_, entry)| (*entry).clone()).collect();
    }
    fn set_query(&mut self, query: String) {
        self.query = query;
        self.set_selection_index(0);
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
    fn get_selected_entry(&self) -> Option<&Entry> {
        self.filtered_entries.get(self.selection_index)
    }
    fn launch_entry(&mut self, entry: Entry) {
        if entry.user_confirm() {
            self.confirming_entry = Some(entry);
        }
        else {
            self.execute_entry(entry);
        }
    }
    fn execute_entry(&self, entry: Entry) {
        println!("Launching entry: {:?}", entry);
        let command = entry.launch_command();

        if let Some(command) = command {
            let Ok(args) = shell_words::split(&command) else {
                return;
            };
            let args: Vec<String> = args.into_iter().flat_map(|part|{
                if part == "%%" {
                    Some("%".to_string())
                } else if part.starts_with("%") {
                    None
                } else {
                    Some(part)
                }
            }).collect();
            
            println!("Launching command: {:?}", args);
            
            #[cfg(target_family = "unix")]
            {
                use std::process::Command;

                if entry.is_terminal() {
                    Command::new("xdg-terminal")
                        .args(&args)
                        .spawn()
                        .expect("Failed to launch command");
                }
                else {
                    Command::new(&args[0])
                        .args(&args[1..])
                        .spawn()
                        .expect("Failed to launch command");
                }
            }
            
            std::process::exit(0);
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
        let scroll_height = f32::from(state.scroll_handle.max_offset().height);
        let viewport_height: f32 = state.scroll_view_bounds.map(|b| f32::from(b.size.height)).unwrap_or(0.);
        let scrollbar_height = (viewport_height * (viewport_height / scroll_height)).clamp(f32::min(16., viewport_height / 2.), viewport_height / 2.);
        let scroll_ratio = (-f32::from(state.scroll_handle.offset().y) / scroll_height).clamp(0., 1.);
        let scrollbar_top_offset = scroll_ratio * (viewport_height - scrollbar_height);

        div()
            .bg(rgb(0x1e1e2e).with_alpha(0.9))
            .border_color(rgb(0x313244))
            .border_1()
            .rounded(px(4.))
            .overflow_hidden()
            .h_full()
            .w_full()
            .flex()
            .flex_col()
            .content_stretch()
            .child(
                div()
                .h(px(32.))
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .overflow_hidden()
                .child(
                    TextInput::new(input)
                    .h_full()
                    .w_full()
                    .selected(state.confirming_entry.is_none())
                    .appearance(false)
                )
            )
            .child(
                div()
                .overflow_hidden()
                .flex_1()
                .flex()
                .flex_row()
                .flex_nowrap()
                .child(
                    div()
                    .h_full()
                    .flex_1()
                    .p(px(8.))
                    .pr_0()
                    .pt_0()
                    .on_children_prepainted(|bounds, _window, cx|{
                        let bounds = bounds.first();
                        let Some(bounds) = bounds else {
                            return;
                        };
                        let state = cx.global_mut::<State>();
                        state.scroll_view_bounds = Some(*bounds);
                    })
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

                                        let img_src = entry.icon().map(|icon|state.icon_map.get(icon)).flatten();

                                        let img_el = match img_src {
                                            Some(src) => vec![
                                                img(src.to_owned())
                                                .w_full()
                                                .h_full()
                                                .object_fit(ObjectFit::Contain)
                                            ],
                                            None => vec![],
                                        };

                                        div()
                                            .flex()
                                            .flex_row()
                                            .overflow_hidden()
                                            .h(px(48.))
                                            .rounded(px(4.))
                                            .w_full()
                                            .overflow_hidden()
                                            .bg(if is_selected { rgb(0x313244).with_alpha(0.8) } else { rgba(0x0).into() })
                                            .child(
                                                div()
                                                .flex_basis(px(48.))
                                                .flex_none()
                                                .h_full()
                                                .p_1()
                                                .children(img_el)
                                            )
                                            .child(
                                                div()
                                                .flex_1()
                                                .w_full()
                                                .overflow_hidden()
                                                .child(
                                                    div()
                                                    .child(entry.title().to_string())
                                                    .text_size(px(16.))
                                                    .overflow_hidden()
                                                    .w_full()
                                                    .text_ellipsis()
                                                )
                                                .child(
                                                    div()
                                                    .child(entry.description().unwrap_or_default().to_string())
                                                    .text_size(px(14.))
                                                    .text_color(rgb(0xbac2de))
                                                    .overflow_hidden()
                                                    .w_full()
                                                    .text_ellipsis()
                                                )
                                            )
                                    })
                                    .collect()
                            },
                        )
                        .track_scroll(&state.scroll_handle)
                    )
                )
                .child(
                    div()
                    .h_full()
                    .w(px(8.))
                    .flex_shrink_0()
                    .child(
                        div()
                        .absolute()
                        .top(px(scrollbar_top_offset))
                        .h(px(scrollbar_height))
                        .w_full()
                        .bg(rgb(0x7f849c))
                        .rounded(px(4.))
                    )
                )
            )
            .when(state.confirming_entry.is_some(), |e|{
                let entry = state.confirming_entry.clone().unwrap();

                e.child(
                    div()
                    .absolute()
                    .top(px(0.))
                    .left(px(0.))
                    .right(px(0.))
                    .bottom(px(0.))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .p(px(8.))
                        .rounded(px(4.))
                        .bg(rgb(0x313244))
                        .border_color(rgb(0x313244))
                        .border_1()
                        .shadow(vec![gpui::BoxShadow {
                            color: Hsla {
                                h: 0.,
                                s: 0.,
                                l: 0.,
                                a: 0.4,
                            },
                            blur_radius: px(8.),
                            spread_radius: px(0.),
                            offset: point(px(0.), px(0.)),
                        }])
                        .child(format!("{}", entry.title()))
                        .child(
                            div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(
                                Button::new("confirm")
                                .primary()
                                .label("Run")
                                .w_full()
                                .child(Kbd::new(Keystroke::parse("ctrl-enter").unwrap()))
                                .on_click(|_event, _window, cx| {
                                    let state = cx.global::<State>();
                                    let entry = state.confirming_entry.clone();
                                    if let Some(entry) = entry {
                                        state.execute_entry(entry);
                                    }
                                })
                            )
                            .child(
                                Button::new("cancel")
                                .primary()
                                .label("Cancel")
                                .w_full()
                                .child(Kbd::new(Keystroke::parse("escape").unwrap()))
                                .on_click(|_event, window, _cx| {
                                    State::update_global(_cx, |state, _cx| {
                                        state.confirming_entry = None;
                                        state.input_focus_handle.clone().map(|h| h.focus(window));
                                    });
                                })
                            )
                        )
                    )
                )
            })
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
                    cx.update_global(|state: &mut State, cx| {
                        if let Some(entry) = state.get_selected_entry().cloned() {
                            state.launch_entry(entry);
                        }
                    });
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
    env_logger::init();
    let app = Application::new();
    let mut state = State::default();
    state.refresh_entries();

    app.run(move |cx| {
        gpui_component::init(cx);
        Theme::global_mut(cx).window_border = rgba(0x0).into();
        Theme::global_mut(cx).background = rgba(0x0).into();
        Theme::global_mut(cx).muted_foreground = rgb(0xbac2de).into();
        Theme::global_mut(cx).foreground = rgb(0xcdd6f4).into();
        Theme::global_mut(cx).caret = rgb(0xb4befe).into();
        Theme::global_mut(cx).selection = rgb(0x89dceb).with_alpha(0.3).into();
        cx.set_global(state);
        cx.observe_keystrokes(|event, _window, cx| {
            if event.keystroke.key == "up" {
                State::update_global(cx, |state, _cx| {
                    if state.confirming_entry.is_some() { return; }
                    state.add_selection_index(-1);
                });
            }
            if event.keystroke.key == "down" {
                State::update_global(cx, |state, _cx| {
                    if state.confirming_entry.is_some() { return; }
                    state.add_selection_index(1);
                });
            }
            if event.keystroke.key == "pageup" {
                State::update_global(cx, |state, _cx| {
                    if state.confirming_entry.is_some() { return; }
                    state.set_selection_index(0);
                });
            }
            if event.keystroke.key == "pagedown" {
                State::update_global(cx, |state, _cx| {
                    if state.confirming_entry.is_some() { return; }
                    state.set_selection_index(state.filtered_entries.len().saturating_sub(1));
                });
            }
            if event.keystroke.key == "escape" {
                let state = cx.global::<State>();
                if state.confirming_entry.is_some() {
                    State::update_global(cx, |state, _cx| {
                        state.confirming_entry = None;
                        state.input_focus_handle.clone().map(|h| h.focus(_window));
                    });
                    return;
                }
                cx.quit();
            }
            if event.keystroke.key == "enter" {
                if event.keystroke.modifiers.control {
                    let state = cx.global::<State>();
                    let entry = state.confirming_entry.clone();
                    if let Some(entry) = entry {
                        State::update_global(cx, |state, _cx| {
                            state.execute_entry(entry);
                        });
                        return;
                    }
                }
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
