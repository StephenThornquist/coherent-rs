//! Host a Coherent laser on a network server with a port specified in the command line.
use std::time::Duration;
use coherent_rs::{
    Discovery,
    laser::Laser,
};
#[cfg(feature = "network")]
use coherent_rs::network::NetworkLaserServer;

/// Host a Coherent laser on a network server with a port specified in the command line.
/// 
/// # Usage:
/// 
/// ```shell
/// host_discovery_server COM5
/// ``` 
#[cfg(feature = "network")]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <port>", args[0]);
        std::process::exit(1);
    }
    let port = args[1].parse::<String>().unwrap();
    let laser = Discovery::find_first().unwrap();
    match NetworkLaserServer::<Discovery>::new(
        laser, port.as_str(), Some(0.2),
    ) {
        Ok(mut server) => {
            match server.poll() {
                Ok(_) => {
                    println!("Server started on port {}", port);
                },
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    std::process::exit(1);
                }
            }
            while server.polling() {std::thread::sleep(Duration::from_millis(5));}
            return ();
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(not(feature = "network"))]
fn main() {
    eprintln!("This binary requires the 'network' feature to be enabled.\
        \nPlease recompile with the 'network' feature enabled.\
        \n\nExample: cargo run --features network --bin host_discovery_server 127.0.0.1:907");
    std::process::exit(1);
}