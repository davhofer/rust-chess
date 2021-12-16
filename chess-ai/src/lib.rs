use chess::{
    self, BitBoard, Board, BoardStatus, ChessMove, Color, File, Game, MoveGen, Piece, Rank, Square,
    EMPTY,
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

const INFINITY: i32 = i32::MAX - 2;

const _MG_PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25, -20, -14,
    13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4, -10, 3, 3, 33, -12,
    -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0,
];

const _EG_PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 178, 173, 158, 134, 147, 132, 165, 187, 94, 100, 85, 67, 56, 53, 82,
    84, 32, 24, 13, 5, -2, 4, 17, 17, 13, 9, -3, -7, -7, -8, 3, -1, 4, 7, -6, 1, 0, -5, -1, -8, 13,
    8, 8, 10, 13, 0, 2, -7, 0, 0, 0, 0, 0, 0, 0, 0,
];

const _MG_KNIGHT_TABLE: [i32; 64] = [
    -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65, 84,
    129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9, 12, 10,
    19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17, -28, -19, -23,
];

const _EG_KNIGHT_TABLE: [i32; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99, -25, -8, -25, -2, -9, -25, -24, -52, -24, -20, 10, 9,
    -1, -9, -19, -41, -17, 3, 22, 22, 22, 11, 8, -18, -18, -6, 16, 25, 16, 17, 4, -18, -23, -3, -1,
    15, 10, -3, -20, -22, -42, -20, -10, -5, -2, -20, -23, -44, -29, -51, -23, -15, -22, -18, -50,
    -64,
];

const _MG_BISHOP_TABLE: [i32; 64] = [
    -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35, 50,
    37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14, 27, 18,
    10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
];

const _EG_BISHOP_TABLE: [i32; 64] = [
    -14, -21, -11, -8, -7, -9, -17, -24, -8, -4, 7, -12, -3, -13, -4, -14, 2, -8, 0, -1, -2, 6, 0,
    4, -3, 9, 12, 9, 14, 10, 3, 2, -6, 3, 13, 19, 7, 10, -3, -9, -12, -3, 8, 10, 13, 3, -7, -15,
    -14, -18, -7, -1, 4, -9, -15, -27, -23, -9, -23, -5, -9, -16, -5, -17,
];

const _MG_ROOK_TABLE: [i32; 64] = [
    32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45, 61, 16,
    -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25, -16, -17, 3, 0,
    -5, -33, -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7, -37, -26,
];

const _EG_ROOK_TABLE: [i32; 64] = [
    13, 10, 18, 15, 12, 12, 8, 5, 11, 13, 13, 11, -3, 3, 8, 3, 7, 7, 7, 5, 4, -3, -5, -3, 4, 3, 13,
    1, 2, 1, -1, 2, 3, 5, 8, 4, -5, -6, -8, -11, -4, 0, -5, -1, -7, -12, -8, -16, -6, -6, 0, 2, -9,
    -9, -11, -3, -9, 2, 3, -1, -5, -13, 4, -20,
];

const _MG_QUEEN_TABLE: [i32; 64] = [
    -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56, 47,
    57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11, -2, -5, 2,
    14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50,
];

const _EG_QUEEN_TABLE: [i32; 64] = [
    -9, 22, 22, 27, 27, 19, 10, 20, -17, 20, 32, 41, 58, 25, 30, 0, -20, 6, 9, 49, 47, 35, 19, 9,
    3, 22, 24, 45, 57, 40, 57, 36, -18, 28, 19, 47, 31, 34, 39, 23, -16, -27, 15, 6, 9, 17, 10, 5,
    -22, -23, -30, -16, -16, -23, -36, -32, -33, -28, -22, -43, -5, -32, -20, -41,
];

const _MG_KING_TABLE: [i32; 64] = [
    -65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16, -20, 6,
    22, -22, -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44, -33, -51, -14,
    -14, -22, -46, -44, -30, -15, -27, 1, 7, -8, -64, -43, -16, 9, 8, -15, 36, 12, -54, 8, -28, 24,
    14,
];

const _EG_KING_TABLE: [i32; 64] = [
    -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15, 20, 45,
    44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19, -3, 11, 21, 23,
    16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28, -14, -24, -43,
];

pub const PST: [[[i32; 64]; 2]; 6] = [
    [_MG_PAWN_TABLE, _EG_PAWN_TABLE],
    [_MG_KNIGHT_TABLE, _EG_KNIGHT_TABLE],
    [_MG_BISHOP_TABLE, _EG_BISHOP_TABLE],
    [_MG_ROOK_TABLE, _EG_ROOK_TABLE],
    [_MG_QUEEN_TABLE, _EG_QUEEN_TABLE],
    [_MG_KING_TABLE, _EG_KING_TABLE],
];

