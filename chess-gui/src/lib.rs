#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

// imports
use chess::{self, BitBoard, Board, ChessMove, File, Game, MoveGen, Piece, Rank, Square};
use std::str::FromStr;
use std::usize;

use chess_ai::Bot;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;
use ggez::filesystem;
use ggez::graphics::{self, Color};
use ggez::input::{keyboard, mouse};
use ggez::timer;
use ggez::{Context, GameResult};
use std::env;
use std::io::{stdin, stdout, Write};
use std::path;

// constants
const WINDWOW_SIZE: f32 = 800.;
const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// helper functions for canvas & board conversion
fn canvas_square_to_board_square(square: (i16, i16), pov: u8) -> Square {
    let (x, y) = (square.0 as usize, square.1 as usize);
    let (x, y) = if pov == 1 { (x, 7 - y) } else { (7 - x, y) };
    Square::make_square(Rank::from_index(y), File::from_index(x))
}

fn board_square_to_canvas_square(square: &Square, pov: u8) -> (f32, f32) {
    let idx = Square::to_int(square);
    if pov == 1 {
        ((idx % 8) as f32, 7. - (idx / 8) as f32)
    } else {
        (7. - (idx % 8) as f32, (idx / 8) as f32)
    }
}

fn canvas_coord_to_canvas_square(x: i16, y: i16, pov: u8) -> (i16, i16) {
    let file = x / (WINDWOW_SIZE as i16 / 8);
    let rank = y / (WINDWOW_SIZE as i16 / 8);

    (file, rank)
}

// helper functions for move generation, to display the legal moves
fn movegen_empty() -> Vec<ChessMove> {
    let game: Game = Game::from_str(STARTING_FEN).expect("Valid FEN");
    let mut empty = MoveGen::new_legal(&game.current_position());
    empty.remove_mask(BitBoard::new(u64::MAX));
    empty.collect()
}

fn movegen(board: &Board, start_square: Square, color_to_move: chess::Color) -> Vec<ChessMove> {
    match board.color_on(start_square) {
        None => movegen_empty(),
        Some(color) => MoveGen::new_legal(board)
            .filter(|m| m.get_source() == start_square)
            .collect(),
    }
}

// holds the state of the current game
pub struct GameState {
    pov: u8,
    flip_timeout: u16,
    field_selected: bool,
    field: (i16, i16),
    game: Game,
    current_legal_moves: Vec<ChessMove>,
    playable: [bool; 2],
    bot_refs: [Bot; 2],
}

