#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

use chess::{self, Board,Square, Piece, Color, Game, ChessMove};
use chess_ai::Bot;
use std::io;

const SQUARES : [[Square;8];8] = [
    [Square::A8,Square::B8,Square::C8,Square::D8,Square::E8,Square::F8,Square::G8,Square::H8],
    [Square::A7,Square::B7,Square::C7,Square::D7,Square::E7,Square::F7,Square::G7,Square::H7],
    [Square::A6,Square::B6,Square::C6,Square::D6,Square::E6,Square::F6,Square::G6,Square::H6],
    [Square::A5,Square::B5,Square::C5,Square::D5,Square::E5,Square::F5,Square::G5,Square::H5],
    [Square::A4,Square::B4,Square::C4,Square::D4,Square::E4,Square::F4,Square::G4,Square::H4],
    [Square::A3,Square::B3,Square::C3,Square::D3,Square::E3,Square::F3,Square::G3,Square::H3],
    [Square::A2,Square::B2,Square::C2,Square::D2,Square::E2,Square::F2,Square::G2,Square::H2],
    [Square::A1,Square::B1,Square::C1,Square::D1,Square::E1,Square::F1,Square::G1,Square::H1],
];

fn print_board(board: &Board) {
    let mut rank = 8;
    println!("  -------------------------");
    for row in SQUARES {
        let r = rank.to_string();
        let mut s = String::from(r);
        s.push_str(" |");
        rank -= 1;
        for square in row {
            let mut p = match board.piece_on(square) {
                None => "  ",
                Some(piece) => match piece {
                    Piece::Pawn => "p ",
                    Piece::Knight => "n " ,
                    Piece::Bishop => "b ",
                    Piece::Rook => "r ",
                    Piece::Queen => "q ",
                    Piece::King => "k ",
                }
            };
            if board.color_on(square) == Some(Color::White) {
                let p = &p.to_uppercase();
                s.push_str(p);
            } else {
                s.push_str(p);
            }
            
            s.push_str("|");
        }
        println!("{}",s);
        println!("  -------------------------");

    }
    println!("   a  b  c  d  e  f  g  h");

}

#[derive(PartialEq)]
enum PlayerType {
    Human,
    Bot
}

#[derive(PartialEq)]
enum GameVisual {
    CommandLine,
    Gui
}

fn get_move_stdin(board: Board) -> ChessMove {
    println!("Enter the next move (in SAN): ");
    let mut stdin = io::stdin();
    let mut m = Ok(ChessMove::new(Square::A1, Square::A2, None));
    loop {
        let mut buffer = String::new();
        stdin.read_line(&mut buffer);
        buffer.pop();
        m = ChessMove::from_san(&board, &buffer);
        if let Ok(_) = m {
            break;
        } else {
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
    }  
    m.expect("Please enter a valid move in SAN format!")
}

struct Player {
    player_type: PlayerType,
    color: Color,
    bot_ref: Bot
}



impl Player {
    fn new_human(color: Color) -> Player {
        Player {
            player_type: PlayerType::Human,
            color: color,
            bot_ref: Bot::new(color, 0),
        }
    }

    fn new_bot(color: Color, depth: u8) -> Player {
        Player {
            player_type: PlayerType::Bot,
            color: color,
            bot_ref: Bot::new(color, depth),
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
    Player::new_bot(color,3)
}

pub struct ChessGame {
    player1: Player,
    player2: Player,
    game: Game,
    visual: GameVisual
}

impl ChessGame {
    pub fn start(& mut self) {
        if self.visual == GameVisual::CommandLine {
            while self.game.result().is_none() {
                print_board(&self.game.current_position());
                if self.game.side_to_move() == Color::White {
                    self.game.make_move(self.player1.get_move(self.game.current_position()));
                } else {
                    self.game.make_move(self.player2.get_move(self.game.current_position()));
                }
            }
            print_board(&self.game.current_position());
            println!("GAME OVER");
        } else {
            println!("Start gui!")
        }
        
    }
}

pub fn command_line_setup() -> ChessGame {
    let mut buffer1 = String::new();
    let mut buffer2 = String::new();
    let mut stdin = io::stdin();

    println!("Select player 1: human or bot.");
    let mut player1 = Player::new_human(Color::White);
    let mut player2 = Player::new_human(Color::Black);

    stdin.read_line(&mut buffer1);
    buffer1.pop();
    if buffer1 == "human" {
        player1 = Player::new_human(Color::White);
    } else if buffer1 == "bot" {
        player1 = bot_setup(Color::White);
    } else {
        println!("Please select either human or bot!")
        // TODO: handle the case of bad user input
    }
    println!();
    println!("Select player 2: human or bot.");

    stdin.read_line(&mut buffer2);
    buffer2.pop();
    if buffer2 == "human" {
        player2 = Player::new_human(Color::Black);
    } else if buffer2 == "bot" {
        player2 = bot_setup(Color::Black);
    } else {
        println!("Please select either human or bot!")
    }

    println!("Do you want to play from the default starting position or a specific FEN?");
    let mut buf = String::new();
    stdin.read_line(&mut buf);
    buf.pop();
    let mut game = Game::new();
    if buf != "default" {
        println!("Enter FEN:");
        let mut fen = String::new();
        stdin.read_line(&mut fen);
        fen.pop();
        game = Game::new_with_board(Board::from_fen(fen).expect("Valid FEN"));
    } 
    println!("Do yo want to play in the commandline or gui?");

    let mut buf = String::new();
    stdin.read_line(&mut buf);
    buf.pop();
    if buf == "commandline" {
        ChessGame {
            player1,
            player2,
            game,
            visual: GameVisual::CommandLine,
        }
    } else if buf == "gui" {
        ChessGame {
            player1,
            player2,
            game,
            visual: GameVisual::Gui,
            
        }
    } else {
        println!("Please enter either commandline or gui.");
        ChessGame {
            player1: Player::new_human(Color::White),
            player2: Player::new_human(Color::Black),
            game: Game::new(),
            visual: GameVisual::Gui,
        }
    }
 
}