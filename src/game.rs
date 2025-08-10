use crate::utils::*;
use chrono::Local;
use crossterm::{
    cursor::{Hide, Show},
    event::{Event, KeyCode, read},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use std::{
    fs::{self, File, OpenOptions, read_to_string},
    io::{BufRead, BufReader, Error, ErrorKind, Write, stdin, stdout},
    u32,
};

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
pub struct PreviousMovesBuffer {
    board: [u8; 24],
    turn: u8,
    bar: [u8; 2],
    tray: [u8; 2],
}

#[derive(Debug)]
pub struct Game {
    board: [u8; 24],
    turn: u8,
    roll_result: Vec<u8>,
    moves: Vec<(usize, usize)>,
    previous_moves: Vec<PreviousMovesBuffer>,
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
            ], // white takes 1-15, black takes 16-3
            turn: WHITE, // rust doesn't tolerate uninitialized fields, needed
            roll_result: Vec::new(),
            moves: Vec::new(),
            previous_moves: Vec::new(),
            bar: [0, 0],
            tray: [0, 0],
            is_over: false,
            is_running: true,
        }
    }

    fn get_leaderboard(&mut self) -> std::io::Result<()> {
        let mut leaderboard = Vec::new();

        if let Ok(content) = read_to_string("saves/leaderboard.txt") {
            for line in content.lines() {
                let mut parts = line.split_whitespace();
                if let (Some(nick), Some(score)) = (parts.next(), parts.next()) {
                    if let Ok(score) = score.parse::<u32>() {
                        leaderboard.push((nick.to_string(), score));
                    }
                }
            }
        }

        if leaderboard.is_empty() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "No entried found in leaderboard",
            ));
        }

        leaderboard.sort_by(|a, b| b.1.cmp(&a.1));

        clear_screen();
        let mut i = 0;
        print_message(0, i, "LEADERBOARD:");
        for (nick, score) in leaderboard {
            i += 1;
            let message = format!("{}. {} {}", i, nick, score);
            print_message(0, i, &message);
        }
        print_message(0, i + 2, "Press anything to go back, q to quit");

        if let Ok(event) = read() {
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Char('q') => self.quit(),
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn get_nick() -> std::io::Result<String> {
        terminal::disable_raw_mode()?;

        move_cursor(0, LINE_NUMBER_4);
        let mut nick = String::new();
        stdin().read_line(&mut nick)?;
        nick = nick.trim_end().to_string();

        terminal::enable_raw_mode()?;
        Ok(nick)
    }

    fn update_leaderboard() -> std::io::Result<()> {
        let nick = Self::get_nick()?;
        let mut leaderboard = Vec::new();
        let mut is_found = false;

        fs::create_dir_all("saves/")?;

        if let Ok(content) = read_to_string("saves/leaderboard.txt") {
            for line in content.lines() {
                let mut parts = line.split_whitespace();
                if let (Some(file_nick), Some(score)) = (parts.next(), parts.next()) {
                    if nick == file_nick {
                        if let Ok(score) = score.parse::<u32>() {
                            leaderboard.push(format!("{} {}", nick, score + 1));
                            is_found = true;
                        }
                    } else {
                        leaderboard.push(format!("{} {}", file_nick, score));
                    }
                }
            }
        }

        if !is_found {
            leaderboard.push(format!("{} {}", nick, 1));
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("saves/leaderboard.txt")?;

        for entry in leaderboard {
            writeln!(file, "{}", entry)?;
        }

        Ok(())
    }

    fn save_to_file(&self) -> std::io::Result<()> {
        fs::create_dir_all("saves/games")?;
        let filename = Local::now().format("saves/games/%H%M%S_%m%m%Y").to_string();
        let mut file = File::create(&filename)?;

        // board
        for field in self.board {
            write!(file, "{} ", field)?;
        }
        writeln!(file)?;

        // next turn
        writeln!(file, "{}", self.turn)?;

        // bar
        for field in self.bar {
            write!(file, "{} ", field)?;
        }
        writeln!(file)?;

        // tray
        for field in self.tray {
            write!(file, "{} ", field)?;
        }

        Ok(())
    }

    fn save_previous_moves(&self) -> std::io::Result<()> {
        fs::create_dir_all("saves/replays")?;
        let filename = Local::now()
            .format("saves/replays/%H%M%S_%m%m%Y")
            .to_string();
        let mut file = File::create(&filename)?;

        for mv in &self.previous_moves {
            // board
            for field in mv.board {
                write!(file, "{} ", field)?;
            }
            writeln!(file)?;

            // next turn
            writeln!(file, "{}", mv.turn)?;

            // bar
            for field in mv.bar {
                write!(file, "{} ", field)?;
            }
            writeln!(file)?;

            // tray
            for field in mv.tray {
                write!(file, "{} ", field)?;
            }
            writeln!(file)?;
        }

        Ok(())
    }

    fn parse_line_to_vec(
        reader: &mut BufReader<File>,
        line: &mut String,
    ) -> std::io::Result<Vec<u8>> {
        line.clear();
        if reader.read_line(line)? == 0 {
            return Ok(Vec::new());
        }
        Ok(line
            .split_whitespace()
            .filter_map(|x| x.parse().ok())
            .collect())
    }

    fn get_filename(&mut self, dir_path: &str) -> std::io::Result<Option<String>> {
        clear_screen();
        print_message(0, 0, "Select save to read from:");
        let mut filenames = Vec::new();
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                filenames.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        filenames.sort();
        for i in 0..filenames.len() {
            let message = format!("{}. {}", i + 1, filenames[i]);
            print_message(0, (i + 1) as u16, &message);
        }

        let mut cursor = 0 as u16;
        let lenght = filenames.len() as u16;
        while self.is_running {
            for j in 0..filenames.len() {
                if j as u16 != cursor {
                    print_at(20, (j + 1) as u16, " ");
                } else {
                    print_at(20, cursor + 1, "<");
                }
            }
            if let Ok(event) = read() {
                if let Event::Key(key_event) = event {
                    match key_event.code {
                        KeyCode::Up => cursor = if cursor == 0 { lenght - 1 } else { cursor - 1 },
                        KeyCode::Down => cursor = (cursor + 1) % lenght,
                        KeyCode::Enter => return Ok(Some(filenames[cursor as usize].clone())),
                        KeyCode::Esc => break,
                        KeyCode::Char('q') => self.quit(),
                        _ => {}
                    }
                }
            }
        }
        Ok(None)
    }

    fn read_from_file(&mut self) -> std::io::Result<()> {
        if let Some(filename) = self.get_filename("saves/games")? {
            let path = format!("saves/games/{filename}");
            let file = File::open(&path)?;
            let mut reader = BufReader::new(file);
            let mut line = String::new();

            // Read board
            let temp = Self::parse_line_to_vec(&mut reader, &mut line)?;
            if temp.len() != 24 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Expected 24 elements for board in save file",
                ));
            }
            for (i, field) in temp.iter().enumerate() {
                self.board[i] = *field;
            }

            // Read turn
            line.clear();
            reader.read_line(&mut line)?;
            self.turn = line.trim().parse().unwrap_or(WHITE);

            // Read bar
            let temp = Self::parse_line_to_vec(&mut reader, &mut line)?;
            if temp.len() != 2 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Expected 2 elements for bar in save file",
                ));
            }
            for (i, field) in temp.iter().enumerate() {
                self.bar[i] = *field;
            }

            // Read tray
            let temp = Self::parse_line_to_vec(&mut reader, &mut line)?;
            if temp.len() != 2 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Expected 2 elements for tray in save file",
                ));
            }
            for (i, field) in temp.iter().enumerate() {
                self.tray[i] = *field;
            }
        } else {
            return Err(Error::new(ErrorKind::Other, "")); // to simplify returned value
        }
        Ok(())
    }

    fn load_replay(&mut self) -> std::io::Result<()> {
        if let Some(filename) = self.get_filename("saves/replays")? {
            let path = format!("saves/replays/{filename}");
            let file = File::open(&path)?;
            let mut reader = BufReader::new(file);
            let mut line = String::new();
            self.previous_moves.clear();

            loop {
                // Read board
                let board = Self::parse_line_to_vec(&mut reader, &mut line)?;
                if board.len() != 24 {
                    break;
                }

                // Read turn
                line.clear();
                reader.read_line(&mut line)?;
                let turn = line.trim().parse().unwrap_or(WHITE);

                // Read bar
                let bar = Self::parse_line_to_vec(&mut reader, &mut line)?;
                if bar.len() != 2 {
                    break;
                }

                // Read tray
                let tray = Self::parse_line_to_vec(&mut reader, &mut line)?;
                if tray.len() != 2 {
                    break;
                }

                self.previous_moves.push(PreviousMovesBuffer {
                    board: board.try_into().unwrap(),
                    turn,
                    bar: bar.try_into().unwrap(),
                    tray: tray.try_into().unwrap(),
                });
            }
        } else {
            return Err(Error::new(ErrorKind::Other, "")); // to simplify returned value
        }
        Ok(())
    }

    fn record_move(&mut self) {
        self.previous_moves.push(PreviousMovesBuffer {
            board: self.board,
            turn: self.turn,
            bar: self.bar,
            tray: self.tray,
        });
    }

    fn show_move(&mut self, index: usize) {
        if let Some(mv) = self.previous_moves.get(index) {
            self.board = mv.board;
            self.turn = mv.turn;
            self.bar = mv.bar;
            self.tray = mv.tray;
        }

        self.draw_board();
        print_message(
            0,
            LINE_NUMBER_1,
            "←/→ - one move back/forward, s/e - first/last move",
        );
        if index == 0 {
            print_message(0, LINE_NUMBER_2, "Beggining");
            clear_line(LINE_NUMBER_3);
        } else {
            if self.turn == WHITE {
                print_message(0, LINE_NUMBER_2, "White's turn");
            } else {
                print_message(0, LINE_NUMBER_2, "Black's turn");
            }
            let message = format!("Move number: {}", index);
            print_message(0, LINE_NUMBER_3, &message);
        }
    }

    fn visualize_replay(&mut self) {
        if self.load_replay().is_ok() {
            let mut cursor = 0;
            let last_index = self.previous_moves.len() - 1;
            self.show_move(cursor);
            while self.is_running {
                if let Ok(event) = read() {
                    if let Event::Key(key_event) = event {
                        match key_event.code {
                            KeyCode::Left => {
                                if cursor > 0 {
                                    cursor -= 1;
                                    self.show_move(cursor);
                                }
                            }
                            KeyCode::Right => {
                                if cursor < last_index {
                                    cursor += 1;
                                    self.show_move(cursor);
                                }
                            }
                            KeyCode::Char('s') => {
                                cursor = 0;
                                self.show_move(cursor);
                            }
                            KeyCode::Char('e') => {
                                cursor = last_index;
                                self.show_move(cursor);
                            }
                            KeyCode::Esc => return,
                            KeyCode::Char('q') => self.quit(),
                            _ => {}
                        }
                    }
                }
            }
        }
        self.reset();
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

        let dice_1 = Self::roll();
        self.roll_result.push(dice_1);
        let dice_2 = Self::roll();
        self.roll_result.push(dice_2);

        let dice_str = if dice_1 != dice_2 {
            format!("Result: {dice_1}, {dice_2}")
        } else {
            self.roll_result.push(dice_1);
            self.roll_result.push(dice_1);
            format!("Result: {dice_1}, {dice_1}, {dice_1}, {dice_1}")
        };
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
        Self::clear_board();
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
                Self::draw_empty_field(i);
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
        ];
        self.turn = WHITE;
        self.roll_result.clear();
        self.moves.clear();
        self.previous_moves.clear();
        self.bar = [0, 0];
        self.tray = [0, 0];
        self.is_over = false;
        self.is_running = true;
    }

    fn check_is_over(&mut self, loaded: bool) -> bool {
        if self.tray[self.turn as usize] == 15 {
            self.is_over = true;
            let who_won = if self.turn == WHITE { "White" } else { "Black" };
            let message = format!("{who_won} has won! Enter winner's nick:");
            print_message(0, LINE_NUMBER_3, &message);
            let _ = Self::update_leaderboard();
            if !loaded {
                print_message(
                    0,
                    LINE_NUMBER_3,
                    "Do you want to save gameplay to replays folder? y/n",
                );
                clear_line(LINE_NUMBER_4);
                loop {
                    if let Ok(event) = read() {
                        if let Event::Key(key_event) = event {
                            match key_event.code {
                                KeyCode::Char('y') => {
                                    let _ = self.save_previous_moves();
                                    break;
                                }
                                KeyCode::Char('n') => break,
                                _ => {}
                            }
                        }
                    }
                }
            }
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
                            let dice = Self::roll();
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

    fn play(&mut self, loaded: bool) {
        if !loaded {
            self.choose_who_starts();
            self.record_move();
        }
        while self.is_running {
            self.draw();
            print_message(
                0,
                LINE_NUMBER_1,
                "R)oll, S)ave, Q)uit, M)enu (without save)",
            );
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
                                            if !loaded {
                                                self.record_move();
                                            }
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
                                if self.check_is_over(loaded) {
                                    return;
                                }
                            }
                            self.change_turn();
                        }
                        KeyCode::Char('s') => {
                            if let Ok(()) = self.save_to_file() {
                                print_temp_message(
                                    0,
                                    LINE_NUMBER_4,
                                    "Saved game state successfully",
                                    1000,
                                );
                            } else {
                                print_temp_message(
                                    0,
                                    LINE_NUMBER_4,
                                    "Saved game state successfully",
                                    1000,
                                );
                            }
                        }
                        KeyCode::Char('m') => {
                            self.reset();
                            return;
                        }
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
            print_message(
                0,
                LINE_NUMBER_1,
                "P)lay, L)oad, S)how leaderboard, R)eplay, Q)uit",
            );
            if let Ok(event) = read() {
                if let Event::Key(key_event) = event {
                    match key_event.code {
                        KeyCode::Char('p') => self.play(false),
                        KeyCode::Char('l') => {
                            if self.read_from_file().is_ok() {
                                self.play(true);
                            }
                        }
                        KeyCode::Char('s') => {
                            let _ = self.get_leaderboard();
                        }
                        KeyCode::Char('r') => {
                            let _ = self.visualize_replay();
                        }
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
