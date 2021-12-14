use chess::{
    self, BitBoard, Board, BoardStatus, ChessMove, Color, File, Game, MoveGen, Piece, Rank, Square,
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

const INFTY: i32 = i32::MAX - 2;

pub struct Bot {
    pub color: Color,
    objective: i32,
    depth: u8,
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
        let (pos_score, best_move) = self.negamax_ab(board, depth, -INFTY, INFTY, self.objective); //self.negamax(board, depth, self.objective);
        println!(
            "Score for current position (white's perspective): {}",
            self.objective * pos_score
        );
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
            // instead of returning score, start quiscence search (same search function, but only look at capture moves and keep going until no captures are left)
            return (player_obj * self.eval(&board), None);
        }

        let child_nodes = MoveGen::new_legal(&board);
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

fn force_king_to_corner(king_w_idx: i32, king_b_idx: i32) -> (i32, i32) {
    let king_w_x = king_w_idx % 8;
    let king_w_y = king_w_idx / 8;
    let king_b_x = king_b_idx % 8;
    let king_b_y = king_b_idx / 8;
    let center_distance_b_x = cmp::max(3 - king_b_x, king_b_x - 4);
    let center_distance_b_y = cmp::max(3 - king_b_y, king_b_y - 4);
    let center_distance_b = center_distance_b_x + center_distance_b_y;
    let center_distance_w_x = cmp::max(3 - king_w_x, king_w_x - 4);
    let center_distance_w_y = cmp::max(3 - king_w_y, king_w_y - 4);
    let center_distance_w = center_distance_w_x + center_distance_w_y;
    let kings_distance = (king_w_x - king_b_x).abs() + (king_w_y - king_b_y).abs();
    (center_distance_b, center_distance_w)
}

fn evaluate(board: &Board) -> i32 {
    let movegen = MoveGen::new_legal(board);
    if board.status() == BoardStatus::Stalemate {
        return 0;
    } else if board.status() == BoardStatus::Checkmate {
        if board.side_to_move() == Color::Black {
            return INFTY;
        } else {
            return -INFTY;
        }
    }
    // different evaluation based on board state
    // specifically endgame or heuristics when no pieces can be captures (bring own pieces closer to enemy king)
    // sebastian lague ForceKingToCornerEndgameEval

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
    let pawn = 100;
    let bishop = 3 * pawn;
    let knight = 3 * pawn;
    let rook = 5 * pawn;
    let queen = 9 * pawn;

    let white_pieces = board.color_combined(Color::White);
    let black_pieces = board.color_combined(Color::Black);
    //     Piece::Pawn
    let pawns_board = board.pieces(Piece::Pawn);
    let pawns_w = (pawns_board & white_pieces).popcnt() as i32;
    let pawns_b = (pawns_board & black_pieces).popcnt() as i32;
    //     Piece::Knight
    let knights_board = board.pieces(Piece::Knight);
    let knights_w = (knights_board & white_pieces).popcnt() as i32;
    let knights_b = (knights_board & black_pieces).popcnt() as i32;
    //     Piece::Bishop
    let bishops_board = board.pieces(Piece::Bishop);
    let bishops_w = (bishops_board & white_pieces).popcnt() as i32;
    let bishops_b = (bishops_board & black_pieces).popcnt() as i32;
    //     Piece::Rook
    let rooks_board = board.pieces(Piece::Rook);
    let rooks_w = (rooks_board & white_pieces).popcnt() as i32;
    let rooks_b = (rooks_board & black_pieces).popcnt() as i32;
    //     Piece::Queen
    let queens_board = board.pieces(Piece::Queen);
    let queens_w = (queens_board & white_pieces).popcnt() as i32;
    let queens_b = (queens_board & black_pieces).popcnt() as i32;

    let mut mat_white = pawns_w * pawn
        + knights_w * knight
        + bishops_w * bishop
        + rooks_w * rook
        + queens_w * queen;
    let mut mat_black = pawns_b * pawn
        + knights_b * knight
        + bishops_b * bishop
        + rooks_b * rook
        + queens_b * queen;

    let total_piece_val = 8 * pawn + 2 * (rook + bishop + knight) + queen;
    // bonuses
    let no_pawns_penalty = -pawn / 2;
    let bishoppair = pawn / 2;
    let knightpair = -pawn / 10;
    let rookpair = -pawn / 10;
    let endgame_factor_w = total_piece_val - mat_white;
    let endgame_factor_b = total_piece_val - mat_black;

    mat_white += if pawns_w == 0 { no_pawns_penalty } else { 0 };
    mat_white += if knights_w == 2 { knightpair } else { 0 };
    mat_white += if bishops_w == 2 { bishoppair } else { 0 };
    mat_white += if rooks_w == 2 { rookpair } else { 0 };

    mat_black += if pawns_b == 0 { no_pawns_penalty } else { 0 };
    mat_black += if knights_b == 2 { knightpair } else { 0 };
    mat_black += if bishops_b == 2 { bishoppair } else { 0 };
    mat_black += if rooks_b == 2 { rookpair } else { 0 };
    // endgame. TODO: find out how to do this better
    // pawns increase in value the longer the game goes (the less material the player has)
    mat_white += (pawns_w * endgame_factor_w * pawn) / total_piece_val;
    mat_black += (pawns_b * endgame_factor_b * pawn) / total_piece_val;

    let mat_score = mat_white - mat_black;

    // if a player has little material, it's beneficial for his opponent to push him to the corner/edge of the board to deliver checkmate
    // your_score += opponent_king_dist_to_corner * opponent_endgame_factor
    let king_w_idx = board.king_square(Color::White).to_int() as i32;
    let king_b_idx = board.king_square(Color::Black).to_int() as i32;

    let (king_corner_score_w, king_corner_score_b) = force_king_to_corner(king_w_idx, king_b_idx);

    let endgame_force_king = king_corner_score_w * endgame_factor_b * 2 / pawn
        - king_corner_score_b * endgame_factor_w * 2 / pawn;
    // possible moves
    let num_moves_current_player = movegen.len();
    let mobility = 0;

    mat_score + mobility // + endgame_force_king
}

// fn eval_from_fen(fen: String) -> i32 {
//     let b = Board::from_fen(fen).expect("Valid FEN");
//     let e = evaluate(&b);
//     println!("{}", e);
//     e
// }
