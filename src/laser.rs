//! laser.rs
//! 
//! This module contains the `Laser` trait and associated types for interacting with Coherent lasers.

use serialport;
use crate::{get_all_coherent_devices, CoherentError};

#[allow(non_snake_case)]
pub mod DiscoveryNX;

pub use DiscoveryNX::{Discovery, DiscoveryNXCommands, DiscoveryNXQueries, DiscoveryLaser};

/// The Coherent laser models currently supported by this library.
#[derive(Debug, PartialEq)]
pub enum LaserType {
    DiscoveryNX,
    // ChameleonUltra,
    UnrecognizedDevice,
}

impl From<u16> for LaserType {
    /// Convert a Product ID into a `LaserType`.
    fn from(product_id : u16) -> Self {
        match product_id {
            516 => LaserType::DiscoveryNX,
            _ => LaserType::UnrecognizedDevice,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LaserState {
    Standby,
    On,
}

/// The state of the laser shutter.
/// Can be coerced from `bool` with
/// `Open` being `true` and `Closed` being `false`.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ShutterState{
    Open,
    Closed,
}

impl From<bool> for ShutterState{
    /// Coerce a `bool` into a `ShutterState`.
    /// `true` is `Open` and `false` is `Closed`.
    fn from(state : bool) -> Self {
        if state {
            ShutterState::Open
        } else {
            ShutterState::Closed
        }
    }
}

impl std::ops::Not for ShutterState {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            ShutterState::Open => ShutterState::Closed,
            ShutterState::Closed => ShutterState::Open,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TuningStatus {
    Tuning,
    Ready
}

impl From<bool> for TuningStatus {
    fn from(tuning : bool) -> Self {
        if tuning {
            TuningStatus::Tuning
        } else {
            TuningStatus::Ready
        }
    }
}

impl Into<bool> for TuningStatus {
    fn into(self) -> bool {
        match self {
            TuningStatus::Tuning => true,
            TuningStatus::Ready => false,
        }
    }
}

impl std::ops::Not for TuningStatus {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            TuningStatus::Tuning => TuningStatus::Ready,
            TuningStatus::Ready => TuningStatus::Tuning,
        }
    }
}

pub trait LaserCommand {
    fn to_string(&self) -> String;
}

pub trait Query : LaserCommand {
    type Result;
    fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError>;
}

/// Coherent Lasers operate using two types of commands:
/// * Commands - These are commands that are sent to the laser
/// to change its state or configuration.
/// 
/// * Queries - These are commands that are sent to the laser
/// to request information about its state or configuration.
pub trait Laser: Sized {

    type CommandEnum : LaserCommand;
    // type QueryEnum : LaserCommand;
    // type QueryResult;

    /// Create a new instance of the laser by opening a
    /// serial connection to the specified port. If no port
    /// is specified or no serial number is specified, this will
    /// search for a laser on all available serial ports.
    /// 
    /// # Arguments
    /// 
    /// * `port` - The serial port to connect to. If `None`, the
    /// function will attempt to automatically detect the laser
    /// by scanning all available serial ports. If it cannot find
    /// the laser on a comm port, it will scan for USB devices.
    /// 
    /// * `serial_number` - The serial number of the laser to connect to.
    /// If this is specified and `port` is specified, the serial number will be
    /// checked against the laser connected to the specified port.
    fn new(port : Option<&str>, serial_number : Option<&str>) -> Result<Self, CoherentError>{
        let coherent_devices = get_all_coherent_devices();

        Err(CoherentError::UnrecognizedDevice)

        // Get devices that match the serial number
        // coherent_devices.into_iter().filter(|device| {
        //     match &device.port_type {
        //         serialport::SerialPortType::UsbPort(info) => {
        //             match serial_number {
        //                 Some(serial) => {
        //                     info.serial_number == Some(serial.to_string())
        //                 },
        //                 None => {
        //                     true
        //                 }
        //             }
        //         },
        //         _ => false
        //     }
        // }) // Then filter by the struct's `is_valid_device` method
        // .filter(|device| Self::is_valid_device(device)).map(|device| {
        //     let port = serialport::new(&device.port_name, BAUDRATE)
        //         .data_bits(DATABITS)
        //         .stop_bits(STOPBITS)
        //         .parity(PARITY)
        //         .open().unwrap();
        //     Self::from_port(port)
        // }).next().ok_or(CoherentError::UnrecognizedDevice)
    }

    /// Send a command to the laser directly over the serial port. Maybe I shouldn't expose this in the trait??
    fn send_serial_command(&mut self, command : &str) -> Result<(), CoherentError>;

    /// Specifies from a serial port whether or not the device is a valid
    /// instance of the struct deriving the `Laser` trait.
    fn is_valid_device(serialportinfo : &serialport::SerialPortInfo)->bool;

    /// Create a new instance of the laser from a `SerialPortInfo` object
    /// specifying where to access the laser.
    fn from_port_info(serialportinfo : &serialport::SerialPortInfo) -> Self;
    
    /// Create a new instance of the laser from a port name.
    fn from_port_name(port_name : &str) -> Result<Self, CoherentError> {
        let port_info = serialport::available_ports().unwrap().into_iter().filter(|port| {
            port.port_name == port_name
        }).next().ok_or(CoherentError::UnrecognizedDevice)?;
        Ok(Self::from_port_info(&port_info))
    }

    /// Find the first instance of a laser of the class on any available port.
    fn find_first() -> Result<Self, CoherentError> {
        let port_info = serialport::available_ports().unwrap().into_iter().filter(|port| {
            Self::is_valid_device(port)
        }).next().ok_or(CoherentError::UnrecognizedDevice)?;
        Ok(Self::from_port_info(&port_info))
    }

    /// Send a command to the laser that doesn't expect a response
    fn send_command(&mut self, command : Self::CommandEnum) -> Result<(), CoherentError>{
        let command = command.to_string();
        self.send_serial_command(&command)
    }

    /// Send a query to the laser that expects a response
    fn query<Q : Query>(&mut self, query : Q) -> Result<Q::Result, CoherentError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutter_state_from_bool() {
        assert_eq!(ShutterState::Open, ShutterState::from(true));
        assert_eq!(ShutterState::Closed, ShutterState::from(false));
    }

    #[test]
    fn print_available_ports(){
        let ports = serialport::available_ports().unwrap();
        for port in ports {
            println!("{:?}", port);
            match port.port_type {
                serialport::SerialPortType::UsbPort(info) => {
                    println!("USB Port: {:?}", info);
                    println!("Serial : {:?}" , info.serial_number);

                },
                serialport::SerialPortType::PciPort => {
                    println!("PCI Port");
                },
                serialport::SerialPortType::BluetoothPort => {
                    println!("Bluetooth Port");
                },
                serialport::SerialPortType::Unknown => {
                    println!("Unknown Port");
                },
            }
        }
    }

}
