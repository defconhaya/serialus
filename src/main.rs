use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::QueueableCommand;
use serial2::SerialPort;
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct Rect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

fn draw_window(stdout: &mut impl Write, boundary: &Rect) {
    let bar_char = "═";
    let corner_down_right = "╔";
    let corner_down_left = "╗";
    let corner_up_right = "╚";
    let corner_up_left = "╝";
    let vertical_right = "╠";
    let vertical_left = "╣";
    let upper_bar = format!(
        "{}{}{}",
        corner_down_right,
        bar_char.repeat(boundary.w as usize - 2),
        corner_down_left
    );
    let lower_bar = format!(
        "{}{}{}",
        corner_up_right,
        bar_char.repeat(boundary.w as usize - 2),
        corner_up_left
    );
    let middle_bar = format!(
        "{}{}{}",
        vertical_right,
        bar_char.repeat(boundary.w as usize - 2),
        vertical_left
    );
    stdout.queue(MoveTo(0, 0)).unwrap();
    stdout.write(upper_bar.as_bytes()).unwrap();
    stdout.queue(MoveTo(0, boundary.h as u16 - 1)).unwrap();
    stdout.write(middle_bar.as_bytes()).unwrap();
    stdout.queue(MoveTo(0, boundary.h as u16 + 1)).unwrap();
    stdout.write(lower_bar.as_bytes()).unwrap();
    for y in 1..boundary.h - 1 {
        stdout.queue(MoveTo(0, y as u16)).unwrap();
        stdout.write("║".as_bytes()).unwrap();
        stdout.queue(MoveTo(boundary.w as u16, y as u16)).unwrap();
        stdout.write("║".as_bytes()).unwrap();
    }

    // stdout.flush();
}
fn draw_chat_in_window(stdout: &mut impl Write, chat: &[String], boundary: Rect) {
    let n = chat.len();
    let m = n.checked_sub(boundary.h as usize - 2).unwrap_or(0);
    draw_window(stdout, &boundary);
    for (row, line) in chat.iter().skip(m).enumerate() {
        stdout
            .queue(MoveTo(boundary.x as u16 + 1, (boundary.y + row + 1) as u16))
            .unwrap();
        let bytes = line.as_bytes();
        stdout
            .write(bytes.get(0..boundary.w).unwrap_or(bytes))
            .unwrap();
    }
}

fn main() {
    terminal::enable_raw_mode().unwrap();
    let ( mut w,mut  h) = terminal::size().unwrap();
    // let bar_char = "═";
    // let mut prompt = String::new();
    let prompt: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let keyboard_prompt_shared = Arc::clone(&prompt);
    let gui_prompt_shared = Arc::clone(&prompt);

    let chat: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let serial_port_chat_shared = Arc::clone(&chat);
    let keyboard_chat_shared = Arc::clone(&chat);
    let gui_chat_shared = Arc::clone(&chat);

    // let mut quit = false;
    let quit: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let quit_keyboard = Arc::clone(&quit);
    let quit_gui = Arc::clone(&quit);
    let quit_serial = Arc::clone(&quit);

    let serial_handle = std::thread::spawn(move || {
        let port = SerialPort::open("COM13", 115200).unwrap();
        let mut buffer = [0; 512];
        while !*quit_serial.lock().unwrap() {
            match port.read(&mut buffer) {
                Ok(0) => {}
                Ok(n) => {
                    let mut chat = serial_port_chat_shared.lock().unwrap();
                    let msg = String::from_utf8(buffer[..n].to_vec()).unwrap();
                    chat.push(msg);
                }
                // Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => return,
                Err(e) => {
                    eprintln!("Error: Failed to read from port: {}", e);
                    // return Err(());
                }
            }
        }
    });

    let keyboard_handle = std::thread::spawn(move || {
        while !*quit_keyboard.lock().unwrap() {
            while poll(Duration::ZERO).unwrap() {
                match read().unwrap() {
                    Event::Resize(nw, nh) => {
                        w = nw;
                        h = nh;
                    }
                    Event::Paste(data) => {
                        let mut prompt = keyboard_prompt_shared.lock().unwrap();
                        prompt.push_str(&data);
                    }

                    Event::Key(event) => match (event.kind, event.code) {
                        (KeyEventKind::Release, KeyCode::Char(x)) => {
                            if x == 'c' && event.modifiers.contains(KeyModifiers::CONTROL) {
                                *quit_keyboard.lock().unwrap() = true;
                            }
                            let mut prompt = keyboard_prompt_shared.lock().unwrap();
                            prompt.push(x);
                        }
                        (KeyEventKind::Release, KeyCode::Esc) => {
                            *quit_keyboard.lock().unwrap() = true
                        }
                        (KeyEventKind::Release, KeyCode::Enter) => {
                            let mut chat = keyboard_chat_shared.lock().unwrap();
                            chat.push(keyboard_prompt_shared.lock().unwrap().clone());
                            keyboard_prompt_shared.lock().unwrap().clear();
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            terminal::disable_raw_mode().unwrap();
        }
    });

    let gui_handle = std::thread::spawn(move || {
        while !*quit_gui.lock().unwrap() {
            let mut stdout = stdout();
            stdout.queue(Clear(ClearType::All)).unwrap();
            draw_chat_in_window(
                &mut stdout,
                &gui_chat_shared.lock().unwrap(),
                Rect {
                    x: 0,
                    y: 0,
                    w: w as usize,
                    h: h as usize - 2,
                },
            );

            stdout.queue(MoveTo(0, h - 2)).unwrap();
            stdout.write("║".as_bytes()).unwrap();
            stdout.queue(MoveTo(w, h - 2)).unwrap();
            stdout.write("║".as_bytes()).unwrap();
            stdout.queue(MoveTo(2, h - 2)).unwrap();

            let prompt = gui_prompt_shared.lock().unwrap();
            let bytes = prompt.as_bytes();
            stdout
                .write(bytes.get(0..w as usize).unwrap_or(bytes))
                .unwrap();

            stdout.queue(MoveTo(w, h - 2)).unwrap();
            stdout.write("║".as_bytes()).unwrap();
            stdout.flush().unwrap();

            thread::sleep(Duration::from_millis(33));
        }
    });

    serial_handle.join().unwrap();
    keyboard_handle.join().unwrap();
    gui_handle.join().unwrap();
}
