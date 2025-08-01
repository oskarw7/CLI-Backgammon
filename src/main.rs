use game::Game;

mod game;
mod utils;

fn main() {
    let game = Game::new();
    game.run();
}
