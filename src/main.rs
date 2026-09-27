use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;
use std::path::PathBuf;

use eframe::{egui, wgpu::Color};
use egui::{Color32, CornerRadius, FontId, Margin, RichText, ScrollArea, Sense, Stroke, TextEdit};

mod actions;
mod color;
mod data;
mod entry;

use crate::{
    color::{ColorUtil, rgb},
    entry::Entry,
};

struct App {
    selection_index: usize,
    query: String,
    entries: Vec<Entry>,
    filtered_entries: Vec<Entry>,
    icon_map: HashMap<String, String>,
    confirming_entry: Option<Entry>,
    first_frame: bool,
    target_index: Option<(usize, usize)>,
    target_pos: Option<f32>,
    scroll_offset: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            selection_index: 0,
            query: String::new(),
            entries: Vec::new(),
            filtered_entries: Vec::new(),
            icon_map: HashMap::new(),
            confirming_entry: None,
            first_frame: true,
            target_index: None,
            target_pos: None,
            scroll_offset: 0.0,
        }
    }
}

impl App {
    fn find_icons(&mut self) {
        let theme_name = linicon_theme::get_icon_theme();
        let entries = &self.entries;
        self.icon_map.clear();
        let iter: Vec<(String, PathBuf)> = entries
            .par_iter()
            .map(|entry| {
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
            })
            .flatten()
            .collect();
        for (icon_name, icon_path) in iter {
            self.icon_map
                .insert(icon_name, format!("file://{}", icon_path.display()));
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
        let mut vec = self
            .entries
            .iter()
            .flat_map(|entry| {
                if self.query.is_empty() {
                    Some((0, entry))
                } else {
                    let title_match = matcher.fuzzy_match(entry.title(), &self.query);
                    let generic_name_match = entry
                        .generic_name()
                        .and_then(|gn| matcher.fuzzy_match(gn, &self.query));
                    let score = match (title_match, generic_name_match) {
                        (Some(ts), Some(gs)) => Some(ts.max(gs)),
                        (Some(ts), None) => Some(ts),
                        (None, Some(gs)) => Some(gs),
                        (None, None) => None,
                    };
                    score.map(|score| (score, entry))
                }
            })
            .collect::<Vec<_>>();
        vec.sort_by(|a, b| a.1.title().cmp(b.1.title()));
        vec.sort_by(|a, b| b.0.cmp(&a.0));
        self.filtered_entries = vec.iter().map(|(_, entry)| (*entry).clone()).collect();
    }
    fn update_query(&mut self) {
        self.set_selection_index(0);
        self.update_filtered_entries();
    }
    fn set_selection_index(&mut self, index: usize) {
        let old_index = self.selection_index;
        let mut new_index = index;
        if new_index >= self.filtered_entries.len() {
            new_index = self.filtered_entries.len().saturating_sub(1);
        }
        self.selection_index = new_index;
        self.update_filtered_entries();
        self.target_index = Some((new_index, old_index));
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
        } else {
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
            let args: Vec<String> = args
                .into_iter()
                .flat_map(|part| {
                    if part == "%%" {
                        Some("%".to_string())
                    } else if part.starts_with("%") {
                        None
                    } else {
                        Some(part)
                    }
                })
                .collect();

            println!("Launching command: {:?}", args);

            #[cfg(target_family = "unix")]
            {
                use std::process::Command;

                if entry.is_terminal() {
                    Command::new("xdg-terminal")
                        .args(&args)
                        .spawn()
                        .expect("Failed to launch command");
                } else {
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

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let first_frame = self.first_frame;
        self.first_frame = false;
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if first_frame {
            self.refresh_entries();
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = Color32::TRANSPARENT;
            visuals.window_fill = Color32::TRANSPARENT;
            visuals.override_text_color = Some(rgb!("#cdd6f4"));
            visuals.text_cursor.stroke.color = rgb!("#b4befe");
            visuals.selection.bg_fill = rgb!("#89dceb").with_alpha(0.3);

            ui.set_visuals(visuals);
        }

        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.add_selection_index(-1);
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.add_selection_index(1);
        }
        if ui.input(|i| i.key_pressed(egui::Key::PageUp)) {
            self.set_selection_index(0);
        }
        if ui.input(|i| i.key_pressed(egui::Key::PageDown)) {
            self.set_selection_index(self.filtered_entries.len().saturating_sub(1));
        }
        let is_ctrl_pressed = ui.input(|i|i.modifiers.ctrl);
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(entry) = self.get_selected_entry().cloned() {
                if is_ctrl_pressed {
                    self.execute_entry(entry);
                } else {
                    self.launch_entry(entry);
                }
            }
        }

        let window_frame = egui::Frame::new()
            .fill(rgb!("#1e1e2e").with_alpha(0.9))
            .corner_radius(CornerRadius::same(4))
            .stroke(Stroke::new(1.0, rgb!("#313244")))
            .inner_margin(Margin::same(8));

        egui::CentralPanel::default()
            .frame(window_frame)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    let input_font = FontId::proportional(16.0);
                    let input = ui.add(
                        TextEdit::singleline(&mut self.query)
                            .font(input_font.clone())
                            .hint_text(RichText::new("Search").font(input_font))
                            .frame(egui::Frame::new())
                            .desired_width(ui.available_width()),
                    );
                    if input.changed() {
                        self.update_query();
                    }
                    if first_frame
                        || ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary))
                    {
                        input.request_focus();
                    }

