use eframe::{
    App,
    egui::{
        self, CentralPanel, Color32, Pos2, Rect, Sense, SidePanel, Theme, Vec2, ViewportBuilder,
    },
};
use wazir_drop::Coord;

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
            /*
            let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::empty());
            let size = response.rect.size();
            let tile_size =
                (size.x / (Coord::WIDTH + 2) as f32).min(size.y / (Coord::HEIGHT + 2) as f32);
            for y in 0..Coord::HEIGHT {
                for x in 0..Coord::WIDTH {
                    let rect = Rect::from_min_size(
                        Pos2::new((x + 1) as f32 * tile_size, (y + 1) as f32 * tile_size),
                        Vec2::new(tile_size, tile_size),
                    );
                    painter.rect_filled(rect, 0.0, Self::square_color(x, y));
                }
            }
            */
        });
    }
}

impl WazirDropApp {
    fn new() -> Self {
        Self
    }

    fn square_color(x: usize, y: usize) -> Color32 {
        if (x + y).is_multiple_of(2) {
            Color32::from_rgb(255, 0, 0)
        } else {
            Color32::from_rgb(0, 0, 255)
        }
    }
}
