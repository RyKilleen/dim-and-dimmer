mod app;
mod commands;
mod ddc;
mod display;
mod gamma;
mod ui;

use eframe::egui;

use app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([450.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DimAndDimmer",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}
