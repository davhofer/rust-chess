#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

// imports
use chess::{self, Board, ChessMove, Color, Game, Piece, Square};
use chess_ai::Bot;
use chess_gui::{self, GameState};
use ggez::GameResult;
use std::io;
use std::str::FromStr;

// chessboard squares
const SQUARES: [[Square; 8]; 8] = [
    [
        Square::A8,
        Square::B8,
        Square::C8,
        Square::D8,
        Square::E8,
        Square::F8,
        Square::G8,
        Square::H8,
    ],
    [
        Square::A7,
        Square::B7,
        Square::C7,
        Square::D7,
        Square::E7,
        Square::F7,
        Square::G7,
        Square::H7,
    ],
    [
        Square::A6,
        Square::B6,
        Square::C6,
        Square::D6,
        Square::E6,
        Square::F6,
        Square::G6,
        Square::H6,
    ],
    [
        Square::A5,
        Square::B5,
        Square::C5,
        Square::D5,
        Square::E5,
        Square::F5,
        Square::G5,
        Square::H5,
    ],
    [
        Square::A4,
        Square::B4,
        Square::C4,
        Square::D4,
        Square::E4,
        Square::F4,
        Square::G4,
        Square::H4,
    ],
    [
        Square::A3,
        Square::B3,
        Square::C3,
        Square::D3,
        Square::E3,
        Square::F3,
        Square::G3,
        Square::H3,
    ],
    [
        Square::A2,
        Square::B2,
        Square::C2,
        Square::D2,
        Square::E2,
        Square::F2,
        Square::G2,
        Square::H2,
    ],
    [
        Square::A1,
        Square::B1,
        Square::C1,
        Square::D1,
        Square::E1,
        Square::F1,
        Square::G1,
        Square::H1,
    ],
];

// display board in commandline
fn print_board(board: &Board) {
    let mut rank = 8;
    println!("  -------------------------");
    for row in SQUARES {
        let mut s = String::from(rank.to_string());
        rank -= 1;
        s.push_str(" |");
        for square in row {
            let p = match board.piece_on(square) {
                None => "  ",
                Some(piece) => match piece {
                    Piece::Pawn => "p ",
                    Piece::Knight => "n ",
                    Piece::Bishop => "b ",
                    Piece::Rook => "r ",
                    Piece::Queen => "q ",
                    Piece::King => "k ",
                },
            };
            if board.color_on(square) == Some(Color::White) {
                let p = &p.to_uppercase();
                s.push_str(p);
            } else {
                s.push_str(p);
            }
            s.push_str("|");
        }
        println!("{}", s);
        println!("  -------------------------");
    }
    println!("   a  b  c  d  e  f  g  h");
}

#[derive(PartialEq)]
pub enum PlayerType {
    Human,
    Bot,
}

#[derive(PartialEq)]
pub enum GameVisual {
    CommandLine,
    Gui,
}

// get a string from stdin
fn stdin_get_input() -> String {
    let stdin = io::stdin();
    let mut s = String::new();
    let _ = stdin.read_line(&mut s);
    trim_newline(&mut s);
    s
}

fn print_san_help() {
    println!("-------------------------------------------------------");
    println!("Please enter a valid move in SAN format.");
    println!("Capture: exd5 or Nxc6");
    println!("If the move results in a check: add + at the end");
    println!("En passant: add (ep) at the end");
    println!("Promotion: add =Q at the end, replace Q with the \npiece you want to promote to");
    println!("To disambiguate between two pieces, e.g. both Rooks \ncould take on c1: Raxc1 to specify the rook on the a file");
    println!("Checkmate: add ++ at the end");
    println!("Castle kingside / queenside: O-O / O-O-O");
    println!("-------------------------------------------------------");
}

// get a move from the player through stdin
fn get_move_stdin(board: Board) -> ChessMove {
    println!("Enter the next move (in SAN): ");
    let mut _move;
    loop {
        let input = stdin_get_input();
        _move = ChessMove::from_san(&board, &input);
        match _move {
            Ok(_) => break,
            Err(_) => print_san_help(),
        }
    }
    _move.expect("Please enter a valid move in SAN format!")
}

pub struct Player {
    player_type: PlayerType,
    color: Color,
    bot_ref: Bot,
}

