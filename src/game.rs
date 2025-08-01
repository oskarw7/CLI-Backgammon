use std::io::{stdout, Write};
use crossterm::{
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{MoveTo, Hide, Show},
    event::{self, Event, KeyCode},
};
use crate::utils::*;

const PLAYER_1: char = 'w';
const PLAYER_2: char = 'b';

#[derive(Debug)]
pub struct Game {
    board: [u8; 24],
    turn: char,
    roll_result: Vec<u8>,
}

impl Game {
    pub fn new() -> Self {
        terminal::enable_raw_mode().unwrap();
        execute!(stdout(), EnterAlternateScreen, Hide).unwrap();
        
        Self {
            board: [2+15, 0, 0, 0, 0, 5, 0, 3, 0, 0, 0, 5+15, 5, 0, 0, 0, 3+15, 0, 5+15, 0, 0, 0, 0, 2], // white takes 1-15, black takes 16-30
            turn: PLAYER_1, // rust doesn't tolerate uninitialized fields, needed
            roll_result: Vec::new(),
        }
    }

    fn draw_checker(&self, index: usize) {
        if self.board[index] <= 15 {
            print!("●");
        }
        else {
            print!("○");
        }
    }

    fn draw_empty_field(&self, i: usize) {
        for j in 0..3 {
            if i < 12 {
                move_cursor(((11 - i)*5) as u16, (15 - j) as u16);
            } else {
                move_cursor(((i - 12)*5) as u16, j as u16);
            }
            print!("|");
        }
    }

    fn draw_board(&self) {
        clear_screen();
        for i in 0..self.board.len() {
            let mut checker_count = self.board[i];
            if checker_count == 0 {
                self.draw_empty_field(i);
                continue;
            } 
            if checker_count > 15 {
                checker_count -= 15;
            }
            for j in 0..checker_count {
                if i < 12 {
                    move_cursor(((11-i)*5) as u16, (15-j) as u16); // TODO: add checker count threshold
                }
                else {
                    move_cursor(((i-12)*5) as u16, j as u16);
                }
                self.draw_checker(i);
            }
        }  
        move_cursor(0, 0);
    }

    pub fn run(&self) {
        self.draw_board();  

        // Cleanup
        execute!(stdout(), LeaveAlternateScreen, Show).unwrap();
        terminal::disable_raw_mode().unwrap();
    }
}
