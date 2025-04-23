use ggez::{ContextBuilder, GameResult};
use ggez::event;
use ggez::conf::{WindowSetup, WindowMode};

mod miner;
mod game_state;
mod ui;
mod pet;

use game_state::MainState;

const WINDOW_WIDTH: f32 = 1920.0;
const WINDOW_HEIGHT: f32 = 1080.0;


// Main function to run the game and initialize the state
fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("placeholder_title", "Daniel Zheng")
        .window_setup(WindowSetup::default().title("Placeholder Title"))
        .window_mode(WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()?;
    
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}