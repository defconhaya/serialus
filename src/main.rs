use crossterm::cursor::MoveTo;
use crossterm::event::{self, poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::QueueableCommand;
use std::io::{stdout, Bytes, Stdout, Write};
use std::ops::{Mul, Sub};
use std::thread;
use std::time::Duration;

struct Rect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

fn chat_window(stdout: &mut impl Write, chat: &[String], boundary: Rect) {
    let n = chat.len();
    let m = n.checked_sub(boundary.h as usize).unwrap_or(0);
    for (row, line) in chat.iter().skip(m).enumerate() {
        stdout
            .queue(MoveTo(boundary.x as u16, (boundary.y + row) as u16))
            .unwrap();
        let bytes = line.as_bytes();
        stdout
            .write(bytes.get(0..boundary.w).unwrap_or(bytes))
            .unwrap();
    }
}

// use serial2::SerialPort;
// use std::env;

// fn chat_window(stdout: &mut impl QueueableCommand)

fn main() {
    let mut stdout = stdout();
    terminal::enable_raw_mode().unwrap();
    let (mut w, mut h) = terminal::size().unwrap();
    let bar_char = "─";
    let mut bar = bar_char.repeat(w as usize);
    let mut prompt = String::new();
    let mut chat = Vec::new();
    let mut quit = false;
    while !quit {
        while poll(Duration::ZERO).unwrap() {
            match read().unwrap() {
                Event::Resize(nw, nh) => {
                    w = nw;
                    h = nh;
                    bar = bar_char.repeat(w as usize);
                }
                Event::Paste(data) => {
                    prompt.push_str(&data);
                }

                Event::Key(event) => match (event.kind, event.code) {
                    (KeyEventKind::Release, KeyCode::Char(x)) => {
                        if x == 'c' && event.modifiers.contains(KeyModifiers::CONTROL) {
                            quit = true;
                        }
                        prompt.push(x)
                    }
                    (KeyEventKind::Release, KeyCode::Esc) => quit = true,
                    (KeyEventKind::Release, KeyCode::Enter) => {
                        chat.push(prompt.clone());
                        prompt.clear();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        stdout.queue(Clear(ClearType::All)).unwrap();
        chat_window(
            &mut stdout,
            &chat,
            Rect {
                x: 0,
                y: 0,
                w: w as usize,
                h: h as usize - 2,
            },
        );

        stdout.queue(MoveTo(0, h - 2)).unwrap();
        stdout.write(bar.as_bytes()).unwrap();
        stdout.queue(MoveTo(1, h - 1)).unwrap();
        {
            let bytes = prompt.as_bytes();
            stdout
                .write(bytes.get(0..w as usize).unwrap_or(bytes))
                .unwrap();
        }        
        stdout.flush();
        thread::sleep(Duration::from_millis(33));
        terminal::disable_raw_mode().unwrap();
    }
}

// fn test_ser() -> Result<(), ()> {
//     println!("OS is {}", env::consts::OS); // Prints the current OS.
//     let result = get_all_ports();
//     println!("{:?}", result);
//     let port = open_port(String::from("COM13"), 115200).unwrap();
//     let mut buffer = [0; 512];
//     loop {
//         match port.read(&mut buffer) {
//             Ok(0) => return Ok(()),
//             Ok(n) => {
//                 std::io::stdout()
//                     .write_all(&buffer[..n])
//                     .map_err(|e| eprintln!("Error: Failed to write to stdout: {}", e))?;
//             }
//             Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
//             Err(e) => {
//                 eprintln!("Error: Failed to read from port: {}", e);
//                 return Err(());
//             }
//         }
//     }
// }

fn get_all_ports() -> Vec<String> {
    let mut all_ports = Vec::new();
    match serial2::SerialPort::available_ports() {
        Err(e) => {
            eprintln!("Failed to enumerate serial ports:  --> {}  ", e);
            std::process::exit(1);
        }
        Ok(ports) => {
            eprintln!("Found {} ports", ports.len());
            for port in ports {
                all_ports.push(String::from(port.to_str().unwrap()));
                // println!("{}", port.display())
            }
        }
    }
    all_ports
}

// fn open_port(port_name: String, baud: u32) -> Result<SerialPort, String> {
//     let port = SerialPort::open(&port_name, baud)
//         .map_err(|e| format!("Error: Failed to open {}: {} ", port_name, e))?;
//     Ok(port)
// }
