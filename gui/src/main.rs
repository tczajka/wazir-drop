use eframe::{
    App,
    egui::{
        self, Align2, CentralPanel, Color32, FontId, Image, Pos2, Rect, ScrollArea, Sense,
        SidePanel, Theme, Ui, Vec2, ViewportBuilder, include_image,
    },
};
use wazir_drop::{
    Color, ColoredPiece, Coord, Move, Piece, Position, SetupMove, ShortMove, ShortMoveFrom, Square,
    Stage,
    enums::{EnumMap, SimpleEnumExt},
    movegen,
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
    next_move_state: NextMoveState,
    history: Vec<HistoryEntry>,
}

impl WazirDropApp {
    fn new(ctx: &eframe::CreationContext) -> Self {
        egui_extras::install_image_loaders(&ctx.egui_ctx);
        let mut app = Self {
            reverse: false,
            piece_images: Self::piece_images(),
            tile_size: 0.0,
            position: Position::initial(),
            next_move_state: NextMoveState::EndOfGame, // temporary
            history: Vec::new(),
        };
        app.start_next_move();
        app
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
            Color32::from_rgb(200, 170, 100)
        } else {
            Color32::from_rgb(150, 75, 0)
        }
    }

    fn selected_square_color(square: Square) -> Color32 {
        let coord = Coord::from(square);
        if (coord.x() + coord.y()).is_multiple_of(2) {
            Color32::from_rgb(200, 170, 250)
        } else {
            Color32::from_rgb(150, 75, 150)
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

    fn captured_rect(&self, cpiece: ColoredPiece) -> Rect {
        let x = Coord::WIDTH + 2 + cpiece.piece().index();
        let y = 1
            + (Coord::HEIGHT - 1)
                * (if self.reverse {
                    cpiece.color().opposite().index()
                } else {
                    cpiece.color().index()
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
        let position = match self.next_move_state {
            NextMoveState::HumanSetup { setup, .. } => self
                .position
                .make_setup_move(setup)
                .expect("Invalid setup move"),
            _ => self.position,
        };

        for square in Square::all() {
            let rect = self.square_rect(square);
            if ui.allocate_rect(rect, Sense::click()).clicked() {
                self.click_square(square);
            }
            let selected = match self.next_move_state {
                NextMoveState::HumanRegular { from: Some(from) } => {
                    let short_move = ShortMove::Regular { from, to: square };
                    from == ShortMoveFrom::Square(square)
                        || self.position.move_from_short_move(short_move).is_ok()
                }
                NextMoveState::HumanSetup {
                    swap_from: Some(swap_from),
                    ..
                } => swap_from == square,
                _ => false,
            };
            let color = if selected {
                Self::selected_square_color(square)
            } else {
                Self::square_color(square)
            };
            ui.painter().rect_filled(rect, 0.0, color);
            if let Some(cpiece) = position.square(square) {
                self.draw_piece(ui, square, cpiece);
            }
        }
    }

    fn update_captured(&mut self, ui: &mut Ui) {
        for cpiece in ColoredPiece::all() {
            let rect = self.captured_rect(cpiece);
            if ui.allocate_rect(rect, Sense::click()).clicked() {
                self.click_captured(cpiece);
            }
            let selected = match self.next_move_state {
                NextMoveState::HumanRegular {
                    from: Some(ShortMoveFrom::Piece(from_cpiece)),
                } => cpiece == from_cpiece,
                _ => false,
            };
            let square = Square::from_index(cpiece.piece().index());
            let color = if selected {
                Self::selected_square_color(square)
            } else {
                Self::square_color(square)
            };
            ui.painter().rect_filled(rect, 0.0, color);
            let num = self.position.num_captured(cpiece);
            if num > 0 {
                self.draw_captured_piece(ui, cpiece, num);
            }
        }
    }

    fn start_next_move(&mut self) {
        self.next_move_state = match self.position.stage() {
            Stage::Setup => NextMoveState::HumanSetup {
                setup: movegen::setup_moves(self.position.to_move())
                    .next()
                    .unwrap(),
                swap_from: None,
            },
            Stage::Regular => NextMoveState::HumanRegular { from: None },
            Stage::End => NextMoveState::EndOfGame,
        };
    }

    fn draw_piece(&self, ui: &mut Ui, square: Square, piece: ColoredPiece) {
        self.piece_images[piece].paint_at(ui, self.square_rect(square));
    }

    fn draw_captured_piece(&self, ui: &mut Ui, cpiece: ColoredPiece, num: usize) {
        let image = &self.piece_images[cpiece];
        let rect = self.captured_rect(cpiece);
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

    fn draw_history(&self, ui: &mut Ui) {
        ui.heading("Moves");
        ScrollArea::vertical().show(ui, |ui| {
            for entry in self.history.iter() {
                ui.label(entry.mov.to_string());
            }
        });
    }

    fn pov_color(&self) -> Color {
        if self.reverse {
            Color::Blue
        } else {
            Color::Red
        }
    }

    fn click_square(&mut self, square: Square) {
        match self.next_move_state {
            NextMoveState::HumanSetup {
                ref mut setup,
                ref mut swap_from,
            } => {
                if square.pov(setup.color).index() < SetupMove::SIZE {
                    match *swap_from {
                        None => {
                            *swap_from = Some(square);
                        }
                        Some(swap_from_square) => {
                            setup.pieces.swap(
                                swap_from_square.pov(setup.color).index(),
                                square.pov(setup.color).index(),
                            );
                            *swap_from = None;
                        }
                    }
                }
            }
            NextMoveState::HumanRegular { ref mut from } => {
                if let Some(cpiece) = self.position.square(square)
                    && cpiece.color() == self.position.to_move()
                {
                    if *from == Some(ShortMoveFrom::Square(square)) {
                        *from = None;
                    } else {
                        *from = Some(ShortMoveFrom::Square(square));
                    }
                } else if let Some(from_square) = *from {
                    let short_move = ShortMove::Regular {
                        from: from_square,
                        to: square,
                    };
                    if let Ok(mov) = self.position.move_from_short_move(short_move) {
                        self.make_move(mov);
                    }
                }
            }
            _ => {}
        }
    }

    fn click_captured(&mut self, cpiece: ColoredPiece) {
        if let NextMoveState::HumanRegular { ref mut from } = self.next_move_state
            && cpiece.color() == self.position.to_move()
            && self.position.num_captured(cpiece) > 0
        {
            if *from == Some(ShortMoveFrom::Piece(cpiece)) {
                *from = None;
            } else {
                *from = Some(ShortMoveFrom::Piece(cpiece));
            }
        }
    }

    fn make_move(&mut self, mov: Move) {
        self.history.push(HistoryEntry {
            position: self.position,
            mov,
        });
        self.position = self.position.make_move(mov).expect("Invalid move");
        self.start_next_move();
    }

    fn new_game(&mut self) {
        self.position = Position::initial();
        self.history.clear();
        self.start_next_move();
    }

    fn undo(&mut self) {
        if let Some(entry) = self.history.pop() {
            self.position = entry.position;
            self.start_next_move();
        }
    }
}

impl App for WazirDropApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_theme(Theme::Light);

        SidePanel::right("side panel").show(ctx, |ui| {
            ui.checkbox(&mut self.reverse, "Reverse view");

            if ui.button("New Game").clicked() {
                self.new_game();
            }

            if let NextMoveState::HumanSetup { setup, .. } = &self.next_move_state
                && ui.button("Make setup move").clicked()
            {
                self.make_move(Move::Setup(*setup));
            }

            if !self.history.is_empty() && ui.button("Undo").clicked() {
                self.undo();
            }

            self.draw_history(ui);
        });

        CentralPanel::default().show(ctx, |ui| self.update_chessboard(ui));
    }
}

#[derive(Debug)]
enum NextMoveState {
    HumanSetup {
        setup: SetupMove,
        swap_from: Option<Square>,
    },
    HumanRegular {
        from: Option<ShortMoveFrom>,
    },
    EndOfGame,
}

#[derive(Debug)]
struct HistoryEntry {
    position: Position,
    mov: Move,
}
