use crate::utils::*;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEvent, poll, read},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use std::{
    fmt::format,
    io::{Write, stdout},
};

const WHITE: char = 'w';
const BLACK: char = 'b';
const LINE_NUMBER_1: u16 = 17;
const LINE_NUMBER_2: u16 = 18;
const LINE_NUMBER_3: u16 = 19;
const LINE_NUMBER_4: u16 = 20;
const LINE_NUMBER_5: u16 = 21;

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
            turn: WHITE, // rust doesn't tolerate uninitialized fields, needed
            roll_result: Vec::new(),
            is_running: true,
        }
    }

    fn which_color(&self, field: usize) -> Option<char> {
        if self.board[field - 1] >= 1 && self.board[field - 1] <= 15 {
            return Some(WHITE);
        } else if self.board[field - 1] >= 16 && self.board[field - 1] <= 30 {
            return Some(BLACK);
        }
        None
    }

    fn is_move_valid(&self, source: usize, destination: usize) -> bool {
        // withing board bounds
        if source < 1 || source > 24 || destination < 1 || source > 24 {
            return false;
        }

        // has checker on source field
        if self.board[source - 1] == 0 {
            return false;
        }

        // right color
        if let Some(color) = self.which_color(source) {
            if color != self.turn {
                return false;
            }
        }
        if let Some(color) = self.which_color(destination) {
            if color != self.turn {
                return false;
            }
        }

        if self.turn == WHITE {
            // right direction
            if destination >= source {
                return false;
            }
        } else {
            // right direction
            if destination <= source {
                return false;
            }
        }
        true
    }

    fn move_checker(&mut self, source: usize, destination: usize) {
        // black moves to empty fields
        if self.board[source - 1] > 15 && self.board[destination - 1] == 0 {
            self.board[destination - 1] += 15;
        }
        self.board[source - 1] -= 1;
        // black moves single checker
        if self.board[source - 1] == 15 {
            self.board[source - 1] = 0;
        }
        self.board[destination - 1] += 1; // TODO: add capture logic
    }

    fn roll() -> u8 {
        let mut rng = rand::rng();
        rng.random_range(1..=6)
    }

    fn handle_roll(&mut self) {
        self.roll_result.clear();

        let dice_1 = Game::roll();
        self.roll_result.push(dice_1);
        let dice_2 = Game::roll();
        self.roll_result.push(dice_2);

        let mut dice_str = String::new();
        if dice_1 != dice_2 {
            dice_str = format!("Result: {dice_1}, {dice_2}");
        } else {
            self.roll_result.push(dice_1);
            self.roll_result.push(dice_1);
            dice_str = format!("Result: {dice_1}, {dice_1}, {dice_1}, {dice_1}");
        }
        print_message(0, LINE_NUMBER_5, &dice_str);
    }

    fn change_turn(&mut self) {
        if self.turn == WHITE {
            self.turn = BLACK;
        } else {
            self.turn = WHITE;
        }
    }

    fn print_turn(&self) {
        if self.turn == WHITE {
            print_message(0, LINE_NUMBER_2, "White's turn");
        } else {
            print_message(0, LINE_NUMBER_2, "Black's turn");
        }
    }

    fn draw_checker(&self, index: usize) {
        if self.board[index] <= 15 {
            print!("●");
        } else {
            print!("○");
        }
    }

    fn draw_empty_field(i: usize) {
        for j in 0..3 {
            if i < 12 {
                move_cursor(((11 - i) * 5) as u16, (15 - j) as u16);
            } else {
                move_cursor(((i - 12) * 5) as u16, j as u16);
            }
            print!("|");
        }
    }

    fn clear_board() {
        for i in 0..15 {
            clear_line(i);
        }
    }

    fn draw_board(&self) {
        clear_screen();
        for i in 0..self.board.len() {
            let mut checker_count = self.board[i];
            if checker_count == 0 {
                Game::draw_empty_field(i);
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
        print_message(0, LINE_NUMBER_1, "ESC - reset selection");
        let prompt = format!("Enter {mode} number:");
        print_message(0, LINE_NUMBER_3, &prompt);

        let mut input = String::new();
        while self.is_running {
            if let Ok(event) = read() {
                if let Event::Key(key_event) = event {
                    clear_line(LINE_NUMBER_4);
                    match key_event.code {
                        KeyCode::Char(c) if c.is_digit(10) => {
                            input.push(c);
                            let input_str = input.to_string();
                            print_message(0, 20, &input_str);
                        }
                        KeyCode::Enter => {
                            if let Ok(num) = input.parse::<u8>() {
                                if (1..=24).contains(&num) {
                                    clear_line(LINE_NUMBER_4);
                                    return Some(num);
                                } else {
                                    print_temp_message(0, LINE_NUMBER_4, "Invalid number", 1000);
                                }
                            } else {
                                print_temp_message(0, LINE_NUMBER_4, "Invalid number", 1000);
                            }
                            input.clear();
                        }
                        KeyCode::Backspace => {
                            input.pop();
                            println!("{}", input);
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
        None
    }

    fn choose_who_starts(&mut self) {
        let mut rolls_count = 0;
        while rolls_count < 2 {
            self.draw();
            self.print_turn();
            print_message(0, LINE_NUMBER_1, "R)oll, Q)uit");
            if rolls_count == 0 {
                self.change_turn();
            }
            if let Ok(event) = read() {
                if let Event::Key(key_event) = event {
                    match key_event.code {
                        KeyCode::Char('r') => {
                            rolls_count += 1;
                            let dice = Game::roll();
                            self.roll_result.push(dice);
                            let dice_str = format!("Result: {dice}");
                            print_temp_message(0, LINE_NUMBER_5, &dice_str, 1000);
                        }
                        KeyCode::Char('q') => {
                            self.quit();
                            return;
                        }
                        _ => {}
                    }
                }
            }
            if rolls_count == 2 && self.roll_result[0] == self.roll_result[1] {
                self.roll_result.clear();
                rolls_count = 0;
                self.change_turn();
                print_temp_message(0, LINE_NUMBER_3, "Tie", 1000);
            }
        }
        if self.roll_result[0] > self.roll_result[1] {
            self.turn = WHITE;
            print_temp_message(0, LINE_NUMBER_3, "White starts", 1000);
        } else {
            self.turn = BLACK;
            print_temp_message(0, LINE_NUMBER_3, "Black starts", 1000);
        }
    }

    fn play(&mut self) {
        self.choose_who_starts();
        while self.is_running {
            self.draw();
            print_message(0, LINE_NUMBER_1, "R)oll, Q)uit, ESC - back to menu");
            self.print_turn();
            if let Ok(event) = read() {
                if let Event::Key(key_event) = event {
                    match key_event.code {
                        KeyCode::Char('r') => {
                            self.handle_roll();
                            loop {
                                if let Some(source) = self.get_number("source") {
                                    if let Some(destination) = self.get_number("destination") {
                                        if self.is_move_valid(source as usize, destination as usize)
                                        {
                                            self.move_checker(source as usize, destination as usize);
                                            break;
                                        } else {
                                            print_temp_message(0, LINE_NUMBER_4, "Invalid move", 1000);
                                        }
                                    }
                                }
                            }
                            self.change_turn();
                        }
                        KeyCode::Esc => return,
                        KeyCode::Char('q') => self.quit(),
                        _ => {}
                    }
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
            print_message(0, LINE_NUMBER_1, "P)lay, Q)uit");
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
