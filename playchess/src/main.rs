use chess_gamesetup as setup;
use ggez::GameResult;

fn main() -> GameResult {
    let (p1, p2, game, visual) = setup::command_line_setup();
    setup::start_game(p1, p2, game, visual)
}
