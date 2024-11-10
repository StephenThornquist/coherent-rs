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


/// C ABI
#[no_mangle]
pub unsafe extern "C" fn discovery_find_first() -> *mut Discovery {
    Box::into_raw(Box::new(Discovery::find_first().unwrap()))
}

#[no_mangle]
pub unsafe extern "C" fn free_discovery(laser : *mut Discovery) {
    if laser.is_null() {return}
    drop(Box::from_raw(laser)); // drop is for clarity
}

#[no_mangle]
pub unsafe extern "C" fn discovery_by_port_name(port_name : *const u8, port_name_len : usize) -> *mut Discovery {
    let port_name = unsafe {
        std::str::from_utf8(std::slice::from_raw_parts(port_name, port_name_len)).unwrap()
    };
    Box::into_raw(Box::new(Discovery::from_port_name(port_name).unwrap()))
}

#[no_mangle]
pub unsafe extern "C" fn discovery_by_serial_number(serial_number : *const u8, serial_number_len : usize) -> *mut Discovery {
    let serial_number = unsafe {
        std::str::from_utf8(std::slice::from_raw_parts(serial_number, serial_number_len)).unwrap()
    };
    Box::into_raw(Box::new(Discovery::new(None, Some(serial_number)).unwrap()))
}

#[no_mangle]
pub extern "C" fn discovery_set_wavelength(discovery : *mut Discovery, wavelength : f32) {
    unsafe {discovery.as_mut().unwrap().set_wavelength(wavelength).unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_get_wavelength(discovery : *mut Discovery) -> f32 {
    unsafe {(*discovery).get_wavelength().unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_get_power_variable(discovery : *mut Discovery) -> f32 {
    unsafe {(*discovery).get_power(laser::DiscoveryLaser::VariableWavelength).unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_get_power_fixed(discovery : *mut Discovery) -> f32 {
    unsafe {(*discovery).get_power(laser::DiscoveryLaser::FixedWavelength).unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_set_gdd(discovery : *mut Discovery, gdd : f32) {
    unsafe {(*discovery).set_gdd(gdd).unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_get_gdd(discovery : *mut Discovery) -> f32 {
    unsafe {(*discovery).get_gdd().unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_set_alignment_variable(discovery : *mut Discovery, alignment : bool) {
    unsafe {(*discovery).set_alignment_mode(laser::DiscoveryLaser::VariableWavelength, alignment).unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_get_alignment_variable(discovery : *mut Discovery) -> bool {
    unsafe {(*discovery).get_alignment_mode(laser::DiscoveryLaser::VariableWavelength).unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_set_alignment_fixed(discovery : *mut Discovery, alignment : bool) {
    unsafe {(*discovery).set_alignment_mode(laser::DiscoveryLaser::FixedWavelength, alignment).unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_get_alignment_fixed(discovery : *mut Discovery) -> bool {
    unsafe {(*discovery).get_alignment_mode(laser::DiscoveryLaser::FixedWavelength).unwrap()}
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
        println!{"Returned : {:?}", not_discovery}

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
        use super::laser::{Discovery, DiscoveryNXQueries, DiscoveryNXCommands, DiscoveryLaser};

        let not_discovery = open::<Discovery>("NotAPort");
        assert!(not_discovery.is_err());
        println!{"Returned : {:?}", not_discovery}

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