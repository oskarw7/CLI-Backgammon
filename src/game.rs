use crate::utils::*;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEvent, poll, read},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use std::io::stdout;

const WHITE: u8 = 0;
const BLACK: u8 = 1;
const BOARD_OFFSET: u16 = 1;
const LINE_NUMBER_1: u16 = 17;
const LINE_NUMBER_2: u16 = 18;
const LINE_NUMBER_3: u16 = 19;
const LINE_NUMBER_4: u16 = 20;
const LINE_NUMBER_5: u16 = 21;
const LINE_NUMBER_6: u16 = 22;

#[derive(Debug)]
pub struct Game {
    board: [u8; 24],
    turn: u8,
    roll_result: Vec<u8>,
    moves: Vec<(usize, usize)>,
    bar: [u8; 2],
    tray: [u8; 2],
    is_over: bool,    // is certain game finished
    is_running: bool, // is whole program running
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
            /*
            board: [
                0,
                0,
                0,
                3,
                5,
                7,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                7+15,
                5+15,
                3+15,
                0,
                0,
                0,
            ], // white takes 1-15, black takes 16-30
            */
            turn: WHITE, // rust doesn't tolerate uninitialized fields, needed
            roll_result: Vec::new(),
            moves: Vec::new(),
            bar: [0, 0],
            tray: [0, 0],
            is_over: false,
            is_running: true,
        }
    }

    fn which_color(&self, field: usize) -> Option<u8> {
        if self.board[field - 1] >= 1 && self.board[field - 1] <= 15 {
            return Some(WHITE);
        } else if self.board[field - 1] >= 16 && self.board[field - 1] <= 30 {
            return Some(BLACK);
        }
        None
    }

    fn are_all_home(&self, color: u8) -> bool {
        let mut checker_count = 0;
        if color == WHITE {
            for field in (1..=6).rev() {
                if self.which_color(field) == Some(WHITE) {
                    checker_count += self.board[field - 1];
                }
            }
        } else {
            for field in 19..=24 {
                if self.which_color(field) == Some(BLACK) {
                    checker_count += self.board[field - 1] - 15;
                }
            }
        }

        if checker_count == 15 - self.tray[self.turn as usize] {
            return true;
        }
        false
    }

    fn farthest_to_tray(&self, color: u8) -> Option<usize> {
        if color == WHITE {
            for field in (1..=6).rev() {
                if self.which_color(field) == Some(WHITE) {
                    return Some(field);
                }
            }
        } else {
            for field in 19..=24 {
                if self.which_color(field) == Some(BLACK) {
                    return Some(field);
                }
            }
        }
        None
    }

    fn validate_home(&self, source: usize, destination: usize) -> bool {
        // 0 and 25 represent tray for white and black, respectively
        let valid_destination = if self.turn == WHITE { 0 } else { 25 };
        if destination != valid_destination || source < 1 || source > 24 {
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

        // right direction
        if self.turn == WHITE {
            if destination >= source {
                return false;
            }
        } else {
            if destination <= source {
                return false;
            }
        }
        true
    }

    fn validate_bar(&self, source: usize, destination: usize) -> bool {
        // 25 and 0 represent bar for white and black, respectively
        let valid_source = if self.turn == WHITE { 25 } else { 0 };
        if source != valid_source || destination < 1 || destination > 24 {
            return false;
        }

        // right color
        if let Some(color) = self.which_color(destination) {
            if color != self.turn {
                if !(self.board[destination - 1] == 1 || self.board[destination - 1] == 16) {
                    return false;
                }
            }
        }

        // right direction
        if self.turn == WHITE {
            if destination >= source {
                return false;
            }
        } else {
            if destination <= source {
                return false;
            }
        }
        true
    }

    fn is_move_valid(&self, source: usize, destination: usize) -> bool {
        if self.bar[self.turn as usize] > 0 {
            return self.validate_bar(source, destination);
        }

        // withing board bounds
        if source < 1 || source > 24 || destination < 1 || destination > 24 {
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
                if !(self.board[destination - 1] == 1 || self.board[destination - 1] == 16) {
                    return false;
                }
            }
        }

        // right direction
        if self.turn == WHITE {
            if destination >= source {
                return false;
            }
        } else {
            if destination <= source {
                return false;
            }
        }
        true
    }

    fn add_moves_to_tray_dice(&mut self, destination: usize, mut dice: isize) {
        if let Some(field) = self.farthest_to_tray(self.turn) {
            if dice > (destination as i32 - field as i32).abs() as isize {
                dice = (destination as i32 - field as i32).abs() as isize;
            }
        }
        if self.turn == BLACK {
            dice *= -1;
        }
        let src = destination as isize + dice;
        if src >= 1 && src <= 24 && self.validate_home(src as usize, destination) {
            self.moves.push((src as usize, destination));
        }
    }

    fn add_moves_to_tray(&mut self) {
        let destination: usize = if self.turn == WHITE { 0 } else { 25 };
        let mut dice: isize = self.roll_result[0] as isize;
        self.add_moves_to_tray_dice(destination, dice);

        if self.roll_result.len() > 1 && self.roll_result[0] != self.roll_result[1] {
            dice = self.roll_result[1] as isize;
            self.add_moves_to_tray_dice(destination, dice);
        }
    }

    fn add_moves_from_dice(&mut self, source: usize, mut dice: isize) {
        if self.turn == WHITE {
            dice *= -1;
        }
        let dest = source as isize + dice;
        if dest >= 1 && dest <= 24 && self.is_move_valid(source, dest as usize) {
            self.moves.push((source, dest as usize));
        }
    }

    fn add_moves_from(&mut self, source: usize) {
        let mut dice: isize = self.roll_result[0] as isize;
        self.add_moves_from_dice(source, dice);

        if self.roll_result.len() > 1 && self.roll_result[0] != self.roll_result[1] {
            dice = self.roll_result[1] as isize;
            self.add_moves_from_dice(source, dice);
        }
    }

    fn generate_moves(&mut self) {
        self.moves.clear();

        // moves from bar first
        if self.bar[self.turn as usize] > 0 {
            let source: usize = if self.turn == WHITE { 25 } else { 0 };
            self.add_moves_from(source);
        } else {
            // typical moves
            for source in 1..=24 {
                if let Some(color) = self.which_color(source) {
                    if color == self.turn {
                        self.add_moves_from(source);
                    }
                }
            }

            // moves to tray
            if self.are_all_home(self.turn) {
                self.add_moves_to_tray();
            }
        }

        // sort and remove duplicates (needed for forced bear off)
        self.moves.sort();
        self.moves.dedup();
    }

    fn move_checker(&mut self, source: usize, destination: usize) {
        // moves to tray / other moves
        if self.are_all_home(self.turn) && (destination == 0 || destination == 25) {
            self.tray[self.turn as usize] += 1;
            // removing the roll if taking of was forced (smaller move than the greatest roll)
            // TODO: test that
            if let Some((index, &max)) = self
                .roll_result
                .iter()
                .enumerate()
                .max_by_key(|&(_, &val)| val)
            {
                let diff = (destination as i32 - source as i32).abs() as u8;
                if !self.roll_result.contains(&diff) && max > diff {
                    self.roll_result.remove(index);
                }
            }
        } else {
            // checker gets captured
            if let Some(color) = self.which_color(destination) {
                if color != self.turn {
                    self.board[destination - 1] = 0;
                    self.bar[color as usize] += 1;
                }
            }

            // black moves to empty fields
            if self.turn == BLACK && self.board[destination - 1] == 0 {
                self.board[destination - 1] += 15;
            }
            self.board[destination - 1] += 1;
        }

        // moves from bar / other moves
        if self.bar[self.turn as usize] != 0 {
            self.bar[self.turn as usize] -= 1;
        } else {
            self.board[source - 1] -= 1;
            // black moves single checker
            if self.board[source - 1] == 15 {
                self.board[source - 1] = 0;
            }
        }

        // removing the roll
        if let Some(index) = self
            .roll_result
            .iter()
            .position(|&x| x == ((destination as i32 - source as i32).abs()) as u8)
        {
            self.roll_result.remove(index);
        }
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

    fn print_moves(&self) {
        let mut moves_str = self
            .moves
            .iter()
            .map(|(src, dst)| format!("{src}->{dst}"))
            .collect::<Vec<String>>()
            .join(", ");
        moves_str = format!("Moves: {moves_str}");
        print_message(0, LINE_NUMBER_6, &moves_str);
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
                move_cursor(((11 - i) * 5) as u16, (15 - (j + BOARD_OFFSET)) as u16);
            } else {
                move_cursor(((i - 12) * 5) as u16, j + BOARD_OFFSET as u16);
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
        Game::clear_board();
        print_message(
            0,
            0,
            "13   14   15   16   17   18   19   20   21   22   23   24",
        );
        print_message(
            0,
            15,
            "12   11   10   9    8    7    6    5    4    3    2    1",
        );
        for i in 0..self.board.len() {
            let mut checker_count = self.board[i] as u16;
            if checker_count == 0 {
                Game::draw_empty_field(i);
                continue;
            }
            if checker_count > 15 {
                checker_count -= 15;
            }
            for j in 0..checker_count {
                if i < 12 {
                    move_cursor(((11 - i) * 5) as u16, 15 - (j + BOARD_OFFSET)); // TODO: add checker count threshold
                } else {
                    move_cursor(((i - 12) * 5) as u16, j + BOARD_OFFSET);
                }
                self.draw_checker(i);
            }
        }
        move_cursor(60, 0);
        print!(
            "Bar:  {} x ●, {} x ○",
            self.bar[WHITE as usize], self.bar[BLACK as usize]
        );
        move_cursor(65, 1);
        print!(" 25     0");
        move_cursor(60, 3);
        print!(
            "Tray: {} x ●, {} x ○",
            self.tray[WHITE as usize], self.tray[BLACK as usize]
        );
        move_cursor(65, 4);
        print!(" 0      25");
    }

    fn draw(&self) {
        clear_screen();
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
                                if (0..=25).contains(&num) {
                                    // temporary for moving from bar
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

    fn reset(&mut self) {
        self.board = [
            0,
            0,
            0,
            3,
            5,
            7,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            7 + 15,
            5 + 15,
            3 + 15,
            0,
            0,
            0,
        ];
        self.turn = WHITE;
        self.roll_result.clear();
        self.moves.clear();
        self.bar = [0, 0];
        self.tray = [0, 0];
        self.is_over = false;
        self.is_running = true;
    }

    fn check_is_over(&mut self) -> bool {
        if self.tray[self.turn as usize] == 15 {
            self.is_over = true;
            let who_won = if self.turn == WHITE { "White" } else { "Black" };
            let message = format!("{who_won} has won! Exiting to main menu...");
            print_temp_message(0, LINE_NUMBER_4, &message, 3000);
            self.reset();
            return true;
        }
        false
    }

    fn quit(&mut self) {
        self.is_running = false;
    }

    fn choose_who_starts(&mut self) {
        let mut rolls_count = 0;
        while rolls_count < 2 {
            self.draw();
            self.print_turn();
            print_message(0, LINE_NUMBER_1, "R)oll, Q)uit");
            if let Ok(event) = read() {
                if let Event::Key(key_event) = event {
                    match key_event.code {
                        KeyCode::Char('r') => {
                            rolls_count += 1;
                            let dice = Game::roll();
                            self.roll_result.push(dice);
                            let dice_str = format!("Result: {dice}");
                            print_temp_message(0, LINE_NUMBER_5, &dice_str, 1000);
                            self.change_turn();
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
            print_message(0, LINE_NUMBER_1, "R)oll, S)ave, Q)uit, ESC - back to menu");
            self.print_turn();
            if let Ok(event) = read() {
                if let Event::Key(key_event) = event {
                    match key_event.code {
                        KeyCode::Char('r') => {
                            self.handle_roll();
                            while !self.roll_result.is_empty() {
                                self.draw_board();
                                self.print_turn();
                                self.generate_moves();
                                if self.moves.is_empty() {
                                    print_temp_message(0, LINE_NUMBER_4, "No moves possible", 1000);
                                    break;
                                }
                                self.print_moves();
                                if let Some(source) = self.get_number("source") {
                                    if let Some(destination) = self.get_number("destination") {
                                        if self
                                            .moves
                                            .contains(&(source as usize, destination as usize))
                                        {
                                            self.move_checker(
                                                source as usize,
                                                destination as usize,
                                            );
                                        } else {
                                            print_temp_message(
                                                0,
                                                LINE_NUMBER_4,
                                                "Invalid move",
                                                1000,
                                            );
                                        }
                                    }
                                }
                                if self.check_is_over() {
                                    return;
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
