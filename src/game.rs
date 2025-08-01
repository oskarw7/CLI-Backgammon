use crate::utils::*;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEvent, poll, read},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{Write, stdout};

const PLAYER_1: char = 'w';
const PLAYER_2: char = 'b';
const MENU_LINE_NUMBDER: u16 = 20;
const PROMPT_LINE_NUMBER: u16 = 21;
const INPUT_LINE_NUMBER: u16 = 22;

#[derive(Debug)]
pub struct Game {
    board: [u8; 24],
    turn: char,
    roll_result: Vec<u8>,
    is_running: bool,
}

impl Game {
    pub fn new() -> Self {
        terminal::enable_raw_mode().unwrap();
        execute!(stdout(), EnterAlternateScreen, Hide).unwrap();

        Self {
            board: [
                2 + 15,
                0,
                0,
                0,
                0,
                5,
                0,
                3,
                0,
                0,
                0,
                5 + 15,
                5,
                0,
                0,
                0,
                3 + 15,
                0,
                5 + 15,
                0,
                0,
                0,
                0,
                2,
            ], // white takes 1-15, black takes 16-30
            turn: PLAYER_1, // rust doesn't tolerate uninitialized fields, needed
            roll_result: Vec::new(),
            is_running: true,
        }
    }

    fn move_checker(&mut self, source: usize, destination: usize) {
        if self.board[source - 1] > 15 && self.board[destination - 1] == 0 {
            self.board[destination - 1] += 15;
        }
        self.board[source - 1] -= 1;
        if self.board[source - 1] == 15 {
            self.board[source - 1] = 0;
        }
        self.board[destination - 1] += 1; // TODO: add capture logic
    }

    fn draw_checker(&self, index: usize) {
        if self.board[index] <= 15 {
            print!("●");
        } else {
            print!("○");
        }
    }

    fn draw_empty_field(&self, i: usize) {
        for j in 0..3 {
            if i < 12 {
                move_cursor(((11 - i) * 5) as u16, (15 - j) as u16);
            } else {
                move_cursor(((i - 12) * 5) as u16, j as u16);
            }
            print!("|");
        }
    }

    fn clear_board(&self) {
        for i in 0..20 {
            clear_line(i);
        }
    }

    fn draw_board(&self) {
        self.clear_board();
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
                    move_cursor(((11 - i) * 5) as u16, (15 - j) as u16); // TODO: add checker count threshold
                } else {
                    move_cursor(((i - 12) * 5) as u16, j as u16);
                }
                self.draw_checker(i);
            }
        }
        move_cursor(0, 0);
    }

    fn draw(&self) {
        self.draw_board();
        // TODO: menu, other UI components
    }

    fn get_number(&mut self, mode: &str) -> Option<u8> {
        print_message(0, MENU_LINE_NUMBDER, "Q)uit, ESC - reset selection");
        let prompt = format!("Enter {mode} number:");
        print_message(0, PROMPT_LINE_NUMBER, &prompt);

        let mut input = String::new();
        while self.is_running {
            if let Ok(event) = read() {
                if let Event::Key(key_event) = event {
                    clear_line(INPUT_LINE_NUMBER);
                    match key_event.code {
                        KeyCode::Char(c) if c.is_digit(10) => {
                            input.push(c);
                            move_cursor(0, INPUT_LINE_NUMBER);
                            println!("{}", input);
                        }
                        KeyCode::Enter => {
                            if let Ok(num) = input.parse::<u8>() {
                                if (1..=24).contains(&num) {
                                    clear_line(INPUT_LINE_NUMBER);
                                    return Some(num);
                                } else {
                                    print_temp_message(
                                        0,
                                        INPUT_LINE_NUMBER,
                                        "Invalid number",
                                        1000,
                                    );
                                }
                            } else {
                                print_temp_message(0, INPUT_LINE_NUMBER, "Invalid number", 1000);
                            }
                            input.clear();
                        }
                        KeyCode::Backspace => {
                            input.pop();
                            println!("{}", input);
                        }
                        KeyCode::Esc => break,
                        KeyCode::Char('q') => self.quit(),
                        _ => {}
                    }
                }
            }
        }
        None
    }

    pub fn play(&mut self) {
        while self.is_running {
            self.draw();
            print_message(0, MENU_LINE_NUMBDER, "R)oll, Q)uit, ESC - back to menu");
            /*
            if let Ok(event) = read() {
                if let Event::Key(key_event) = event {
                    match key_event.code {
                        KeyCode::Esc => return,
                        KeyCode::Char('q') => self.quit(),
                        _ => {}
                    }
                }
            }
            */
            if let Some(source) = self.get_number("source") {
                if let Some(destination) = self.get_number("destination") {
                    self.move_checker(source as usize, destination as usize);
                }
            }
        }
    }

    fn quit(&mut self) {
        self.is_running = false;
    }

    pub fn run(&mut self) {
        while self.is_running {
            self.draw();
            print_message(0, MENU_LINE_NUMBDER, "P)lay, Q)uit");
            if let Ok(event) = read() {
                if let Event::Key(key_event) = event {
                    match key_event.code {
                        KeyCode::Char('p') => self.play(),
                        KeyCode::Char('q') => self.quit(),
                        _ => {}
                    }
                }
            }
        }

        // Cleanup
        execute!(stdout(), LeaveAlternateScreen, Show).unwrap();
        terminal::disable_raw_mode().unwrap();
    }
}
