use std::time::Instant;

pub struct Pet {
    pub unlocked: bool,
    pub alive: bool,
    pub mining: bool,
    pub searching: bool,
    pub last_mine_time: Instant,
}

impl Pet {
    pub fn new() -> Self {
        Pet {
            unlocked: false,
            alive: true,
            mining: false,
            searching: false,
            last_mine_time: Instant::now(),
        }
    }
    
    pub fn unlock(&mut self) {
        self.unlocked = true;
    }
    
    pub fn toggle_mining(&mut self) {
        if self.alive && self.unlocked {
            self.mining = !self.mining;
            if self.mining {
                self.searching = false;
            }
        }
    }
    
    pub fn toggle_searching(&mut self) {
        if self.alive && self.unlocked {
            self.searching = !self.searching;
            if self.searching {
                self.mining = false;
            }
        }
    }
    
    pub fn take_hit(&mut self) {
        if self.alive {
            self.alive = false;
            self.mining = false;
            self.searching = false;
        }
    }
}