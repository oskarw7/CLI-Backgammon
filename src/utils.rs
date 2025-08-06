use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    terminal::{Clear, ClearType},
};
use std::io::{stdout, Write};
use std::{thread, time::Duration};

pub fn move_cursor(x: u16, y: u16) {
    stdout().queue(MoveTo(x, y)).unwrap().flush().unwrap();
}

pub fn clear_screen() {
    stdout()
        .queue(Clear(ClearType::All))
        .unwrap()
        .flush()
        .unwrap();
}

pub fn clear_line(line_number: u16) {
    move_cursor(0, line_number);
    stdout()
        .queue(Clear(ClearType::CurrentLine))
        .unwrap()
        .flush()
        .unwrap();
}

pub fn print_message(x: u16, y: u16, message: &str) {
    clear_line(y);
    move_cursor(x, y);
    print!("{message}");
    stdout().flush().unwrap();
}

pub fn print_temp_message(x: u16, y: u16, message: &str, time_millis: u64) {
    print_message(x, y, message); 
    thread::sleep(Duration::from_millis(time_millis));
    clear_line(y);
}
