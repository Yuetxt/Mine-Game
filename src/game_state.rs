use ggez::graphics::Rect;
use ggez::{Context, GameResult};
use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::input::mouse::MouseButton;
use rand::Rng;
use std::time::{Duration, Instant};

use crate::miner::{Miner, MinerType};
use crate::ui;

// Game constants
pub const MAX_ROUNDS: usize = 15;
pub const ROUND_DURATION: Duration = Duration::from_secs(60); // 1 minute
pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

pub enum GameState {
    Playing,
    RoundEnd,
    GameOver,
}

pub struct MainState {
    pub player: Miner,
    pub bots: Vec<Miner>,
    pub current_round: usize,
    pub round_start_time: Instant,
    pub game_state: GameState,
    pub round_results: Option<Vec<(usize, f32)>>, // (miner_index, donated_gold)
    pub past_results: Vec<bool>, // true for win, false for loss
}

impl MainState {
    pub fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let player = Miner::new(MinerType::Player);
        let mut bots = Vec::new();
        
        // Create 3 bot miners
        for _ in 0..3 {
            bots.push(Miner::new(MinerType::Bot));
        }
    
        Ok(MainState {
            player,
            bots,
            current_round: 1,
            round_start_time: Instant::now(),
            game_state: GameState::Playing,
            round_results: None,
            past_results: Vec::new(),
        })
    }
    

    pub fn bot_make_decision(&mut self, bot_index: usize) {
        let bot = &mut self.bots[bot_index];
        if !bot.alive {
            return;
        }

        // Calculate time left in the round to determine "end of round" behavior
        let now = std::time::Instant::now();
        let round_elapsed = now.duration_since(self.round_start_time);
        let round_progress = round_elapsed.as_secs_f32() / ROUND_DURATION.as_secs_f32();
        let is_end_of_round = round_progress >= 0.8; // Last 20% of the round
        
        // Skip donation logic if bot has already donated this round
        if bot.has_donated_this_round {
            // If already donated, only consider upgrades
            self.bot_consider_upgrades(bot_index);
            return;
        }
        
        // Get upgrade costs
        let pickaxe_cost = bot.pickaxe_upgrade_cost();
        let mine_cost = bot.mine_upgrade_cost();
        
        // Different strategies based on bot index
        match bot_index {
            0 => {
                // Bot 1: Economy-focused bot

                // If less than 3 hp, donate all gold
                if bot.health < 3 {
                    let contribution = bot.gold;
                    if contribution > 0.0 {
                        bot.contribute_gold(contribution);
                    }
                }
                
                // Only consider donating at end of round
                if is_end_of_round {
                    // Donate 10% of gold at end of round
                    let contribution = bot.gold * 0.1;
                    if contribution > 0.0 {
                        bot.contribute_gold(contribution);
                    }
                } else {
                    // Not end of round, focus on upgrades
                    self.bot_consider_upgrades(bot_index);
                }
            },
            1 => {
                // Bot 2: Aggressive end-round donator
                
                // In early rounds, focus on getting at least one upgrade
                if self.current_round <= 2 && bot.pickaxe_level == 0 && bot.mine_level == 0 {
                    if bot.gold >= pickaxe_cost {
                        bot.upgrade_pickaxe();
                        return;
                    }
                }
                
                // End of round donation with health-based amounts
                if is_end_of_round {
                    let contribution_percentage = if bot.health < 3 {
                        0.9 // 90% when critically low HP
                    } else if bot.health < 5 {
                        0.5 // 50% when low HP
                    } else {
                        0.7 // 70% normally
                    };
                    
                    let contribution = bot.gold * contribution_percentage;
                    if contribution > 0.0 {
                        bot.contribute_gold(contribution);
                    }
                } else {
                    // Not end of round, focus on upgrades
                    self.bot_consider_upgrades(bot_index);
                }
            },
            2 => {
                // Bot 3: Mixed/balanced playstyle
                
                // In very early rounds, try to get at least one upgrade first
                if self.current_round == 1 && bot.pickaxe_level == 0 && bot.mine_level == 0 {
                    if bot.gold >= pickaxe_cost {
                        bot.upgrade_pickaxe();
                        return;
                    }
                }
                
                // End of round donation with health-based amounts
                if is_end_of_round {
                    let contribution_percentage = if bot.health < 3 {
                        0.9 // 90% when critically low HP
                    } else {
                        0.3 // 30% normally
                    };
                    
                    let contribution = bot.gold * contribution_percentage;
                    if contribution > 0.0 {
                        bot.contribute_gold(contribution);
                    }
                } else {
                    // Not end of round, focus on upgrades
                    self.bot_consider_upgrades(bot_index);
                }
            },
            _ => {
                // Fallback behavior
                // Only donate at end of round
                if is_end_of_round && !bot.has_donated_this_round {
                    let mut rng = rand::thread_rng();
                    let contribution_percentage = rng.gen_range(0.1..0.4);
                    let contribution = bot.gold * contribution_percentage;
                    if contribution > 0.0 {
                        bot.contribute_gold(contribution);
                    }
                } else {
                    // Not end of round, focus on upgrades
                    self.bot_consider_upgrades(bot_index);
                }
            }
        }
    }

    fn bot_consider_upgrades(&mut self, bot_index: usize) {
        let bot = &mut self.bots[bot_index];
        
        // Skip if bot is dead
        if !bot.alive {
            return;
        }
        
        let pickaxe_cost = bot.pickaxe_upgrade_cost();
        let mine_cost = bot.mine_upgrade_cost();
        
        match bot_index {
            0 => {
                // Bot 1: Focus on upgrading the lowest level
                if bot.pickaxe_level < bot.mine_level && 
                   bot.pickaxe_level < 4 && 
                   bot.gold >= pickaxe_cost {
                    // Upgrade pickaxe since it's lower
                    bot.upgrade_pickaxe();
                } else if bot.mine_level < bot.pickaxe_level && 
                          bot.mine_level < 4 && 
                          bot.gold >= mine_cost {
                    // Upgrade mine since it's lower
                    bot.upgrade_mine();
                } else if bot.pickaxe_level < 4 && bot.gold >= pickaxe_cost {
                    // If levels are equal, upgrade pickaxe
                    bot.upgrade_pickaxe();
                } else if bot.mine_level < 4 && bot.gold >= mine_cost {
                    // If pickaxe is maxed, upgrade mine
                    bot.upgrade_mine();
                }
            },
            1 => {
                // Bot 2: Random upgrades with fallback
                let mut rng = rand::thread_rng();
                let upgrade_decision = rng.gen_range(0..2); // 0: Pickaxe, 1: Mine
                
                match upgrade_decision {
                    0 => {
                        if bot.pickaxe_level < 4 && bot.gold >= bot.pickaxe_upgrade_cost() {
                            bot.upgrade_pickaxe();
                        } else if bot.mine_level < 4 && bot.gold >= bot.mine_upgrade_cost() {
                            // Try mine upgrade as fallback
                            bot.upgrade_mine();
                        }
                    },
                    1 => {
                        if bot.mine_level < 4 && bot.gold >= bot.mine_upgrade_cost() {
                            bot.upgrade_mine();
                        } else if bot.pickaxe_level < 4 && bot.gold >= bot.pickaxe_upgrade_cost() {
                            // Try pickaxe upgrade as fallback
                            bot.upgrade_pickaxe();
                        }
                    },
                    _ => {}
                }
            },
            2 => {
                // Bot 3: Balanced upgrades
                if bot.pickaxe_level < bot.mine_level && 
                   bot.pickaxe_level < 4 && 
                   bot.gold >= pickaxe_cost {
                    // Prioritize pickaxe to catch up
                    bot.upgrade_pickaxe();
                } else if bot.mine_level < bot.pickaxe_level && 
                          bot.mine_level < 4 && 
                          bot.gold >= mine_cost {
                    // Prioritize mine to catch up
                    bot.upgrade_mine();
                } else {
                    // If levels are equal, decide randomly which to upgrade
                    let mut rng = rand::thread_rng();
                    let upgrade_choice = rng.gen_range(0..2);
                    
                    if upgrade_choice == 0 && 
                       bot.pickaxe_level < 4 && 
                       bot.gold >= pickaxe_cost {
                        bot.upgrade_pickaxe();
                    } else if upgrade_choice == 1 && 
                              bot.mine_level < 4 && 
                              bot.gold >= mine_cost {
                        bot.upgrade_mine();
                    }
                }
            },
            _ => {
                // Fallback random behavior
                let mut rng = rand::thread_rng();
                let decision = rng.gen_range(0..2); // 0: Upgrade pickaxe, 1: Upgrade mine

                match decision {
                    0 => {
                        if bot.pickaxe_level < 4 && bot.gold >= bot.pickaxe_upgrade_cost() {
                            bot.upgrade_pickaxe();
                        }
                    },
                    1 => {
                        if bot.mine_level < 4 && bot.gold >= bot.mine_upgrade_cost() {
                            bot.upgrade_mine();
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    pub fn end_round(&mut self) {
        // Collect all miners' donated gold amounts (including player)
        let mut results = Vec::new();
        
        // Add player
        results.push((0, self.player.donated_gold));
        
        // Add bots
        for (i, bot) in self.bots.iter().enumerate() {
            if bot.alive {
                results.push((i + 1, bot.donated_gold));
            }
        }
        
        // Sort by donated gold (highest first)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Record if the player won this round (was ranked #1)
        let player_won = results.first().map_or(false, |(index, _)| *index == 0);
        self.past_results.push(player_won);
        
        // Assign damage based on position
        for (position, (miner_index, _)) in results.iter().enumerate() {
            let damage = position as i32;
            
            if *miner_index == 0 {
                // Player
                self.player.take_damage(damage);
            } else {
                // Bot
                self.bots[*miner_index - 1].take_damage(damage);
            }
        }
        
        // Reset donated gold
        self.player.donated_gold = 0.0;
        for bot in &mut self.bots {
            bot.donated_gold = 0.0;
        }
        
        // Store results for display
        self.round_results = Some(results);
        
        // Check win/loss conditions
        
        // Check if player is dead
        if !self.player.alive {
            self.game_state = GameState::GameOver;
            return;
        }
        
        // Check if all bots are dead
        let bots_alive = self.bots.iter().any(|bot| bot.alive);
        if !bots_alive {
            self.game_state = GameState::GameOver;
            return;
        }
        
        // Check if max rounds reached
        if self.current_round >= MAX_ROUNDS {
            self.game_state = GameState::GameOver;
            return;
        }
        
        // If we're still here, continue to the next round
        self.game_state = GameState::RoundEnd;
    }

    pub fn player_has_won(&self) -> bool {
        // Player wins if they're alive and all bots are dead
        self.player.alive && !self.bots.iter().any(|bot| bot.alive)
    }

    pub fn start_next_round(&mut self) {
        self.current_round += 1;
        self.round_start_time = Instant::now();
        self.game_state = GameState::Playing;
        self.round_results = None;
        
        // Reset donation flags for all miners
        self.player.has_donated_this_round = false;
        for bot in &mut self.bots {
            bot.has_donated_this_round = false;
        }
    }

    pub fn restart_game(&mut self) {
        self.player = Miner::new(MinerType::Player);
        self.bots = Vec::new();
        for _ in 0..3 {
            self.bots.push(Miner::new(MinerType::Bot));
        }
        self.current_round = 1;
        self.round_start_time = Instant::now();
        self.game_state = GameState::Playing;
        self.round_results = None;
        self.past_results = Vec::new();
    }

    pub fn handle_game_ui_click(&mut self, x: f32, y: f32) {
        // Check pickaxe upgrade button
        let pickaxe_btn_rect = Rect::new(30.0, 220.0, 200.0, 40.0);
        if x >= pickaxe_btn_rect.x && x <= pickaxe_btn_rect.x + pickaxe_btn_rect.w && 
        y >= pickaxe_btn_rect.y && y <= pickaxe_btn_rect.y + pickaxe_btn_rect.h {
            self.player.upgrade_pickaxe();
        }
        
        // Check mine upgrade button
        let mine_btn_rect = Rect::new(30.0, 270.0, 200.0, 40.0);
        if x >= mine_btn_rect.x && x <= mine_btn_rect.x + mine_btn_rect.w && 
        y >= mine_btn_rect.y && y <= mine_btn_rect.y + mine_btn_rect.h {
            self.player.upgrade_mine();
        }
        
        // Check contribute buttons
        let contribution_amounts = [10.0, 50.0, 100.0, 500.0, 1000.0];
        let contrib_btn_x = WINDOW_WIDTH - 240.0;
        let contrib_btn_width = 220.0;
        
        // Check numeric contribution options
        for (i, amount) in contribution_amounts.iter().enumerate() {
            let y_pos = 190.0 + (i as f32 * 40.0);
            
            if x >= contrib_btn_x && x <= contrib_btn_x + contrib_btn_width && 
            y >= y_pos && y <= y_pos + 30.0 && *amount <= self.player.gold {
                self.player.contribute_gold(*amount);
                break;
            }
        }
        
        // Check "All" option
        let all_y_pos = 190.0 + (contribution_amounts.len() as f32 * 40.0);
        
        if x >= contrib_btn_x && x <= contrib_btn_x + contrib_btn_width && 
        y >= all_y_pos && y <= all_y_pos + 30.0 && self.player.gold > 0.0 {
            self.player.contribute_gold(self.player.gold);
        }
    }

    pub fn handle_round_end_ui_click(&mut self, x: f32, y: f32) {
        if let Some(results) = &self.round_results {
            // Calculate panel dimensions to match the UI drawing code
            let panel_height = (results.len() as f32 * 40.0) + 150.0; // Increased panel height for button
            let panel_y = WINDOW_HEIGHT / 2.0 - panel_height / 2.0;
            
            // Continue button position - exactly matching what's drawn in the UI
            let button_rect = Rect::new(
                WINDOW_WIDTH / 2.0 - 125.0,
                panel_y + panel_height - 60.0,
                250.0,
                40.0
            );
            
            if x >= button_rect.x && x <= button_rect.x + button_rect.w &&
               y >= button_rect.y && y <= button_rect.y + button_rect.h {
                self.start_next_round();
            }
        }
    }

    pub fn handle_game_over_ui_click(&mut self, x: f32, y: f32) {
        // Panel position calculation to match the UI drawing code
        let panel_rect = Rect::new(
            WINDOW_WIDTH / 2.0 - 250.0,
            WINDOW_HEIGHT / 2.0 - 200.0,
            500.0,
            400.0
        );
        
        // Check restart button - positioned to match what's drawn in the UI
        let restart_rect = Rect::new(
            WINDOW_WIDTH / 2.0 - 75.0,
            panel_rect.y + 330.0,
            150.0,
            40.0
        );
        
        if x >= restart_rect.x && x <= restart_rect.x + restart_rect.w &&
        y >= restart_rect.y && y <= restart_rect.y + restart_rect.h {
            self.restart_game();
        }
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Only update player and bots when in Playing state
        // This fixes issue with gold accumulating during round end screen
        match self.game_state {
            GameState::Playing => {
                // Update player and bots
                self.player.update(ctx);
                for bot in &mut self.bots {
                    bot.update(ctx);
                }
                
                // Make random decisions for bots
                for i in 0..self.bots.len() {
                    self.bot_make_decision(i);
                }

                // Check if round is over
                let now = Instant::now();
                let round_elapsed = now.duration_since(self.round_start_time);
                if round_elapsed >= ROUND_DURATION {
                    self.end_round();
                }
            },
            GameState::RoundEnd => {
                // Wait for player to continue - no updates to miners
            },
            GameState::GameOver => {
                // Wait for player to restart - no updates to miners
            },
        }

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        _repeat: bool,
    ) {
        // Only process cheatcodes during gameplay
        if let GameState::Playing = self.game_state {
            // Cheatcode 1: Shift+X for 1000 gold
            if keycode == KeyCode::X && keymods.contains(KeyMods::SHIFT) {
                // Add 1000 gold to player
                self.player.gold += 1000.0;
            }
            
            // Cheatcode 2: Shift+Y to skip 10 seconds
            if keycode == KeyCode::Y && keymods.contains(KeyMods::SHIFT) {
                // Adjust the round_start_time to be 10 seconds earlier
                // This makes the game think 10 more seconds have passed
                if let Some(new_time) = self.round_start_time.checked_sub(std::time::Duration::from_secs(10)) {
                    self.round_start_time = new_time;
                    
                    // If we would skip past the round end, just end the round
                    let now = std::time::Instant::now();
                    let round_elapsed = now.duration_since(self.round_start_time);
                    if round_elapsed >= ROUND_DURATION {
                        self.end_round();
                    }
                }
            }
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        use ggez::graphics::{self, Color};
        graphics::clear(ctx, Color::WHITE);

        // Draw UI based on game state
        match self.game_state {
            GameState::Playing => {
                ui::draw_game_ui(self, ctx)?;
            },
            GameState::RoundEnd => {
                ui::draw_round_end_ui(self, ctx)?;
            },
            GameState::GameOver => {
                ui::draw_game_over_ui(self, ctx)?;
            },
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) {
        if button == MouseButton::Left {
            match self.game_state {
                GameState::Playing => {
                    // Handle UI clicks during gameplay
                    self.handle_game_ui_click(x, y);
                },
                GameState::RoundEnd => {
                    // Handle round end UI clicks
                    self.handle_round_end_ui_click(x, y);
                },
                GameState::GameOver => {
                    // Handle game over UI clicks
                    self.handle_game_over_ui_click(x, y);
                },
            }
        }
    }
}