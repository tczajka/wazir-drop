use eframe::{
    App,
    egui::{self, CentralPanel, SidePanel, Theme, ViewportBuilder},
};

fn main() {
    eframe::run_native(
        "Wazir Drop",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(egui::vec2(1024.0, 768.0)),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(WazirDropApp::new()))),
    )
    .unwrap();
}

struct WazirDropApp;

impl App for WazirDropApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_theme(Theme::Light);
        SidePanel::right("side panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Side panel");
            });
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Central panel");
        });
    }
}

impl WazirDropApp {
    fn new() -> Self {
        Self
    }
}
