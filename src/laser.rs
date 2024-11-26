//! laser.rs
//! 
//! This module contains the `Laser` trait and associated types for interacting with Coherent lasers.

use serialport;
use crate::CoherentError;

#[cfg(feature = "network")]
use serde::{Serialize, Deserialize};

pub mod discoverynx;
pub mod debug;

pub use discoverynx::{Discovery, DiscoveryNXCommands, DiscoveryNXQueries, DiscoveryLaser};

#[cfg_attr(feature = "network", derive(Serialize, Deserialize))]
/// The Coherent laser models currently supported by this library.
#[derive(Debug, PartialEq, Clone)]
pub enum LaserType {
    DiscoveryNX,
    // ChameleonUltra,
    DebugLaser, // For testing purposes -- behaves like a laser.
    UnrecognizedDevice,
}

impl From<u16> for LaserType {
    /// Convert a Product ID into a `LaserType`.
    fn from(product_id : u16) -> Self {
        match product_id {
            0 => LaserType::DebugLaser,
            516 => LaserType::DiscoveryNX,
            _ => LaserType::UnrecognizedDevice,
        }
    }
}

#[cfg_attr(feature = "network", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LaserState {
    Standby,
    On,
}

/// The state of the laser shutter.
/// Can be coerced from `bool` with
/// `Open` being `true` and `Closed` being `false`.
#[cfg_attr(feature = "network", derive(Serialize, Deserialize))]
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

#[cfg_attr(feature = "network", derive(Serialize, Deserialize))]
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

pub trait LaserCommand : Sized {
    fn to_string(&self) -> String;
}

#[cfg(feature = "network")]
pub trait Query : LaserCommand + Deserialize<'static> + Serialize {
    type Result;
    fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError>;
}


#[cfg(not(feature = "network"))]
pub trait Query : LaserCommand{
    type Result;
    fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError>;
}

/// Coherent Lasers operate using two types of commands:
/// * Commands - These are commands that are sent to the laser
/// to change its state or configuration.
/// 
/// * Queries - These are commands that are sent to the laser
/// to request information about its state or configuration.
/// 
/// Lasers implement `Send` to allow for threading and 
/// Network interfaces.
pub trait Laser: Into<LaserType> + Send {

    #[cfg(feature = "network")]
    type CommandEnum : LaserCommand + Serialize + Deserialize<'static> + core::fmt::Debug;

    #[cfg(not(feature = "network"))]
    type CommandEnum : LaserCommand + core::fmt::Debug;

    #[cfg(feature = "network")]
    type LaserStatus: Serialize + Deserialize<'static> + core::fmt::Debug; // for status communication over serial

    /// Create a new instance of the laser by opening a
    /// serial connection to the specified port. If no port
    /// is specified and no serial number is specified, this will
    /// search for a laser on all available serial ports. This
    /// method is intended to find a laser if there's one available,
    /// provided it matches the requests.
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
    /// 
    /// # Returns
    /// 
    /// A `Result` containing the laser object if successful, or a `CoherentError` if not.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use coherent_rs::{Discovery};
    /// 
    /// // Open a specific port, but it doesn't exist
    /// let discovery = Discovery::new(Some("NotAPort"), None);
    /// assert!(discovery.is_err());
    /// 
    /// // Open a specific port that exists
    /// let discovery = Discovery::new(Some("COM5"), None).unwrap();
    /// 
    /// // Open the first available laser
    /// let discovery = Discovery::new(None, None).unwrap();
    /// 
    /// // Open a specific laser by serial number
    /// let discovery = Discovery::new(None, Some("123456")).unwrap();
    /// 
    /// // Open a specific laser by serial number on a specific port
    /// let discovery = Discovery::new(Some("COM5"), Some("123456")).unwrap();
    /// ```
    fn new(port_name : Option<&str>, serial_number : Option<&str>) -> Result<Self, CoherentError>{

        // Nested cases
        match port_name {
            Some(port_name) => {
                let port_info = serialport::available_ports().unwrap().into_iter().filter(|port| {
                    port.port_name == port_name
                }).next().ok_or(CoherentError::UnrecognizedDevice)?;
                
                match serial_number {
                    Some(serial) => {
                        match &port_info.port_type {
                            serialport::SerialPortType::UsbPort(info) => {
                                if info.serial_number == Some(serial.to_string()) {
                                    Self::from_port_info(&port_info)
                                } else {
                                    Err(CoherentError::UnrecognizedDevice)
                                }
                            },
                            _ => Err(CoherentError::UnrecognizedDevice)
                        }
                    },
                    None => Self::from_port_info(&port_info)
                }
            }
            None => {
                match serial_number {
                    Some(serial) => {
                        let port_info = serialport::available_ports().unwrap().into_iter().filter(|port| {
                            Self::is_valid_device(port)
                        }).next().ok_or(CoherentError::UnrecognizedDevice)?;
                        match &port_info.port_type {
                            serialport::SerialPortType::UsbPort(info) => {
                                if info.serial_number == Some(serial.to_string()) {
                                    Self::from_port_info(&port_info)
                                } else {
                                    Err(CoherentError::UnrecognizedDevice)
                                }
                            },
                            _ => Err(CoherentError::UnrecognizedDevice)
                        }
                    },
                    None => Self::find_first()
                }
            }
        }
    }

