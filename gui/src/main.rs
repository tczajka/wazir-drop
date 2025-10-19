use eframe::{
    App,
    egui::{
        self, Align2, CentralPanel, Color32, FontId, Image, Pos2, Rect, Sense, SidePanel, Theme,
        Ui, Vec2, ViewportBuilder, include_image,
    },
};
use std::str::FromStr;
use wazir_drop::{
    Color, ColoredPiece, Coord, Piece, Position, ShortMoveFrom, Square, Stage,
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

#[derive(Debug)]
struct WazirDropApp {
    reverse: bool,
    piece_images: EnumMap<ColoredPiece, Image<'static>>,
    tile_size: f32,
    position: Position,
}

impl WazirDropApp {
    fn new(ctx: &eframe::CreationContext) -> Self {
        egui_extras::install_image_loaders(&ctx.egui_ctx);
        Self {
            reverse: false,
            piece_images: Self::piece_images(),
            tile_size: 0.0,
            position: Position::from_str(
                "\
regular
red
AFF
f
.W.A.D.D
Aa.A.DDA
..A.A.A.
......A.
...a.a.d
..d..nN.
a.a...f.
add.w..a
",
            )
            .unwrap(),
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
        self.tile_size = (size.x / (Coord::WIDTH + Piece::COUNT + 3) as f32)
            .min(size.y / (Coord::HEIGHT + 2) as f32);

        self.draw_coordinates(ui);
        self.update_board(ui);
        self.update_captured(ui);
        self.draw_to_move(ui);
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

    fn captured_rect(&self, color: Color, piece: Piece) -> Rect {
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

    fn update_board(&mut self, ui: &mut Ui) {
        for square in Square::all() {
            let rect = self.square_rect(square);
            if ui.allocate_rect(rect, Sense::click()).clicked() {
                self.click_square(square);
            }
            ui.painter()
                .rect_filled(rect, 0.0, Self::square_color(square));
            if let Some(cpiece) = self.position.square(square) {
                self.draw_piece(ui, square, cpiece);
            }
        }
    }

    fn update_captured(&mut self, ui: &mut Ui) {
        for color in Color::all() {
            for piece in Piece::all() {
                let rect = self.captured_rect(color, piece);
                if ui.allocate_rect(rect, Sense::click()).clicked() {
                    self.click_captured(color, piece);
                }
                ui.painter().rect_filled(
                    rect,
                    0.0,
                    Self::square_color(Square::from_index(piece.index())),
                );
                let num = self.position.num_captured(color, piece);
                if num > 0 {
                    self.draw_captured_piece(ui, color, piece, num);
                }
            }
        }
    }

    fn draw_piece(&self, ui: &mut Ui, square: Square, piece: ColoredPiece) {
        self.piece_images[piece].paint_at(ui, self.square_rect(square));
    }

    fn draw_captured_piece(&self, ui: &mut Ui, color: Color, piece: Piece, num: usize) {
        let image = &self.piece_images[piece.with_color(color)];
        let rect = self.captured_rect(color, piece);
        image.paint_at(ui, rect);
        if num > 1 {
            let pos = rect.left_top() + 0.8 * rect.size();
            ui.painter()
                .circle_filled(pos, 0.2 * self.tile_size, Color32::GREEN);
            ui.painter().text(
                pos,
                Align2::CENTER_CENTER,
                num.to_string(),
                FontId::monospace(0.4 * self.tile_size),
                Color32::BLACK,
            );
        }
    }

    fn draw_to_move(&self, ui: &mut Ui) {
        if self.position.stage() != Stage::End {
            let x = 1.1 * self.tile_size;
            let y = if self.position.to_move() == self.pov_color() {
                0.8 * self.tile_size
            } else {
                ((Coord::HEIGHT + 1) as f32 + 0.2) * self.tile_size
            };
            let color = match self.position.to_move() {
                Color::Red => Color32::WHITE,
                Color::Blue => Color32::BLACK,
            };
            let pos = Pos2::new(x, y);
            ui.painter()
                .circle_filled(pos, 0.12 * self.tile_size, Color32::BLACK);
            ui.painter().circle_filled(pos, 0.1 * self.tile_size, color);
        }
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

    fn click_captured(&mut self, color: Color, piece: Piece) {
        println!(
            "Clicked on captured {piece}",
            piece = piece.with_color(color)
        );
    }
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

#[derive(Debug)]
enum NextMoveState {
    HumanRegular { from: Option<ShortMoveFrom> },
    EndOfGame,
}
