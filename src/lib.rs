//! Coherent-RS is a `Rust` library to control Coherent brand lasers common
//! in two-photon microscopy experiments. It implements simple serial control
//! of the laser and provides an asynchronous API to read out and control laser
//! parameters.

use serialport;
pub mod laser;
use laser::Laser;
pub use laser::{DiscoveryNX, DiscoveryNXCommands, DiscoveryNXQueries};
pub use laser::Discovery;

const COHERENT_VENDOR_ID : u16 = 3405;

/// The error types that can be returned by the Coherent-RS library.
#[derive(Debug)]
pub enum CoherentError {
    SerialError(serialport::Error),
    WriteError(std::io::Error),
    TimeoutError,
    InvalidArgumentsError(String),
    InvalidResponseError(String),
    LaserUnavailableError,
    UnrecognizedDevice,
}

impl From<serialport::Error> for CoherentError {
    fn from(error : serialport::Error) -> Self {
        CoherentError::SerialError(error)
    }
}

/// Returns a vector of `SerialPortInfo` objects that are made by Coherent Inc.
/// 
/// # Returns
/// 
/// A `Vec` of `SerialPortInfo` objects that are made by Coherent Inc.
/// 
/// # Example
/// 
/// ```rust
/// use coherent_rs::get_all_coherent_devices;
/// let ports = get_all_coherent_devices();
/// for port in ports {
///    println!("{:?}", port);
/// }
/// ```
pub fn get_all_coherent_devices() -> Vec<serialport::SerialPortInfo> {
    serialport::available_ports().unwrap()
        .into_iter()
        .filter(
            |port| match &port.port_type {
                serialport::SerialPortType::UsbPort(info) => {info.vid.clone() == COHERENT_VENDOR_ID},
                _ => false
            }
        )
        .collect()
}

/// Open a serial connection to the Coherent laser.
/// 
/// # Arguments
/// 
/// * `port` - The serial port to connect to.
/// 
/// # Returns
/// 
/// A `Result` containing the laser object if successful, or a `CoherentError` if not.
/// 
/// # Example
/// 
/// ```rust
/// use coherent_rs::{open, Discovery};
/// let discovery = open::<Discovery>("COM5").unwrap();
/// println!("{:?}", discovery);
/// ```
pub fn open<L : Laser>(port : &str) -> Result<L, CoherentError> {
    // Open serial port
    Ok(L::from_port_name(port)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_all_coherent_devices(){
        let ports = get_all_coherent_devices();
        for port in ports {
            println!("{:?}", port);
        }
    }

    #[test]
    fn test_discovery_nx() {
        use super::laser::{Discovery, DiscoveryNXQueries};

        let mut discovery = Discovery::find_first().unwrap();
        println!("{:?}", discovery);

        println!{"Serial : {:?}", discovery.query(
                DiscoveryNXQueries::Serial{}
            ).unwrap()
        }
    }
}