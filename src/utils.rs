use std::io::{Write, stdout};
use crossterm::{
    cursor::MoveTo,
    QueueableCommand,
    queue,
    terminal::{Clear, ClearType},
};

pub fn move_cursor(x: u16, y: u16) {
    stdout()
        .queue(MoveTo(x, y)).unwrap()
        .flush().unwrap();
}

pub fn clear_screen() {
    stdout()
        .queue(Clear(ClearType::All)).unwrap()
        .flush().unwrap();
}

pub fn clear_line() {
    stdout()
        .queue(Clear(ClearType::CurrentLine)).unwrap()
        .flush().unwrap();
}
