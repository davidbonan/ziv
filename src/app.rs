use crate::library::ui::empty_state::empty_state;

const APP_NAME: &str = "Ziv";

#[derive(Default)]
pub struct ZivApp;

impl eframe::App for ZivApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, empty_state);
    }
}

pub fn run() -> eframe::Result<()> {
    eframe::run_native(
        APP_NAME,
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(ZivApp))),
    )
}