impl GameState {
    pub fn new(game: Game, playable: [bool; 2], bot_refs: [Bot; 2]) -> GameState {
        let pov = if !playable[0] && playable[1] { 2 } else { 1 };
        let s = GameState {
            pov,
            flip_timeout: 0,
            field_selected: false,
            field: (-1, -1),
            game,
            current_legal_moves: movegen_empty(),
            playable,
            bot_refs,
        };

        s
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    // gets called on update events
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if !self.game.result().is_none() {
            return Ok(());
        }
        // variable to index into playable and bot_ref arrays
        let current_player_as_idx = if self.game.side_to_move() == chess::Color::White {
            0
        } else {
            1
        } as usize;

        // flip board orientation, set timeout to make sure it doesn't switch back and forth too quickly
        if self.flip_timeout == 0 && keyboard::is_key_pressed(ctx, event::KeyCode::F) {
            self.pov = (self.pov % 2) + 1;
            self.flip_timeout = 10;
            self.current_legal_moves = movegen_empty();
            self.field_selected = false;
        }

        // player clicks on a square
        if mouse::button_pressed(ctx, mouse::MouseButton::Left) {
            // if current player is not a bot
            if self.playable[current_player_as_idx] {
                let canvas_square_clicked = canvas_coord_to_canvas_square(
                    mouse::position(ctx).x as i16,
                    mouse::position(ctx).y as i16,
                    self.pov,
                );
                // check if this field was not already selected previously
                if self.field != canvas_square_clicked {
                    // no field selected yet -> select field, display legal moves
                    if !self.field_selected {
                        let square = canvas_square_to_board_square(canvas_square_clicked, self.pov);
                        self.current_legal_moves = movegen(
                            &self.game.current_position(),
                            square,
                            self.game.side_to_move(),
                        );
                        self.field = canvas_square_clicked;
                        self.field_selected = true;
                    // a field is already selected -> try to make a move
                    } else if self.field_selected {
                        let start_square = canvas_square_to_board_square(self.field, self.pov);
                        let target_square =
                            canvas_square_to_board_square(canvas_square_clicked, self.pov);

                        // get pieces on start & target square
                        let (is_piece_1, piece1) =
                            match self.game.current_position().piece_on(start_square) {
                                Some(x) => (true, x),
                                None => (false, Piece::Pawn),
                            };
                        let (is_piece_2, _) =
                            match self.game.current_position().piece_on(target_square) {
                                Some(x) => (true, x),
                                None => (false, Piece::Pawn),
                            };
                        // get current board
                        let board = self.game.current_position();

                        // check if the move can be made
                        if is_piece_1
                            && (board.color_on(start_square) != board.color_on(target_square)
                                || !is_piece_2)
                            && Some(self.game.side_to_move()) == board.color_on(start_square)
                            && self
                                .current_legal_moves
                                .iter()
                                .filter(|m| m.get_dest() == target_square)
                                .count()
                                > 0
                        {
                            // TODO: special case for promotions, choose which piece to promote to
                            // if piece is a pawn and it moves to the first or eighth rank, promote to a queen
                            let prom = if piece1 == Piece::Pawn
                                && (target_square.get_rank() == Rank::First
                                    || target_square.get_rank() == Rank::Eighth)
                            {
                                Some(Piece::Queen)
                            } else {
                                None
                            };

                            // make the move
                            self.game
                                .make_move(ChessMove::new(start_square, target_square, prom));
                            // reset field and legal moves
                            self.field_selected = false;
                            self.current_legal_moves = movegen_empty();
                        } else {
                            // if no move is possible -> select new field
                            self.field = canvas_square_clicked;
                            self.current_legal_moves =
                                movegen(&board, target_square, self.game.side_to_move());
                        }
                    }
                }
            } else {
                // if the current player is a bot, let the bot make a move
                self.game.make_move(
                    self.bot_refs[current_player_as_idx].get_move(self.game.current_position()),
                );
            }
            // press the right mouse button to deselect fields
            if mouse::button_pressed(ctx, mouse::MouseButton::Right) {
                self.field_selected = false;
                self.field = (-1, -1);
                self.current_legal_moves = movegen_empty();
            }
        }
        // if game is over, print result
        if !self.game.result().is_none() {
            match self.game.result() {
                Some(chess::GameResult::WhiteCheckmates) => println!("Checkmate! Winner: White"),
                Some(chess::GameResult::BlackCheckmates) => println!("Checkmate! Winner: Black"),
                Some(chess::GameResult::Stalemate) => println!("Stalemate!"),
                Some(chess::GameResult::DrawAccepted) => println!("Draw!"),
                _ => println!("GAME OVER"),
            };
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // since draw gets constantly called, use this to decrease the timeout
        if self.flip_timeout > 0 {
            self.flip_timeout -= 1;
        }
        let tile_size = (WINDWOW_SIZE as u32 / 8) as f32;
        graphics::clear(ctx, [1., 1., 1., 1.0].into());
        let color_to_move = self.game.side_to_move();
        let king_square = board_square_to_canvas_square(
            &self.game.current_position().king_square(color_to_move),
            self.pov,
        );

        // loop over all squares and draw the square
        for x in 0..8 {
            for y in 0..8 {
                // set square color
                let color = if self.field == (x, y) && self.field_selected {
                    // selected
                    Color::from((240, 60, 140, 255))
                } else if self
                    // is a valid target square for a move with the selected piece
                    .current_legal_moves
                    .iter()
                    .filter(|m| {
                        m.get_dest()
                            == canvas_square_to_board_square((x as i16, y as i16), self.pov)
                    })
                    .count()
                    > 0
                {
                    Color::from((200, 80, 80, 255))
                } else if self.game.current_position().checkers().popcnt() > 0
                    && (x as f32, y as f32) == king_square
                {
                    // king square && king is in check
                    Color::from((100, 6, 5, 255))
                } else if (x + y) % 2 == 0 {
                    // dark square
                    Color::from((200, 200, 200, 255))
                } else {
                    // light square
                    Color::from((50, 50, 50, 255))
                };

                let square = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    graphics::Rect::new(
                        x as f32 * tile_size,
                        y as f32 * tile_size,
                        tile_size,
                        tile_size,
                    ),
                    color,
                )?;
                graphics::draw(ctx, &square, (glam::Vec2::new(0.0, 0.0),))?;
            }
        }

        // pieces
        let mut piece_imgs: [[graphics::Image; 6]; 2] = [
            [
                graphics::Image::new(ctx, "/Chess_plt60.png")?,
                graphics::Image::new(ctx, "/Chess_nlt60.png")?,
                graphics::Image::new(ctx, "/Chess_blt60.png")?,
                graphics::Image::new(ctx, "/Chess_rlt60.png")?,
                graphics::Image::new(ctx, "/Chess_qlt60.png")?,
                graphics::Image::new(ctx, "/Chess_klt60.png")?,
            ],
            [
                graphics::Image::new(ctx, "/Chess_pdt60.png")?,
                graphics::Image::new(ctx, "/Chess_ndt60.png")?,
                graphics::Image::new(ctx, "/Chess_bdt60.png")?,
                graphics::Image::new(ctx, "/Chess_rdt60.png")?,
                graphics::Image::new(ctx, "/Chess_qdt60.png")?,
                graphics::Image::new(ctx, "/Chess_kdt60.png")?,
            ],
        ];
        let img_size = piece_imgs[0][0].width() as f32;
        let offset = (tile_size - img_size) / 2.0;
        let board = self.game.current_position();

        // loop over all squares and draw the pieces
        for i in 0..8 {
            for j in 0..8 {
                let square = canvas_square_to_board_square((i as i16, j as i16), self.pov);
                if let Some(piece) = board.piece_on(square) {
                    // if a piece is on the square
                    // get piece color
                    let color = if board.color_on(square) == Some(chess::Color::White) {
                        0
                    } else {
                        1
                    };
                    let dest_point = glam::Vec2::new(
                        i as f32 * tile_size + offset,
                        j as f32 * tile_size + offset,
                    );
                    //let scale = glam::Vec2::new(1.3, 1.3);
                    let param = graphics::Rect::new(
                        i as f32 * tile_size + tile_size * 0.1,
                        j as f32 * tile_size + tile_size * 0.1,
                        tile_size - (tile_size * 0.2),
                        tile_size - (tile_size * 0.2),
                    );
                    // draw the piece
                    graphics::draw(
                        ctx,
                        &piece_imgs[color as usize][piece.to_index() as usize],
                        (dest_point,),
                    )?;
                }
            }
        }

        graphics::present(ctx)?;

        Ok(())
    }
}

// start the game loop
pub fn run(gamestate: GameState) -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("assets");
        path
    } else {
        path::PathBuf::from("./assets")
    };

    let cb = ggez::ContextBuilder::new("Chess", "davhofer")
        .add_resource_path(resource_dir)
        .window_setup(WindowSetup {
            title: "Chess".to_string(),
            icon: "/Chess_rlt60.png".to_string(),
            ..WindowSetup::default()
        })
        .window_mode(WindowMode {
            width: WINDWOW_SIZE,
            height: WINDWOW_SIZE,
            resizable: false,
            ..WindowMode::default()
        });
    let (mut ctx, event_loop) = cb.build()?;

    event::run(ctx, event_loop, gamestate)
}
