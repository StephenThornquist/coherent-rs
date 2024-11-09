use serialport;
use rusb;
use crate::CoherentError;

const BAUDRATE : u32 = 19200;
const DATABITS : serialport::DataBits = serialport::DataBits::Eight;
const STOPBITS : serialport::StopBits = serialport::StopBits::One;
const PARITY : serialport::Parity = serialport::Parity::None;

/// The Coherent laser models currently supported by this library.
pub enum LaserType {
    DiscoveryNX,
    // ChameleonUltra,
    UnrecognizedDevice,
}

pub enum LaserState {
    Standby,
    On,
}

/// The state of the laser shutter.
/// Can be coerced from `bool` with
/// `Open` being `true` and `Closed` being `false`.
#[derive(Debug, PartialEq)]
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

/// Coherent Lasers operate using two types of commands:
/// * Commands - These are commands that are sent to the laser
/// to change its state or configuration.
/// 
/// * Queries - These are commands that are sent to the laser
/// to request information about its state or configuration.
pub trait Laser : Sized {

    type CommandEnum;
    type QueryEnum;
    type QueryResult;

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
        
        // If a port is specified, search for it.
        let port_builder = match port {
            // Port is specified
            Some(port) => {
                // Feels like I'm probably using the API wrong...
                let sp = serialport::new(port, BAUDRATE);
                let sp = sp.baud_rate(BAUDRATE);
                let sp = sp.data_bits(DATABITS);
                let sp = sp.stop_bits(STOPBITS);
                let sp = sp.parity(PARITY);
                sp.open()?
            },
            // No port specified, search for serial number
            None => {
                if serial_number.is_none() {
                    return Err(CoherentError::InvalidArgumentsError(
                        "If port is None, serial_number must be provided.".to_string()
                    ));
                }
                return Err(CoherentError::InvalidResponseError);
                
                // let ports = serialport::available_ports()?;
                // for port in ports {
                //     let port = serialport::new(&port.port_name, 9600).open();
                // }
            }
        };
        Err(CoherentError::UnrecognizedDevice)
    }

    fn send_command(&self, command : Self::CommandEnum) -> Result<(), CoherentError>{Ok(())}

    fn query(&self, query : Self::QueryEnum) -> Result<Self::QueryResult, CoherentError>;

}

/// The Coherent laser model Discovery NX.
pub struct DiscoveryNX{
    port : Box<dyn serialport::SerialPort>,
    pub serial_number : String,
}

pub enum DiscoveryLaser {
    VariableWavelength,
    FixedWavelength,
}

pub enum DiscoveryNXCommands {
    Echo(bool), // Sets whether or not the laser will echo commands
    Laser(LaserState), // Set the laser to standby
    Shutter((DiscoveryLaser, ShutterState)), // Open or close the shutter
    FaultClear, // Clear any faults
    AlignmentMode((DiscoveryLaser, bool)), // Set the laser to alignment mode
    Wavelength(f32), // Set the wavelength
    Heartbeat,
    GddCurve(u8), // Set the GDD calibration curve
    GddCurveN(String), // Set the GDD calibration curve by name
    Gdd(f32),
    SetCurveN(String), // Sets name of current calibration curve
}

pub enum DiscoveryNXQueries {
    Echo,
    Laser,
    Shutter(DiscoveryLaser),
    Keyswitch,
    Faults,
    FaultText,
    Tuning,
    AlignmentMode(DiscoveryLaser),
    Status,
    Wavelength,
    Power(DiscoveryLaser),
    GddCurve,
    GddCurveN,
    Gdd,
    Serial,
}

pub enum QueryResult {
    Echo(bool),
    Laser(LaserState),
    Shutter(ShutterState),
    Keyswitch(bool),
    Faults(u8),
    FaultText(String),
    Tuning(u8),
    AlignmentMode(bool),
    Status(u8),
    Wavelength(f32),
    Power(f32),
    GddCurve(u8),
    GddCurveN(String),
    Gdd(f32),
    Serial(String),
}

impl DiscoveryNX {

    fn send_serial_command(&mut self, command : &str) -> Result<(), CoherentError> {
        let command = command.to_string() + "\r\n"; // Need to end with <CR><LF>
        self.port.write(command.as_bytes()).map_err(
            |e| CoherentError::WriteError(e)
        )?;
        Ok(())
    }
}

impl Laser for DiscoveryNX {
    type CommandEnum = DiscoveryNXCommands;
    type QueryEnum = DiscoveryNXQueries;  
    type QueryResult = QueryResult;

    fn query(&self, query : Self::QueryEnum) -> Result<Self::QueryResult, CoherentError> {
        Err(CoherentError::UnrecognizedDevice)
    }
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

    #[test]
    fn print_available_usbs(){
        for device in rusb::devices().unwrap().iter() {
            let device_desc = device.device_descriptor().unwrap();
            let handle = device.open().unwrap();
            println!(
                "Device: Vendor ID: {:?} Product ID: {:?} Len: {:?} Address: {:?}, ManStrAscii {:?} ProdStrAscii {:?} SerialStrAscii {:?}",
                device_desc.vendor_id(),
                device_desc.product_id(),
                device_desc.length(),
                device_desc.serial_number_string_index(),
                handle.read_manufacturer_string_ascii(&device_desc).unwrap(),
                handle.read_product_string_ascii(&device_desc).unwrap(),
                handle.read_serial_number_string_ascii(&device_desc).unwrap(),
            );
        }
    }
}
