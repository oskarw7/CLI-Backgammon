use game::Game;

mod game;
mod utils;

fn main() {
    let mut game = Game::new();
    game.run();
}
