//! Forces a `Server` at a port to forget its primary client
//! 
use coherent_rs::Discovery;
#[cfg(feature = "network")]
use coherent_rs::network::{NetworkLaserClient,BasicNetworkLaserClient};

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
    let client = BasicNetworkLaserClient::<Discovery>::connect(port.as_str());
    match client {
        Ok(mut client) => {
            println!("Client connected to port {}", port);
            match client.force_forget_primary_client() {
                Ok(_) => {
                    println!("Primary client forgotten");
                },
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    std::process::exit(1);
                }
            }
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
        \n\nExample: cargo run --features network --bin listen_and_print_discovery 127.0.0.1:907");
    std::process::exit(1);
}