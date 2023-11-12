use serial2::SerialPort;
use std::env;
use std::io::Write;

fn main() -> Result<(), ()> {
    println!("OS is {}", env::consts::OS); // Prints the current OS.
    let result = get_all_ports();
    println!("{:?}", result);
    let port = open_port(String::from("COM13"), 115200).unwrap();
    let mut buffer = [0; 512];
    loop {
        match port.read(&mut buffer) {
            Ok(0) => return Ok(()),
            Ok(n) => {
                std::io::stdout()
                    .write_all(&buffer[..n])
                    .map_err(|e| eprintln!("Error: Failed to write to stdout: {}", e))?;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
            Err(e) => {
                eprintln!("Error: Failed to read from port: {}", e);
                return Err(());
            }
        }
    }
}

fn get_all_ports() -> Vec<String> {
    let mut all_ports = Vec::new();
    match serial2::SerialPort::available_ports() {
        Err(e) => {
            eprintln!("Failed to enumerate serial ports: {}  ", e);
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

fn open_port(port_name: String, baud: u32) -> Result<SerialPort, String> {
    let port = SerialPort::open(&port_name, baud)
        .map_err(|e| format!("Error: Failed to open {}: {} ", port_name, e))?;
    Ok(port)
}