pub struct Bot {
    pub color: Color,
    objective: i32,
    depth: u8,
    _debug: bool,
}

impl Bot {
    pub fn new(color: Color, depth: u8, _debug: bool) -> Bot {
        Bot {
            color,
            objective: if color == Color::White { 1 } else { -1 },
            depth,
            _debug,
        }
    }

    pub fn eval(&self, board: &Board) -> i32 {
        evaluate(board)
    }

    pub fn get_move(&self, board: Board) -> ChessMove {
        // features:
        // negamax + alpha beta
        // TODO:
        // iterative deepening
        // transposition tables

        println!("Searching for move...");
        let (pos_score, best_move, positions) =
            self.negamax(&board, self.depth, -INFINITY, INFINITY, self.objective);

        // some output
        println!();
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

    

    pub fn negamax(
        &self,
        board: &Board,
        depth: u8,
        alpha: i32,
        beta: i32,
        player_obj: i32,
    ) -> (i32, Option<ChessMove>, u32) {
        if depth == 0 || board.status() != BoardStatus::Ongoing {
            // instead of returning score, start quiscence search (same search function, but only look at capture moves and keep going until no captures are left)
            return (player_obj * self.eval(&board), None, 1);
        }
        let mut alpha = alpha;

        let mut child_nodes = MoveGen::new_legal(&board);
        let mut best_score = i32::MIN;
        let mut best_move = None;

        let mut count = 0;

        // TODO: better move ordering
        // use movegen.filter... ?
        // create a sorted vector?

        // first, only iterate capture moves
        let targets = board.color_combined(!board.side_to_move());
        child_nodes.set_iterator_mask(*targets);

        for m in &mut child_nodes {
            let (child_score, child_move, c) = self.negamax(
                &board.make_move_new(m),
                depth - 1,
                -beta,
                -alpha,
                -player_obj,
            );
            count += c;
            // if a move leads to checkmate, prefer the shortest sequence
            let child_score = if child_score >= INFINITY - 1 - self.depth as i32 {
                -(child_score - 1)
            } else {
                -child_score
            };
            if child_score > best_score {
                best_score = child_score;
                best_move = Some(m);
            }

            alpha = cmp::max(alpha, child_score);
            if alpha >= beta {
                break;
            }
        }

        // all the other moves
        child_nodes.set_iterator_mask(!EMPTY);
        for m in &mut child_nodes {
            let (child_score, child_move, c) = self.negamax(
                &board.make_move_new(m),
                depth - 1,
                -beta,
                -alpha,
                -player_obj,
            );
            count += c;
            // if a move leads to checkmate, prefer the shortest sequence
            let child_score = if child_score >= INFINITY - 1 - self.depth as i32 {
                -(child_score - 1)
            } else {
                -child_score
            };
            if child_score > best_score {
                best_score = child_score;
                best_move = Some(m);
            }

            alpha = cmp::max(alpha, child_score);
            if alpha >= beta {
                break;
            }
        }
        (best_score, best_move, count)
    }
}

fn negamax_no_moveorder(
    &self,
    board: Board,
    depth: u8,
    alpha: i32,
    beta: i32,
    player_obj: i32,
) -> (i32, Option<ChessMove>, u32) {
    if depth == 0 || board.status() != BoardStatus::Ongoing {
        // instead of returning score, start quiscence search (same search function, but only look at capture moves and keep going until no captures are left)
        return (player_obj * self.eval(&board), None, 1);
    }
    let mut alpha = alpha;

    let mut child_nodes = MoveGen::new_legal(&board);
    let mut score = i32::MIN;
    let mut best_move = None;
    let mut count = 0;

    for m in &mut child_nodes {
        let (child_score, child_move, c) = self.negamax_no_moveorder(
            board.make_move_new(m),
            depth - 1,
            -beta,
            -alpha,
            -player_obj,
        );
        count += c;
        // if a move leads to checkmate, prefer the shortest sequence
        let child_score = if child_score >= INFINITY - 1 - self.depth as i32 {
            -(child_score - 1)
        } else {
            -child_score
        };
        if child_score > score {
            score = child_score;
            best_move = Some(m);
        }
        alpha = cmp::max(alpha, child_score);
        if alpha >= beta {
            break;
        }
    }

    (score, best_move, count)
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
    
    if board.status() == BoardStatus::Stalemate {
        return 0;
    } else if board.status() == BoardStatus::Checkmate {
        if board.side_to_move() == Color::Black {
            return INFINITY;
        } else {
            return -INFINITY;
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

    let mut gamephase = 0;
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
    gamephase += knights_b + knights_w + bishops_w + bishops_b;
    //     Piece::Rook
    let rooks_board = board.pieces(Piece::Rook);
    let rooks_w = (rooks_board & white_pieces).popcnt() as i32;
    let rooks_b = (rooks_board & black_pieces).popcnt() as i32;
    gamephase += (rooks_w + rooks_b) * 2;
    //     Piece::Queen
    let queens_board = board.pieces(Piece::Queen);
    let queens_w = (queens_board & white_pieces).popcnt() as i32;
    let queens_b = (queens_board & black_pieces).popcnt() as i32;
    gamephase += (queens_w + queens_b) * 4;

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

    // draw
    if mat_white == 0 && mat_black == 0 {
        return 0;
    }

    // is_endgame ?
    // might be either if:
    // Both sides have no queens or
    // Every side which has a queen has additionally no other pieces or one minorpiece maximum.
    let is_endgame = if (queens_w == 0 || queens_w + rooks_w + bishops_w + knights_w <= 2)
        && (queens_b == 0 || queens_b + rooks_b + bishops_b + knights_b <= 2)
    {
        1
    } else {
        0
    };

    let mut eg_score = 0;
    let mut mg_score = 0;
    for i in 0..64 {
        let sq = unsafe { Square::new(i as u8) };
        match board.piece_on(sq) {
            None => continue,
            Some(p) => {
                let p_idx = match p {
                    Piece::Pawn => 0,
                    Piece::Knight => 1,
                    Piece::Bishop => 2,
                    Piece::Rook => 3,
                    Piece::Queen => 4,
                    Piece::King => 5,
                };
                let (sign, idx) = if board.color_on(sq) == Some(Color::White) {
                    (1, i)
                } else {
                    let is = i as i16;
                    let mirrored = (is + 8 * (7 - 2 * (is / 8))) as usize;
                    (-1, mirrored)
                };
                mg_score += sign * PST[p_idx][0][idx];
                eg_score += sign * PST[p_idx][1][idx];
            }
        }
    }
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

    // /* tapered eval */
    // int mgScore = mg[side2move] - mg[OTHER(side2move)];
    // int egScore = eg[side2move] - eg[OTHER(side2move)];
    // int mgPhase = gamePhase;
    // if (mgPhase > 24) mgPhase = 24; /* in case of early promotion */
    // int egPhase = 24 - mgPhase;
    // return (mgScore * mgPhase + egScore * egPhase) / 24;
    let mg_phase = if gamephase > 24 { 24 } else { gamephase };
    let eg_phase = 24 - mg_phase;

    let pst_adjusted = (mg_score * mg_phase + eg_score * eg_phase) / 24;

    let mat_score = mat_white - mat_black + pst_adjusted;

    // if a player has little material, it's beneficial for his opponent to push him to the corner/edge of the board to deliver checkmate
    // your_score += opponent_king_dist_to_corner * opponent_endgame_factor
    let king_w_idx = board.king_square(Color::White).to_int() as i32;
    let king_b_idx = board.king_square(Color::Black).to_int() as i32;

    let (king_corner_score_w, king_corner_score_b) = force_king_to_corner(king_w_idx, king_b_idx);

    let endgame_force_king = king_corner_score_w * endgame_factor_b * 2 / pawn
        - king_corner_score_b * endgame_factor_w * 2 / pawn;
    // possible moves
    let movegen = MoveGen::new_legal(board);

    let num_moves_current_player = movegen.len();
    let mobility = 0;

    mat_score + mobility + endgame_force_king
}

fn eval_from_fen(fen: String) -> i32 {
    let b = Board::from_fen(fen).expect("Valid FEN");
    let e = evaluate(&b);
    println!("{}", e);
    e
}

fn eval_piecescore_simple(board: &Board) -> i32 {
    let pawn = 10;
    let bishop = 30;
    let knight = 30;
    let rook = 50;
    let queen = 90;

    let no_pawns_penalty = -5;
    let bishoppair = 5;
    let knightpair = -1;
    let rookpair = -1;
    let white_pieces = board.color_combined(chess::Color::White);
    let black_pieces = board.color_combined(chess::Color::Black);
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

    mat_white - mat_black
}
