use std::{
    error::Error,
    process::ExitCode,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use eframe::{
    App,
    egui::{
        self, Align2, CentralPanel, Color32, FontId, Image, Pos2, Rect, ScrollArea, Sense,
        SidePanel, Theme, Ui, Vec2, ViewportBuilder, include_image,
    },
};
use extra::moverand;
use rand::{SeedableRng, rngs::StdRng};
use simplelog::{ColorChoice, LevelFilter, TermLogger, TerminalMode};
use wazir_drop::{
    Color, ColoredPiece, Coord, LinearEvaluator, Move, Piece, PieceSquareFeatures, Position,
    Search, SetupMove, ShortMove, ShortMoveFrom, Square, Stage, Symmetry,
    enums::{EnumMap, SimpleEnumExt},
    movegen,
};

fn main() -> ExitCode {
    if let Err(e) = run() {
        log::error!("{e}");
        eprintln!("Error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), Box<dyn Error>> {
    wazir_drop::log::init(wazir_drop::log::Level::Info);
    TermLogger::init(
        LevelFilter::Info,
        simplelog::Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )?;

    eframe::run_native(
        "Wazir Drop",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(egui::vec2(1024.0, 768.0)),
            ..Default::default()
        },
        Box::new(|ctx| Ok(Box::new(WazirDropApp::new(ctx)))),
    )?;

    Ok(())
}

struct WazirDropApp {
    reverse: bool,
    is_computer_player: EnumMap<Color, bool>,
    time_limit_str: String,
    piece_images: EnumMap<ColoredPiece, Image<'static>>,
    tile_size: f32,
    position: Position,
    next_move_state: NextMoveState,
    history: Vec<HistoryEntry>,
    rng: Arc<Mutex<StdRng>>,
    search: Arc<Mutex<Search<LinearEvaluator<PieceSquareFeatures>>>>,
}

impl WazirDropApp {
    fn new(ctx: &eframe::CreationContext) -> Self {
        egui_extras::install_image_loaders(&ctx.egui_ctx);
        let mut app = Self {
            reverse: false,
            is_computer_player: EnumMap::from_fn(|_| false),
            time_limit_str: "1000".to_string(),
            piece_images: Self::piece_images(),
            tile_size: 0.0,
            position: Position::initial(),
            next_move_state: NextMoveState::EndOfGame, // temporary
            history: Vec::new(),
            rng: Arc::new(Mutex::new(StdRng::from_os_rng())),
            search: Arc::new(Mutex::new(Search::new(&Arc::new(
                LinearEvaluator::default(),
            )))),
        };
        app.start_next_move(&ctx.egui_ctx);
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

    fn is_dark_square(square: Square) -> bool {
        let coord = Coord::from(square);
        !(coord.x() + coord.y()).is_multiple_of(2)
    }

    fn square_color(square: Square) -> Color32 {
        if Self::is_dark_square(square) {
            Color32::from_rgb(185, 136, 102)
        } else {
            Color32::from_rgb(242, 217, 183)
        }
    }

    fn selected_square_color(square: Square) -> Color32 {
        if Self::is_dark_square(square) {
            Color32::from_rgb(99, 111, 67)
        } else {
            Color32::from_rgb(128, 151, 108)
        }
    }

    fn last_move_square_color(square: Square) -> Color32 {
        if Self::is_dark_square(square) {
            Color32::from_rgb(100, 100, 100)
        } else {
            Color32::from_rgb(130, 130, 130)
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
        let coord = Coord::from(self.symmetry().apply(square));
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
            _ = ui.painter().text(
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
            _ = ui.painter().text(
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
            _ => self.position.clone(),
        };

        for square in Square::all() {
            let rect = self.square_rect(square);
            if ui.allocate_rect(rect, Sense::click()).clicked() {
                self.click_square(square, ui.ctx());
            }
            let is_selected = match self.next_move_state {
                NextMoveState::HumanRegular { from: Some(from) } => {
                    let short_move = ShortMove::Regular { from, to: square };
                    from == ShortMoveFrom::Square(square)
                        || movegen::move_from_short_move(&self.position, short_move).is_ok()
                }
                NextMoveState::HumanSetup {
                    swap_from: Some(swap_from),
                    ..
                } => swap_from == square,
                _ => false,
            };
            let is_last_move = match self.history.last() {
                Some(HistoryEntry {
                    mov: Move::Regular(mov),
                    ..
                }) => mov.from == Some(square) || mov.to == square,
                _ => false,
            };
            let color = if is_selected {
                Self::selected_square_color(square)
            } else if is_last_move {
                Self::last_move_square_color(square)
            } else {
                Self::square_color(square)
            };
            _ = ui.painter().rect_filled(rect, 0.0, color);
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
            _ = ui.painter().rect_filled(rect, 0.0, color);
            let num = self.position.num_captured(cpiece);
            if num > 0 {
                self.draw_captured_piece(ui, cpiece, num);
            }
        }
    }

    fn start_next_move(&mut self, ctx: &egui::Context) {
        self.next_move_state = match (
            self.position.stage(),
            self.is_computer_player[self.position.to_move()],
        ) {
            (Stage::End(_), _) => NextMoveState::EndOfGame,
            (Stage::Setup, false) => NextMoveState::HumanSetup {
                setup: movegen::setup_moves(self.position.to_move())
                    .next()
                    .unwrap(),
                swap_from: None,
            },
            (Stage::Regular, false) => NextMoveState::HumanRegular { from: None },
            (_, true) => {
                let result = Arc::new(Mutex::new(None));
                self.launch_computer_thread(ctx, result.clone());
                NextMoveState::Computer { result }
            }
        };
    }

    fn launch_computer_thread(&mut self, ctx: &egui::Context, result: Arc<Mutex<Option<Move>>>) {
        let position = self.position.clone();
        let rng = self.rng.clone();
        let search = self.search.clone();
        let ctx = ctx.clone();
        let time_limit_ms = self.time_limit_str.parse::<u32>().unwrap_or(1000);

        _ = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_millis(time_limit_ms.into());
            let mov = match position.stage() {
                Stage::Setup => {
                    moverand::random_setup(position.to_move(), &mut rng.lock().unwrap()).into()
                }
                Stage::Regular => {
                    let result =
                        search
                            .lock()
                            .unwrap()
                            .search_regular(&position, None, Some(deadline));
                    log::info!(
                        "depth {depth} score {score} \
                            root {root_moves_considered}/{root_all_moves} \
                            nodes {nodes} pv {pv}",
                        depth = result.depth,
                        score = result.score,
                        root_moves_considered = result.root_moves_considered,
                        root_all_moves = result.root_all_moves,
                        nodes = result.nodes,
                        pv = result.pv,
                    );
                    result.pv.moves[0].into()
                }
                Stage::End(_) => panic!("Game is over"),
            };
            *result.lock().unwrap() = Some(mov);
            ctx.request_repaint();
        });
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
            _ = ui
                .painter()
                .circle_filled(pos, 0.2 * self.tile_size, Color32::GREEN);
            _ = ui.painter().text(
                pos,
                Align2::CENTER_CENTER,
                num.to_string(),
                FontId::monospace(0.4 * self.tile_size),
                Color32::BLACK,
            );
        }
    }

    fn draw_to_move(&self, ui: &mut Ui) {
        if !matches!(self.position.stage(), Stage::End(_)) {
            let x = 1.1 * self.tile_size;
            let y = if (self.position.to_move() == Color::Red) != self.reverse {
                0.8 * self.tile_size
            } else {
                ((Coord::HEIGHT + 1) as f32 + 0.2) * self.tile_size
            };
            let color = match self.position.to_move() {
                Color::Red => Color32::WHITE,
                Color::Blue => Color32::BLACK,
            };
            let pos = Pos2::new(x, y);
            _ = ui
                .painter()
                .circle_filled(pos, 0.12 * self.tile_size, Color32::BLACK);
            _ = ui.painter().circle_filled(pos, 0.1 * self.tile_size, color);
        }
    }

    fn draw_history(&self, ui: &mut Ui) {
        _ = ui.heading("Moves");
        _ = ScrollArea::vertical().show(ui, |ui| {
            for (index, entry) in self.history.iter().enumerate() {
                _ = ui.label(format!("{}. {}", index + 1, entry.mov));
            }
        });
    }

    fn symmetry(&self) -> Symmetry {
        if self.reverse {
            Symmetry::Rotate180
        } else {
            Symmetry::Identity
        }
    }

    fn click_square(&mut self, square: Square, ctx: &egui::Context) {
        match self.next_move_state {
            NextMoveState::HumanSetup {
                ref mut setup,
                ref mut swap_from,
            } => {
                let piece_index = Symmetry::pov(setup.color).inverse().apply(square).index();
                if piece_index < SetupMove::SIZE {
                    match *swap_from {
                        None => {
                            *swap_from = Some(square);
                        }
                        Some(swap_from_square) => {
                            let swap_from_index = Symmetry::pov(setup.color)
                                .inverse()
                                .apply(swap_from_square)
                                .index();
                            setup.pieces.swap(swap_from_index, piece_index);
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
                    if let Ok(mov) = movegen::move_from_short_move(&self.position, short_move) {
                        self.make_move(mov, ctx);
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

    fn make_move(&mut self, mov: Move, ctx: &egui::Context) {
        self.history.push(HistoryEntry {
            position: self.position.clone(),
            mov,
        });
        self.position = self.position.make_move(mov).expect("Invalid move");
        self.start_next_move(ctx);
    }

    fn new_game(&mut self, ctx: &egui::Context) {
        if !matches!(self.next_move_state, NextMoveState::Computer { .. }) {
            self.position = Position::initial();
            self.history.clear();
            self.start_next_move(ctx);
        }
    }

    fn undo(&mut self, ctx: &egui::Context) {
        if !matches!(self.next_move_state, NextMoveState::Computer { .. })
            && let Some(entry) = self.history.pop()
        {
            self.position = entry.position;
            self.start_next_move(ctx);
        }
    }
}

impl App for WazirDropApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_zoom_factor(1.5);
        ctx.set_theme(Theme::Light);

        let mut computer_move = None;
        if let NextMoveState::Computer { result } = &self.next_move_state {
            computer_move = result.lock().unwrap().take();
        }
        if let Some(mov) = computer_move {
            self.make_move(mov, ctx);
        }

        _ = SidePanel::right("side panel").show(ctx, |ui| {
            _ = ui.checkbox(&mut self.reverse, "Reverse view");

            for color in Color::all() {
                if ui
                    .checkbox(
                        &mut self.is_computer_player[color],
                        format!("Computer player {color}"),
                    )
                    .changed()
                    && self.position.to_move() == color
                    && !matches!(self.next_move_state, NextMoveState::Computer { .. })
                {
                    self.start_next_move(ctx);
                }
            }

            _ = ui.label("Time limit (ms):");
            _ = ui.text_edit_singleline(&mut self.time_limit_str);

            if let NextMoveState::Computer { .. } = self.next_move_state {
                _ = ui.label("Thinking...");
            } else {
                if ui.button("New Game").clicked() {
                    self.new_game(ctx);
                }

                if !self.history.is_empty() && ui.button("Undo").clicked() {
                    self.undo(ctx);
                }
            }

            if let NextMoveState::HumanSetup { setup, .. } = &self.next_move_state
                && ui.button("Make setup move").clicked()
            {
                self.make_move(Move::Setup(*setup), ctx);
            }

            if let Stage::End(outcome) = self.position.stage() {
                _ = ui.label(outcome.to_string());
            }

            self.draw_history(ui);
        });

        _ = CentralPanel::default().show(ctx, |ui| self.update_chessboard(ui));
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
    Computer {
        result: Arc<Mutex<Option<Move>>>,
    },
    EndOfGame,
}

#[derive(Debug)]
struct HistoryEntry {
    position: Position,
    mov: Move,
}