    /// Send a command to the laser directly over the serial port. Maybe I shouldn't expose this in the trait??
    /// But this is probably a good emergency tool to expose... I don't know. TBD
    fn send_serial_command(&mut self, command : &str) -> Result<(), CoherentError>;

    /// Specifies from a serial port whether or not the device is a valid
    /// instance of the struct deriving the `Laser` trait.
    fn is_valid_device(serialportinfo : &serialport::SerialPortInfo)->bool;

    /// Create a new instance of the laser from a `SerialPortInfo` object
    /// specifying where to access the laser.
    fn from_port_info(serialportinfo : &serialport::SerialPortInfo) -> Result<Self, CoherentError>;
    
    /// Create a new instance of the laser from a port name.
    fn from_port_name(port_name : &str) -> Result<Self, CoherentError> {
        let port_info = serialport::available_ports().unwrap().into_iter().filter(|port| {
            port.port_name == port_name
        }).next().ok_or(CoherentError::UnrecognizedDevice)?;
        Self::from_port_info(&port_info)
    }

    /// Find the first instance of a laser of the class on any available port.
    fn find_first() -> Result<Self, CoherentError> {
        let port_info = serialport::available_ports().unwrap().into_iter().filter(|port| {
            Self::is_valid_device(port)
        }).next().ok_or(CoherentError::NoRecognizedLasers)?;
        Self::from_port_info(&port_info)
    }

    /// Send a command to the laser that doesn't expect a response
    fn send_command(&mut self, command : Self::CommandEnum) -> Result<(), CoherentError>{
        let command = command.to_string();
        self.send_serial_command(&command)
    }

    /// Send a query to the laser that expects a response
    fn query<Q : Query>(&mut self, query : Q) -> Result<Q::Result, CoherentError>;

    /// Returns a struct containing the current status of the laser
    fn status(&mut self) -> Result<Self::LaserStatus, CoherentError>;
    
    /// Executes all of the desired queries and returns them
    /// in a serialized format. Only needed for network-compatible
    /// implementations
    #[cfg(feature = "network")]
    fn serialized_status(&mut self) -> Result<Vec<u8>, CoherentError>;

    fn into_laser_type() -> LaserType;
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

    #[cfg(feature = "network")]
    #[test]
    fn test_serde_laser_type(){
        use rmp_serde::Serializer;

        let laser_type = LaserType::DiscoveryNX;
        let mut buf = Vec::new();
        laser_type.serialize(&mut Serializer::new(&mut buf)).unwrap();

        let laser_type_deserialized = LaserType::deserialize(
           &mut rmp_serde::Deserializer::new(&buf[..])
        ).unwrap();
        assert_eq!(laser_type, laser_type_deserialized);
    }

}