                    ui.add_space(4.0);

                    let total_rows = self.filtered_entries.len();
                    let row_height = 48.0;
                    let row_stride = row_height + ui.spacing().item_spacing.y;

                    let mut scroll_area = ScrollArea::vertical();

                    if let Some(target) = self.target_pos.take() {
                        scroll_area = scroll_area.vertical_scroll_offset(target);
                    }

                    let scroll_area_output = scroll_area.auto_shrink([false; 2]).show_rows(
                        ui,
                        row_height,
                        total_rows,
                        |ui, row_range| {
                            let clip_rect = ui.clip_rect();

                            let is_visible: Vec<_> = row_range
                                .map(|index| {
                                    let item = &self.filtered_entries[index];
                                    let is_selected = self.selection_index == index;
                                    let fill_color = match (is_selected, item.user_confirm(), is_ctrl_pressed) {
                                        (true, false, _) => rgb!("#313244").with_alpha(0.8),
                                        (true, true, false) => rgb!("#df8e1d").with_alpha(0.4),
                                        (true, true, true) => rgb!("#d20f39").with_alpha(0.8),
                                        _ => Color32::TRANSPARENT,
                                    };
                                    let item_frame = egui::Frame::new()
                                        .fill(fill_color)
                                        .corner_radius(CornerRadius::same(4));
                                    let item_frame_res = item_frame
                                        .show(ui, |ui| {
                                            ui.set_height(row_height);
                                            ui.set_width(ui.available_width());
                                            ui.allocate_ui_with_layout(
                                                egui::vec2(ui.available_width(), row_height),
                                                egui::Layout::left_to_right(egui::Align::Center),
                                                |ui| {
                                                    let icon_path = item
                                                        .icon()
                                                        .and_then(|icon| self.icon_map.get(icon));

                                                    ui.add_space(8.0);
                                                    let image_size = 40.0;

                                                    if let Some(icon_path) = icon_path {
                                                        ui.add(
                                                            egui::Image::new(icon_path)
                                                                .show_loading_spinner(false)
                                                                .fit_to_exact_size(egui::vec2(
                                                                    image_size, image_size,
                                                                )),
                                                        );
                                                    } else {
                                                        ui.add_space(image_size);
                                                    }

                                                    ui.add_space(4.0);

                                                    ui.vertical(|ui| {
                                                        ui.add_space(4.0);
                                                        ui.add(
                                                            egui::Label::new(
                                                                egui::RichText::new(item.title())
                                                                    .size(16.0),
                                                            )
                                                            .selectable(false)
                                                            .truncate(),
                                                        );
                                                        if let Some(description) =
                                                            item.description()
                                                            && !description.trim().is_empty()
                                                        {
                                                            ui.add(
                                                                egui::Label::new(
                                                                    egui::RichText::new(
                                                                        description,
                                                                    )
                                                                    .size(14.0)
                                                                    .color(rgb!("#bac2de")),
                                                                )
                                                                .selectable(false)
                                                                .truncate(),
                                                            );
                                                        }
                                                    });
                                                },
                                            );
                                        })
                                        .response
                                        .interact(Sense::click());
                                    if item_frame_res.clicked() {
                                        if self.selection_index == index {
                                            self.launch_entry(item.clone());
                                        } else {
                                            self.set_selection_index(index);
                                        }
                                    }
                                    let is_visible = clip_rect.intersects(item_frame_res.rect);
                                    (is_visible, index)
                                })
                                .collect();
                            if let Some((new_index, old_index)) = self.target_index.take() {
                                let diff = old_index.abs_diff(new_index);
                                let dir = if new_index > old_index { 1.0 } else { -1.0 };
                                let is_index_visible = |target: usize| {
                                    is_visible
                                        .iter()
                                        .any(|(is_visible, index)| *index == target && *is_visible)
                                };
                                if !is_index_visible(new_index) {
                                    if diff == 1 && is_index_visible(old_index) {
                                        self.target_pos =
                                            Some(self.scroll_offset + row_stride * dir);
                                    } else {
                                        self.target_pos = Some(new_index as f32 * row_stride);
                                    }
                                }
                            }
                        },
                    );
                    self.scroll_offset = scroll_area_output.state.offset.y;
                });
            });
    }
}

const APP_NAME: &str = "YDE Launcher";

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 400.0])
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_title(APP_NAME),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}