impl Player {
    fn new_human(color: Color) -> Player {
        Player {
            player_type: PlayerType::Human,
            color: color,
            bot_ref: Bot::new(color, 0, false),
        }
    }

    fn new_bot(color: Color, depth: u8, debug: bool) -> Player {
        Player {
            player_type: PlayerType::Bot,
            color: color,
            bot_ref: Bot::new(color, depth, debug),
        }
    }

    fn get_move(&self, board: Board) -> ChessMove {
        if self.player_type == PlayerType::Human {
            get_move_stdin(board)
        } else {
            self.bot_ref.get_move(board)
        }
    }
}

fn bot_setup(color: Color) -> Player {
    // TODO: allow commandline configuration of bot
    println!("--- BOT setup ---");
    println!("Search depth: ");
    let depth: u8 = if let Ok(d) = stdin_get_input().parse() {
        d
    } else {
        3
    };

    println!("Debug? y/n ");
    let mut debug = false;
    if stdin_get_input(&stdin) == "y" {
        debug = true;
    }

    println!("-----------------");
    Player::new_bot(color, depth, debug)
}

// configure a player
fn stdin_get_player(color: Color) -> std::result::Result<Player, ()> {
    match stdin_get_input().as_str() {
        "human" => Ok(Player::new_human(color)),
        "bot" => Ok(bot_setup(color)),
        _ => Err(()),
    }
}

// remove trailing newline from string
fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

// game setup through commandline
pub fn command_line_setup() -> (Player, Player, Game, GameVisual) {
    // player 1
    println!("Select player 1: human or bot.");

    let player1 = match stdin_get_player(Color::White) {
        Ok(player) => player,
        Err(_) => {
            println!("Invalid input should be 'human' or 'bot'.");
            std::process::exit(1);
        }
    };

    // player 2
    println!("Select player 2: human or bot.");

    let player2 = match stdin_get_player(Color::Black) {
        Ok(player) => player,
        Err(_) => {
            println!("Invalid input should be 'human' or 'bot'.");
            std::process::exit(1);
        }
    };

    // board position
    println!("Do you want to play from the default starting position or a specific FEN?");

    let game = match stdin_get_input().as_str() {
        "default" => Game::new(),
        _ => {
            println!("Enter FEN:");
            let fen = stdin_get_input();
            let board = Board::from_str(&fen).expect("Valid FEN");
            Game::new_with_board(board)
        }
    };

    // visualization
    println!("Do yo want to play in the commandline or gui?");

    match stdin_get_input().as_str() {
        "commandline" => (player1, player2, game, GameVisual::CommandLine),
        "gui" => (player1, player2, game, GameVisual::Gui),
        _ => {
            println!("Invalid input should be 'commandline' or 'gui'.");
            std::process::exit(1);
        }
    }
}

// start the configured game
pub fn start_game(
    player1: Player,
    player2: Player,
    mut game: Game,
    visual: GameVisual,
) -> GameResult {
    if visual == GameVisual::CommandLine {
        // game loop in commandline
        while game.result().is_none() {
            print_board(&game.current_position());
            if game.side_to_move() == Color::White {
                game.make_move(player1.get_move(game.current_position()));
            } else {
                game.make_move(player2.get_move(game.current_position()));
            }
        }
        print_board(&game.current_position());
        match game.result() {
            Some(chess::GameResult::WhiteCheckmates) => println!("Checkmate! Winner: White"),
            Some(chess::GameResult::BlackCheckmates) => println!("Checkmate! Winner: Black"),
            Some(chess::GameResult::Stalemate) => println!("Stalemate!"),
            Some(chess::GameResult::DrawAccepted) => println!("Draw!"),
            _ => println!("GAME OVER"),
        };
        Ok(())
    } else {
        // setup for gui gamestate
        let (playable1, bot_ref1) = if player1.player_type == PlayerType::Human {
            (true, Bot::new(Color::White, 0, false))
        } else {
            (false, player1.bot_ref)
        };
        let (playable2, bot_ref2) = if player2.player_type == PlayerType::Human {
            (true, Bot::new(Color::White, 0, false))
        } else {
            (false, player2.bot_ref)
        };

        let gui_gamestate = GameState::new(game, [playable1, playable2], [bot_ref1, bot_ref2]);
        println!("Starting gui...");
        // run gui gameloop
        chess_gui::run(gui_gamestate)
    }
}
