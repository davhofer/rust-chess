use chess::{
    self, BitBoard, Board, BoardStatus, ChessMove, File, Game, MoveGen, Piece, Rank, Square, Color
};
use std::cmp;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

pub struct Bot {
    pub color: Color,
    objective: i32,
    depth: u8
}

impl Bot {
    pub fn new(color: Color, depth: u8) -> Bot {
        Bot {
            color,
            objective: if color == Color::White { 1 } else { -1 },
            depth,
        }
    }

    pub fn eval(&self, board: &Board) -> i32 {
        evaluate(board)
    }

    pub fn get_move(&self, board: Board) -> ChessMove {
        // features:
        // negamax + alpha beta
        // iterative deepening
        // transposition tables



        let depth = self.depth;
        println!("Searching for move...");
        let (pos_score, best_move) =
            self.negamax_ab(board, depth, i32::MIN + 2, i32::MAX - 2, self.objective); //self.negamax(board, depth, self.objective);
        println!("Score for current position (white's perspective): {}", self.objective * pos_score);
        if let Some(m) = best_move {
            println!("Move chosen: {:?}", m);
            m
        } else {
            println!("No move possible!");
            ChessMove::new(Square::A1, Square::A2, None)
        }
    }

    fn negamax(&self, board: Board, depth: u8, player_obj: i32) -> (i32, Option<ChessMove>) {
        if depth == 0 || board.status() != BoardStatus::Ongoing {
            return (player_obj * self.eval(&board), None);
        }
        let mut score = i32::MIN;
        let mut best_move = None;
        for m in MoveGen::new_legal(&board) {
            let new_board = board.make_move_new(m);
            let (child_score, child_move) = self.negamax(new_board, depth - 1, -player_obj);
            let child_score = -child_score;
            if child_score > score {
                score = child_score;
                best_move = Some(m);
            }
        }
        (score, best_move)
    }

    fn negamax_ab(
        &self,
        board: Board,
        depth: u8,
        alpha: i32,
        beta: i32,
        player_obj: i32,
    ) -> (i32, Option<ChessMove>) {
        if depth == 0 || board.status() != BoardStatus::Ongoing {
            return (player_obj * self.eval(&board), None);
        }

        let child_nodes = MoveGen::new_legal(&board);
        // order the moves
        let mut score = i32::MIN;
        let mut best_move = None;
        for m in child_nodes {
            let new_board = board.make_move_new(m);
            let (child_score, child_move) =
                self.negamax_ab(new_board, depth - 1, -beta, -alpha, -player_obj);
            let child_score = -child_score;
            if child_score > score {
                score = child_score;
                best_move = Some(m);
            }

            let alpha = cmp::max(alpha, child_score);
            if alpha > beta {
                break;
            }
        }
        (score, best_move)
    }
}

fn evaluate(board: &Board) -> i32 {


    // can learn the parameters with a python neural network?

    // material value + balance & other material considerations, adjusting piece values based on game state
    // piece square tables
    // pawn structure
    // mobility
    // center control
    // king safety
    // special piece patterns (fianchetto, outposts, etc)
    // connectivity
    // protectivity of pieces
    // trapped pieces
    // space
    // tempo
    // danger levels
    // attacking
    // stuff like forks
    // hanging pieces

    // material value
    let pawn = 1000;
    let bishop = 3*pawn;
    let knight = 3*pawn;
    let rook = 5*pawn;
    let queen = 9*pawn;

    let no_pawns_penalty = -pawn/2;
    let bishoppair = pawn/2;
    let knightpair = -pawn/10;
    let rookpair = -pawn/10;

    let white_pieces = board.color_combined(Color::White);
    let black_pieces = board.color_combined(Color::Black);
    let mut mat_white = 0;
    let mut mat_black = 0;

    //     Piece::Pawn
    let pawns = board.pieces(Piece::Pawn);
    let pawns_w = (pawns & white_pieces).popcnt() as i32;
    let pawns_b = (pawns & black_pieces).popcnt() as i32;
    mat_white += pawns_w * pawn + if pawns_w == 0 { no_pawns_penalty } else { 0 };
    mat_black += pawns_b * pawn + if pawns_b == 0 { no_pawns_penalty } else { 0 };
    //     Piece::Knight
    let knights = board.pieces(Piece::Knight);
    let knight_w = (knights & white_pieces).popcnt() as i32;
    let knight_b = (knights & black_pieces).popcnt() as i32;
    mat_white += knight_w * knight + if knight_w == 2 { knightpair } else { 0 };
    mat_black += knight_b * knight + if knight_b == 2 { knightpair } else { 0 };
    //     Piece::Bishop
    let bishops = board.pieces(Piece::Bishop);
    let bishops_w = (bishops & white_pieces).popcnt() as i32;
    let bishops_b = (bishops & black_pieces).popcnt() as i32;
    // bonus for bishoppair
    mat_white += bishops_w * bishop + if bishops_w == 2 { bishoppair } else { 0 };
    mat_black += bishops_b * bishop + if bishops_b == 2 { bishoppair } else { 0 };
    //     Piece::Rook
    let rooks = board.pieces(Piece::Rook);
    let rooks_w = (rooks & white_pieces).popcnt() as i32;
    let rooks_b = (rooks & black_pieces).popcnt() as i32;
    // malus for rook pair (redundancy)
    mat_white += rooks_w * rook + if rooks_w == 2 { rookpair } else { 0 };
    mat_black += rooks_b * rook + if rooks_b == 2 { rookpair } else { 0 };
    //
    //     Piece::Queen
    let queens = board.pieces(Piece::Queen);
    mat_white += (queens & white_pieces).popcnt() as i32 * queen;
    mat_black += (queens & black_pieces).popcnt() as i32 * queen;


    let mat_score = mat_white - mat_black;

    // possible moves
    let num_moves_current_player = MoveGen::new_legal(board).len();

    mat_white - mat_black
}
