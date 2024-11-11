//! Coherent-RS is a `Rust` library to control Coherent brand lasers common
//! in two-photon microscopy experiments. It implements simple serial control
//! of the laser and provides an asynchronous API to read out and control laser
//! parameters.

use serialport;
pub mod laser;
use laser::Laser;
pub use laser::{discoverynx, DiscoveryNXCommands, DiscoveryNXQueries};
pub use laser::Discovery;

const COHERENT_VENDOR_ID : u16 = 3405;

/// The error types that can be returned by the Coherent-RS library.
#[derive(Debug)]
pub enum CoherentError {
    SerialError(serialport::Error),
    WriteError(std::io::Error),
    TimeoutError,
    CommandNotExecutedError,
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
/// let discovery = open::<Discovery>("NotAPort");
/// assert!(discovery.is_err());
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
        use super::laser::{Discovery, DiscoveryNXQueries, DiscoveryNXCommands, DiscoveryLaser};

        let not_discovery = open::<Discovery>("NotAPort");
        assert!(not_discovery.is_err());
        println!{"Returned : {:?} (supposed to be unrecognized)", not_discovery}

        let discovery = Discovery::find_first();
        assert!(discovery.is_ok());
        let mut discovery = discovery.unwrap();
        println!("{:?}", discovery);

        println!{"Serial : {:?}", discovery.query(
                DiscoveryNXQueries::Serial{}
            ).unwrap()
        }

        let fixed_wavelength_power = discovery.query(
            DiscoveryNXQueries::Power{laser : laser::DiscoveryLaser::FixedWavelength}
        );
        assert!(fixed_wavelength_power.is_ok());

        println!{"Fixed wavelength beam power : {:?}", fixed_wavelength_power.unwrap()}

        discovery.send_command(
            DiscoveryNXCommands::Shutter(
                (DiscoveryLaser::FixedWavelength, laser::ShutterState::Open)
            )
        ).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(300));

        discovery.send_command(
            DiscoveryNXCommands::Shutter(
                (DiscoveryLaser::FixedWavelength, laser::ShutterState::Closed)
            )
        ).unwrap();
    }

    #[test]
    fn test_discovery_nx_convenience() {
        use super::laser::Discovery;

        let not_discovery = open::<Discovery>("NotAPort");
        assert!(not_discovery.is_err());
        println!{"Returned : {:?} (supposed to be unrecognized)", not_discovery}

        let discovery = Discovery::find_first();
        assert!(discovery.is_ok());
        let mut discovery = discovery.unwrap();
        println!("{:?}", discovery);

        println!{"Serial : {:?}", discovery.get_serial().unwrap()};

        let fixed_wavelength_power = discovery.get_power(laser::DiscoveryLaser::FixedWavelength);
        assert!(fixed_wavelength_power.is_ok());

        println!{"Fixed wavelength beam power : {:?}", fixed_wavelength_power.unwrap()}

        discovery.set_shutter(laser::DiscoveryLaser::FixedWavelength,
            laser::ShutterState::Open).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(300));

        discovery.set_shutter(laser::DiscoveryLaser::FixedWavelength,
             laser::ShutterState::Closed).unwrap();
    }
}