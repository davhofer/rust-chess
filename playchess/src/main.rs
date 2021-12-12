use chess_gamesetup as setup;
fn main() {
    let mut chessgame = setup::command_line_setup();
    chessgame.start();
}
