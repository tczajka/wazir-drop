use eframe::{
    App,
    egui::{
        self, Align2, CentralPanel, Color32, FontId, Pos2, Rect, Sense, SidePanel, Theme, Vec2,
        ViewportBuilder,
    },
};
use wazir_drop::{Coord, Square, enums::SimpleEnumExt};

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

struct WazirDropApp {
    rotated: bool,
}

impl App for WazirDropApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_theme(Theme::Light);

        SidePanel::right("side panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.checkbox(&mut self.rotated, "Rotate view");
            });

        CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::empty());
            let size = response.rect.size();
            let tile_size =
                (size.x / (Coord::WIDTH + 2) as f32).min(size.y / (Coord::HEIGHT + 2) as f32);
            for square in Square::all() {
                let square_on_screen = if self.rotated {
                    square.rotate()
                } else {
                    square
                };
                let coord = Coord::from(square_on_screen);
                let rect = Rect::from_min_size(
                    Pos2::new(
                        (coord.x() + 1) as f32 * tile_size,
                        (coord.y() + 1) as f32 * tile_size,
                    ),
                    Vec2::new(tile_size, tile_size),
                );
                painter.rect_filled(rect, 0.0, Self::square_color(square));
            }
            for y in 0..Coord::HEIGHT {
                let y_on_screen = if self.rotated {
                    Coord::HEIGHT - y - 1
                } else {
                    y
                };
                let name = char::from(b'a' + y as u8);
                painter.text(
                    Pos2::new(0.7 * tile_size, (y_on_screen as f32 + 1.5) * tile_size),
                    Align2::CENTER_CENTER,
                    name,
                    FontId::monospace(0.3 * tile_size),
                    Color32::BLACK,
                );
            }
            for x in 0..Coord::WIDTH {
                let x_on_screen = if self.rotated {
                    Coord::WIDTH - x - 1
                } else {
                    x
                };
                let name = char::from(b'1' + x as u8);
                painter.text(
                    Pos2::new((x_on_screen as f32 + 1.5) * tile_size, 0.7 * tile_size),
                    Align2::CENTER_CENTER,
                    name,
                    FontId::monospace(0.3 * tile_size),
                    Color32::BLACK,
                );
            }
        });
    }
}

impl WazirDropApp {
    fn new() -> Self {
        Self { rotated: false }
    }

    fn square_color(square: Square) -> Color32 {
        let coord = Coord::from(square);
        if (coord.x() + coord.y()).is_multiple_of(2) {
            Color32::from_rgb(220, 170, 100)
        } else {
            Color32::from_rgb(150, 75, 0)
        }
    }
}
