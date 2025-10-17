use eframe::{
    App,
    egui::{
        self, Align2, CentralPanel, Color32, FontId, Image, Pos2, Rect, Sense, SidePanel, Theme,
        Vec2, ViewportBuilder, include_image,
    },
};
use wazir_drop::{
    Color, ColoredPiece, Coord, Square,
    enums::{EnumMap, SimpleEnumExt},
};

fn main() {
    eframe::run_native(
        "Wazir Drop",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(egui::vec2(1024.0, 768.0)),
            ..Default::default()
        },
        Box::new(|ctx| Ok(Box::new(WazirDropApp::new(ctx)))),
    )
    .unwrap();
}

struct WazirDropApp {
    reverse: bool,
    piece_images: EnumMap<ColoredPiece, Image<'static>>,
}

impl App for WazirDropApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_theme(Theme::Light);

        SidePanel::right("side panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.checkbox(&mut self.reverse, "Reverse view");
            });

        CentralPanel::default().show(ctx, |ui| self.update_chessboard(ui));
    }
}

impl WazirDropApp {
    fn new(ctx: &eframe::CreationContext) -> Self {
        egui_extras::install_image_loaders(&ctx.egui_ctx);
        Self {
            reverse: false,
            piece_images: Self::piece_images(),
        }
    }

    fn piece_images() -> EnumMap<ColoredPiece, Image<'static>> {
        EnumMap::from_fn(|cpiece| {
            let image = match cpiece {
                ColoredPiece::RedAlfil => include_image!("../assets/white_bishop.svg"),
                ColoredPiece::BlueAlfil => include_image!("../assets/black_bishop.svg"),
                ColoredPiece::RedDabbaba => include_image!("../assets/white_rook.svg"),
                ColoredPiece::BlueDabbaba => include_image!("../assets/black_rook.svg"),
                ColoredPiece::RedFerz => include_image!("../assets/white_pawn.svg"),
                ColoredPiece::BlueFerz => include_image!("../assets/black_pawn.svg"),
                ColoredPiece::RedKnight => include_image!("../assets/white_knight.svg"),
                ColoredPiece::BlueKnight => include_image!("../assets/black_knight.svg"),
                ColoredPiece::RedWazir => include_image!("../assets/white_king.svg"),
                ColoredPiece::BlueWazir => include_image!("../assets/black_king.svg"),
            };
            Image::new(image)
        })
    }

    fn square_color(square: Square) -> Color32 {
        let coord = Coord::from(square);
        if (coord.x() + coord.y()).is_multiple_of(2) {
            Color32::from_rgb(220, 170, 100)
        } else {
            Color32::from_rgb(150, 75, 0)
        }
    }

    fn update_chessboard(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
        let size = response.rect.size();
        let tile_size =
            (size.x / (Coord::WIDTH + 2) as f32).min(size.y / (Coord::HEIGHT + 2) as f32);

        for y in 0..Coord::HEIGHT {
            let y_on_screen = if self.reverse {
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
            let x_on_screen = if self.reverse {
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

        for square in Square::all() {
            painter.rect_filled(
                self.square_rect(square, tile_size),
                0.0,
                Self::square_color(square),
            );
        }

        self.piece_images[ColoredPiece::RedWazir]
            .paint_at(ui, self.square_rect(Square::E1, tile_size));

        if response.clicked()
            && let Some(click) = response.interact_pointer_pos()
        {
            let x = (click.x / tile_size - 1.0).floor();
            let y = (click.y / tile_size - 1.0).floor();
            if x >= 0.0 && x < Coord::WIDTH as f32 && y >= 0.0 && y < Coord::HEIGHT as f32 {
                let square =
                    Square::from_coord(Coord::new(x as usize, y as usize)).pov(self.pov_color());
                self.click(square);
            }
        }
    }

    fn square_rect(&self, square: Square, tile_size: f32) -> Rect {
        let coord = Coord::from(square.pov(self.pov_color()));
        Rect::from_min_size(
            Pos2::new(
                (coord.x() + 1) as f32 * tile_size,
                (coord.y() + 1) as f32 * tile_size,
            ),
            Vec2::new(tile_size, tile_size),
        )
    }

    fn pov_color(&self) -> Color {
        if self.reverse {
            Color::Blue
        } else {
            Color::Red
        }
    }

    fn click(&mut self, square: Square) {
        println!("Clicked on square {square}");
    }
}
