use eframe::{
    App,
    egui::{
        self, Align2, CentralPanel, Color32, FontId, Image, Pos2, Rect, Sense, SidePanel, Theme,
        Ui, Vec2, ViewportBuilder, include_image,
    },
};
use wazir_drop::{
    Color, ColoredPiece, Coord, Piece, PieceNonWazir, Square,
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
    tile_size: f32,
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
            tile_size: 0.0,
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

    fn update_chessboard(&mut self, ui: &mut Ui) {
        let size = ui.available_size();
        self.tile_size = (size.x / (Coord::WIDTH + PieceNonWazir::COUNT + 3) as f32)
            .min(size.y / (Coord::HEIGHT + 2) as f32);

        self.draw_coordinates(ui);
        self.update_squares(ui);
        self.update_captured(ui);

        // test
        self.draw_piece(ui, Square::E1, ColoredPiece::RedWazir);
    }

    fn square_rect(&self, square: Square) -> Rect {
        let coord = Coord::from(square.pov(self.pov_color()));
        Rect::from_min_size(
            Pos2::new(
                (coord.x() + 1) as f32 * self.tile_size,
                (coord.y() + 1) as f32 * self.tile_size,
            ),
            Vec2::new(self.tile_size, self.tile_size),
        )
    }

    fn captured_rect(&self, color: Color, piece: PieceNonWazir) -> Rect {
        let x = Coord::WIDTH + 2 + piece.index();
        let y = 1
            + (Coord::HEIGHT - 1)
                * (if self.reverse {
                    color.opposite().index()
                } else {
                    color.index()
                });

        Rect::from_min_size(
            Pos2::new(x as f32 * self.tile_size, y as f32 * self.tile_size),
            Vec2::new(self.tile_size, self.tile_size),
        )
    }

    fn draw_coordinates(&self, ui: &mut Ui) {
        for y in 0..Coord::HEIGHT {
            let y_on_screen = if self.reverse {
                Coord::HEIGHT - y - 1
            } else {
                y
            };
            let name = char::from(b'a' + y as u8);
            ui.painter().text(
                Pos2::new(
                    0.7 * self.tile_size,
                    (y_on_screen as f32 + 1.5) * self.tile_size,
                ),
                Align2::CENTER_CENTER,
                name,
                FontId::monospace(0.3 * self.tile_size),
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
            ui.painter().text(
                Pos2::new(
                    (x_on_screen as f32 + 1.5) * self.tile_size,
                    0.7 * self.tile_size,
                ),
                Align2::CENTER_CENTER,
                name,
                FontId::monospace(0.3 * self.tile_size),
                Color32::BLACK,
            );
        }
    }

    fn update_squares(&mut self, ui: &mut Ui) {
        for square in Square::all() {
            let rect = self.square_rect(square);
            if ui.allocate_rect(rect, Sense::click()).clicked() {
                self.click_square(square);
            }
            ui.painter()
                .rect_filled(rect, 0.0, Self::square_color(square));
        }
    }

    fn update_captured(&mut self, ui: &mut Ui) {
        for color in Color::all() {
            for piece in PieceNonWazir::all() {
                let rect = self.captured_rect(color, piece);
                if ui.allocate_rect(rect, Sense::click()).clicked() {
                    self.click_captured(color, piece);
                }
                ui.painter().rect_filled(
                    rect,
                    0.0,
                    Self::square_color(Square::from_index(piece.index())),
                );
            }
        }
    }

    fn draw_piece(&self, ui: &mut Ui, square: Square, piece: ColoredPiece) {
        self.piece_images[piece].paint_at(ui, self.square_rect(square));
    }

    fn pov_color(&self) -> Color {
        if self.reverse {
            Color::Blue
        } else {
            Color::Red
        }
    }

    fn click_square(&mut self, square: Square) {
        println!("Clicked on square {square}");
    }

    fn click_captured(&mut self, color: Color, piece: PieceNonWazir) {
        println!(
            "Clicked on captured {piece}",
            piece = Piece::from(piece).with_color(color)
        );
    }
}
