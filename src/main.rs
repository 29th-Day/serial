use std::io::{self, Read, Write};
use std::time::Duration;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    port: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.port.is_none() {
        match serialport::available_ports() {
            Ok(ports) => {
                if ports.is_empty() {
                    println!("No serial ports found.");
                } else {
                    println!("Available ports:");
                    for port in ports {
                        println!("{}", port.port_name);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to list serial ports: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    let port_name = args
        .port
        .as_deref()
        .expect("Port should be provided when not listing");

    let port = serialport::new(port_name, 57600)
        .timeout(Duration::from_millis(10))
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .flow_control(serialport::FlowControl::None)
        .open();

    match port {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 0x1000];
            let mut prev_cr = false;

            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        let mut out: Vec<u8> = Vec::with_capacity(t);

                        for &b in &serial_buf[..t] {
                            if prev_cr {
                                if b == b'\n' {
                                    out.push(b'\n');
                                    prev_cr = false;
                                    continue;
                                } else {
                                    out.push(b'\n');
                                    prev_cr = false;
                                }
                            }

                            if b == b'\r' {
                                prev_cr = true;
                            } else {
                                out.push(b);
                            }
                        }

                        io::stdout().write_all(&out).unwrap();
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            std::process::exit(1);
        }
    }
}
