use std::{
    fs, io,
    path::{Path, PathBuf},
};

use eframe::egui::{
    self, CentralPanel, Key, Modifiers, Panel, TextEdit, TextStyle, Vec2, ViewportBuilder,
};

use crate::{
    config::{Config, config_path},
    search,
    text_index::{replace_char_range, slice_char_range},
    wrap,
};

const APP_NAME: &str = "Alinea";
const EDITOR_ID: &str = "alinea_editor";

pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_inner_size(Vec2::new(920.0, 720.0))
            .with_min_inner_size(Vec2::new(560.0, 360.0)),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|creation_context| Ok(Box::new(AlineaApp::new(creation_context)))),
    )
}

#[derive(Debug, Clone)]
struct Document {
    text: String,
    saved_text: String,
    path: Option<PathBuf>,
}

impl Document {
    fn empty() -> Self {
        Self {
            text: String::new(),
            saved_text: String::new(),
            path: None,
        }
    }

    fn from_file(path: PathBuf) -> io::Result<Self> {
        let text = fs::read_to_string(&path)?;
        Ok(Self {
            text: text.clone(),
            saved_text: text,
            path: Some(path),
        })
    }

    fn dirty(&self) -> bool {
        self.text != self.saved_text
    }

    fn title(&self) -> String {
        let name = self
            .path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled");

        if self.dirty() {
            format!("{name} - Edited")
        } else {
            name.to_owned()
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum PendingAction {
    New,
    Open,
    Close,
}

#[derive(Debug, Default)]
struct SearchPanel {
    query: String,
    focus: bool,
    last_status: String,
}

pub struct AlineaApp {
    document: Document,
    config: Config,
    pending_action: Option<PendingAction>,
    status: String,
    search: SearchPanel,
    editor_cursor: usize,
    editor_selection: Option<std::ops::Range<usize>>,
    pending_editor_selection: Option<std::ops::Range<usize>>,
}

impl AlineaApp {
    fn new(_: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load();
        let status = config_path()
            .map(|path| format!("Config: {}", path.display()))
            .unwrap_or_else(|| "Config path unavailable; using defaults".to_owned());

        Self {
            document: Document::empty(),
            config,
            pending_action: None,
            status,
            search: SearchPanel::default(),
            editor_cursor: 0,
            editor_selection: None,
            pending_editor_selection: None,
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::N)) {
            self.request_action(PendingAction::New);
        }
        if ctx.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::O)) {
            self.request_action(PendingAction::Open);
        }
        if ctx.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::S)) {
            self.save();
        }
        if ctx
            .input_mut(|input| input.consume_key(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::S))
        {
            self.save_as();
        }
        if ctx.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::F)) {
            self.search.focus = true;
        }
        if ctx.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::G)) {
            self.find_next();
        }
        if ctx
            .input_mut(|input| input.consume_key(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::G))
        {
            self.find_previous();
        }
        if ctx.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::R)) {
            self.reflow_document();
        }
    }

    fn request_action(&mut self, action: PendingAction) {
        if self.document.dirty() {
            self.pending_action = Some(action);
        } else {
            self.run_action(action);
        }
    }

    fn run_action(&mut self, action: PendingAction) {
        match action {
            PendingAction::New => self.new_document(),
            PendingAction::Open => self.open_document(),
            PendingAction::Close => {
                self.status = "Closing".to_owned();
            }
        }
    }

    fn continue_pending_after_save(&mut self, ctx: &egui::Context) {
        if self.save() {
            if let Some(action) = self.pending_action.take() {
                if matches!(action, PendingAction::Close) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else {
                    self.run_action(action);
                }
            }
        }
    }

    fn new_document(&mut self) {
        self.document = Document::empty();
        self.editor_cursor = 0;
        self.editor_selection = None;
        self.pending_editor_selection = Some(0..0);
        self.status = "New document".to_owned();
    }

    fn open_document(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Open Text File")
            .pick_file()
        else {
            self.status = "Open canceled".to_owned();
            return;
        };

        match Document::from_file(path) {
            Ok(document) => {
                let opened = document
                    .path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "Untitled".to_owned());
                self.document = document;
                self.editor_cursor = 0;
                self.editor_selection = None;
                self.pending_editor_selection = Some(0..0);
                self.status = format!("Opened {opened}");
            }
            Err(error) => {
                self.status = format!("Could not open file: {error}");
            }
        }
    }

    fn save(&mut self) -> bool {
        if let Some(path) = self.document.path.clone() {
            self.write_to_path(&path)
        } else {
            self.save_as()
        }
    }

    fn save_as(&mut self) -> bool {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Save Text File")
            .save_file()
        else {
            self.status = "Save canceled".to_owned();
            return false;
        };

        self.write_to_path(&path)
    }

    fn write_to_path(&mut self, path: &Path) -> bool {
        match fs::write(path, &self.document.text) {
            Ok(()) => {
                self.document.path = Some(path.to_path_buf());
                self.document.saved_text.clone_from(&self.document.text);
                self.status = format!("Saved {}", path.display());
                true
            }
            Err(error) => {
                self.status = format!("Could not save file: {error}");
                false
            }
        }
    }

    fn reflow_document(&mut self) {
        let (reflowed, cursor) = wrap::reflow_text_with_cursor(
            &self.document.text,
            self.editor_cursor,
            self.config.wrap_column,
        );

        if reflowed != self.document.text {
            self.document.text = reflowed;
            self.editor_cursor = cursor;
            self.pending_editor_selection = Some(cursor..cursor);
            self.status = format!("Reflowed document at {} columns", self.config.wrap_column);
        } else {
            self.status = "Document already fits wrap column".to_owned();
        }
    }

    fn reflow_selection(&mut self) {
        let Some(range) = self
            .editor_selection
            .clone()
            .filter(|range| range.start != range.end)
        else {
            self.reflow_document();
            return;
        };

        let selected = slice_char_range(&self.document.text, range.clone());
        let reflowed = wrap::reflow_text(selected, self.config.wrap_column);
        replace_char_range(&mut self.document.text, range.clone(), &reflowed);

        let end = range.start + reflowed.chars().count();
        self.pending_editor_selection = Some(range.start..end);
        self.status = format!("Reflowed selection at {} columns", self.config.wrap_column);
    }

    fn find_next(&mut self) {
        let start = self
            .editor_selection
            .as_ref()
            .map(|range| range.end)
            .unwrap_or(self.editor_cursor);
        self.select_search_match(search::find_next(
            &self.document.text,
            &self.search.query,
            start,
        ));
    }

    fn find_previous(&mut self) {
        let start = self
            .editor_selection
            .as_ref()
            .map(|range| range.start)
            .unwrap_or(self.editor_cursor);
        self.select_search_match(search::find_previous(
            &self.document.text,
            &self.search.query,
            start,
        ));
    }

    fn select_search_match(&mut self, range: Option<std::ops::Range<usize>>) {
        let matches = search::count_matches(&self.document.text, &self.search.query);
        match range {
            Some(range) => {
                self.editor_cursor = range.end;
                self.pending_editor_selection = Some(range.clone());
                self.search.last_status = format!("{matches} match{}", plural(matches));
                self.status = format!("Found match at character {}", range.start + 1);
            }
            None => {
                self.search.last_status = "No matches".to_owned();
                self.status = "No search match".to_owned();
            }
        }
    }

    fn save_config(&mut self) {
        match self.config.save() {
            Ok(()) => {
                self.status = "Preferences saved".to_owned();
            }
            Err(error) => {
                self.status = format!("Could not save preferences: {error}");
            }
        }
    }

    fn show_menu(&mut self, root: &mut egui::Ui) {
        Panel::top("menu_bar").show_inside(root, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        ui.close();
                        self.request_action(PendingAction::New);
                    }
                    if ui.button("Open...").clicked() {
                        ui.close();
                        self.request_action(PendingAction::Open);
                    }
                    if ui.button("Save").clicked() {
                        ui.close();
                        self.save();
                    }
                    if ui.button("Save As...").clicked() {
                        ui.close();
                        self.save_as();
                    }
                    if ui.button("Quit").clicked() {
                        ui.close();
                        self.request_action(PendingAction::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Reflow Selection").clicked() {
                        ui.close();
                        self.reflow_selection();
                    }
                    if ui.button("Reflow Document").clicked() {
                        ui.close();
                        self.reflow_document();
                    }
                });

                ui.menu_button("Search", |ui| {
                    if ui.button("Find").clicked() {
                        ui.close();
                        self.search.focus = true;
                    }
                    if ui.button("Find Next").clicked() {
                        ui.close();
                        self.find_next();
                    }
                    if ui.button("Find Previous").clicked() {
                        ui.close();
                        self.find_previous();
                    }
                });
            });
        });
    }

    fn show_toolbar(&mut self, root: &mut egui::Ui) {
        Panel::top("toolbar").show_inside(root, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Find");
                let response = ui.add(
                    TextEdit::singleline(&mut self.search.query)
                        .desired_width(220.0)
                        .hint_text("literal text"),
                );

                if self.search.focus {
                    response.request_focus();
                    self.search.focus = false;
                }
                if response.changed() {
                    let matches = search::count_matches(&self.document.text, &self.search.query);
                    self.search.last_status = if self.search.query.is_empty() {
                        String::new()
                    } else {
                        format!("{matches} match{}", plural(matches))
                    };
                }
                if response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter)) {
                    self.find_next();
                }

                if ui.button("Next").clicked() {
                    self.find_next();
                }
                if ui.button("Previous").clicked() {
                    self.find_previous();
                }

                ui.separator();

                let mut config_changed = false;
                config_changed |= ui
                    .checkbox(&mut self.config.auto_wrap, "Auto wrap")
                    .changed();
                ui.label("Column");
                config_changed |= ui
                    .add(egui::DragValue::new(&mut self.config.wrap_column).range(20..=240))
                    .changed();

                if config_changed {
                    self.save_config();
                }

                if ui.button("Reflow").clicked() {
                    self.reflow_selection();
                }

                if !self.search.last_status.is_empty() {
                    ui.separator();
                    ui.label(&self.search.last_status);
                }
            });
        });
    }

    fn show_editor(&mut self, root: &mut egui::Ui, ctx: &egui::Context) {
        CentralPanel::default().show_inside(root, |ui| {
            let id = ui.make_persistent_id(EDITOR_ID);
            let row_height = ui.text_style_height(&TextStyle::Monospace).max(1.0);
            let editor_rows = (ui.available_height() / row_height).floor().max(12.0) as usize;

            let output = TextEdit::multiline(&mut self.document.text)
                .id(id)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .desired_rows(editor_rows)
                .lock_focus(true)
                .show(ui);

            if let Some(range) = output.cursor_range {
                let sorted = range.as_sorted_char_range();
                self.editor_cursor = range.primary.index;
                self.editor_selection = if sorted.start == sorted.end {
                    None
                } else {
                    Some(sorted)
                };
            }

            if output.response.changed() && self.config.auto_wrap {
                let (reflowed, cursor) = wrap::reflow_text_with_cursor(
                    &self.document.text,
                    self.editor_cursor,
                    self.config.wrap_column,
                );
                if reflowed != self.document.text {
                    self.document.text = reflowed;
                    self.editor_cursor = cursor;
                    self.pending_editor_selection = Some(cursor..cursor);
                    ctx.request_repaint();
                }
            }

            if let Some(range) = self.pending_editor_selection.take() {
                let mut state = output.state;
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::two(
                        egui::text::CCursor::new(range.start),
                        egui::text::CCursor::new(range.end),
                    )));
                state.store(ctx, output.response.id);
                output.response.request_focus();
                ctx.request_repaint();
            }
        });
    }

    fn show_status(&mut self, root: &mut egui::Ui) {
        Panel::bottom("status").show_inside(root, |ui| {
            ui.horizontal_wrapped(|ui| {
                let dirty = if self.document.dirty() {
                    "Edited"
                } else {
                    "Saved"
                };
                ui.label(dirty);
                ui.separator();
                ui.label(self.document.title());
                ui.separator();
                ui.label(format!("{} chars", self.document.text.chars().count()));
                ui.separator();
                ui.label(&self.status);
            });
        });
    }

    fn show_unsaved_prompt(&mut self, ctx: &egui::Context) {
        let Some(action) = self.pending_action else {
            return;
        };

        egui::Window::new("Unsaved changes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Save changes before continuing?");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.continue_pending_after_save(ctx);
                    }
                    if ui.button("Discard").clicked() {
                        self.pending_action = None;
                        if matches!(action, PendingAction::Close) {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        } else {
                            self.run_action(action);
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        self.pending_action = None;
                    }
                });
            });
    }
}

impl eframe::App for AlineaApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "{} - {}",
            self.document.title(),
            APP_NAME
        )));

        if ctx.input(|input| input.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.request_action(PendingAction::Close);
        }

        self.handle_shortcuts(&ctx);
        self.show_menu(ui);
        self.show_toolbar(ui);
        self.show_status(ui);
        self.show_editor(ui, &ctx);
        self.show_unsaved_prompt(&ctx);
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "es" }
}
