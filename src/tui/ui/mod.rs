mod layout;
mod menu;
mod table;

use crate::tui::app::{AppState, Scene};
use ratatui::prelude::Frame;

pub fn draw(f: &mut Frame, app: &AppState) {
    match app.scene {
        Scene::Menu => menu::draw_menu(f, app),
        Scene::Table => table::draw_table(f, app),
    }
}
