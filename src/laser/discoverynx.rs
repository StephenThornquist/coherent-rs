//! discoverynx.rs
//! 
//! DiscoveryNX laser model implementation.

use std::io::{Write, BufRead};

use crate::{CoherentError, Laser};
use crate::laser::{LaserCommand, Query, LaserState, ShutterState, LaserType, TuningStatus};

const BAUDRATE : u32 = 19200;
const DATABITS : serialport::DataBits = serialport::DataBits::Eight;
const STOPBITS : serialport::StopBits = serialport::StopBits::One;
const PARITY : serialport::Parity = serialport::Parity::None;


/// The Coherent laser model Discovery NX.
#[derive(Debug)]
#[repr(C)]
pub struct Discovery{
    pub port : Box<dyn serialport::SerialPort>,
    pub serial_number : String,
    echo : bool, // whether or not the laser will echo commands, which affects parsing
    _prompt : bool, // whether or not the laser will echo prompts, which affects parsing
}

#[derive(Debug)]
pub enum DiscoveryLaser {
    VariableWavelength,
    FixedWavelength,
}

/// Commands to change parameters of the DiscoveryNX
pub enum DiscoveryNXCommands {
    Echo{echo_on : bool}, // Sets whether or not the laser will echo commands
    Laser{state : LaserState}, // Set the laser to standby
    Shutter{laser : DiscoveryLaser, state: ShutterState}, // Open or close the shutter
    FaultClear, // Clear any faults
    AlignmentMode{laser : DiscoveryLaser, alignment_mode_on : bool}, // Set the laser to alignment mode
    Wavelength{wavelength_nm : f32}, // Set the wavelength
    Heartbeat,
    GddCurve{curve_num : u8}, // Set the GDD calibration curve
    GddCurveN{curve_name : String}, // Set the GDD calibration curve by name
    Gdd{gdd_val : f32},
    SetCurveN{new_curve_name : String}, // Sets name of current calibration curve
}

impl LaserCommand for DiscoveryNXCommands {
    fn to_string(&self) -> String {
        match &self {
            DiscoveryNXCommands::Echo{echo_on : echo} => format!("E={}", if *echo {"1"} else {"0"}),
            DiscoveryNXCommands::Laser{state} => format!("L={}", match state {
                LaserState::Standby => "0",
                LaserState::On => "1",
            }),
            DiscoveryNXCommands::FaultClear => String::from("FC"),
            DiscoveryNXCommands::AlignmentMode{laser : laser, alignment_mode_on : mode} => match laser {
                DiscoveryLaser::VariableWavelength => format!("ALIGN={}", if *mode {"1"} else {"0"}),
                DiscoveryLaser::FixedWavelength => format!("ALIGNFIXED={}", if *mode {"1"} else {"0"}),
            },
            DiscoveryNXCommands::Shutter{laser : laser, state : state} => match laser {
                DiscoveryLaser::VariableWavelength => format!("S={}", if *state == ShutterState::Open {"1"} else {"0"}),
                DiscoveryLaser::FixedWavelength => format!("SFIXED={}", if *state == ShutterState::Open {"1"} else {"0"}),
            },
            DiscoveryNXCommands::Wavelength{wavelength_nm : wavelength} => format!("WV={}", wavelength),
            DiscoveryNXCommands::Heartbeat => String::from("HB"),
            DiscoveryNXCommands::GddCurve{curve_num : curve} => format!("GDD={}", curve),
            DiscoveryNXCommands::GddCurveN{curve_name : name} => format!("GDDCURVEN={}", name),
            DiscoveryNXCommands::Gdd{gdd_val : gdd} => format!("GDD={}", gdd),
            DiscoveryNXCommands::SetCurveN{new_curve_name : name} => format!("SETCURVEN={}", name),
        }
    }
}

#[allow(non_snake_case)]
pub mod DiscoveryNXQueries {
    use super::*;

    #[derive(Default)]
    pub struct Echo {}
    impl LaserCommand for Echo {
        fn to_string(&self) -> String {
            String::from("?E")
        }
    }
    impl Query for Echo {
        type Result = bool;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            Ok(result.contains("1"))
        }
    }

    #[derive(Default)]
    pub struct Laser {}
    impl LaserCommand for Laser {
        fn to_string(&self) -> String {
            String::from("?L")
        }
    }
    impl Query for Laser {
        type Result = LaserState;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            match result {
                "0" => Ok(LaserState::Standby),
                "1" => Ok(LaserState::On),
                _ => Err(CoherentError::InvalidResponseError(result.to_string())),
            }
        }
    }


    /// Setting the shutter takes time -- recommended to sleep for ~300 ms after setting
    pub struct Shutter {
        pub laser : DiscoveryLaser,
    }
    impl LaserCommand for Shutter {
        fn to_string(&self) -> String {
            match self.laser {
                DiscoveryLaser::VariableWavelength => String::from("?S"),
                DiscoveryLaser::FixedWavelength => String::from("?SFIXED"),
            }
        }
    }

    /// Setting the shutter takes time -- recommended to sleep for ~300 ms after setting
    impl Query for Shutter {
        type Result = ShutterState;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            match result {
                "0" => Ok(ShutterState::Closed),
                "1" => Ok(ShutterState::Open),
                _ => Err(CoherentError::InvalidResponseError(result.to_string())),
            }
        }
    }

    pub struct Keyswitch {}
    impl LaserCommand for Keyswitch {
        fn to_string(&self) -> String {
            String::from("?K")
        }
    }
    impl Query for Keyswitch {
        type Result = bool;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            Ok(result.contains("1"))
        }
    }

    pub struct Faults {}
    impl LaserCommand for Faults {
        fn to_string(&self) -> String {
            String::from("?F")
        }
    }
    impl Query for Faults {
        type Result = u8;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            Ok(result.parse().unwrap())
        }
    }

    pub struct FaultText {}
    impl LaserCommand for FaultText {
        fn to_string(&self) -> String {
            String::from("?FT")
        }
    }
    impl Query for FaultText {
        type Result = String;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            Ok(result.to_string())
        }
    }

    pub struct Tuning {}
    impl LaserCommand for Tuning {
        fn to_string(&self) -> String {
            String::from("?TS")
        }
    }
    impl Query for Tuning {
        type Result = TuningStatus;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            match result {
                "0" => Ok(TuningStatus::Ready),
                "1" => Ok(TuningStatus::Tuning),
                _ => Err(CoherentError::InvalidResponseError(result.to_string())),
            }
        }
    }

    pub struct AlignmentMode {
        pub laser : DiscoveryLaser,
    }
    impl LaserCommand for AlignmentMode {
        fn to_string(&self) -> String {
            match self.laser {
                DiscoveryLaser::VariableWavelength => String::from("?ALIGNVAR"),
                DiscoveryLaser::FixedWavelength => String::from("?ALIGNFIXED"),
            }
        }
    }
    impl Query for AlignmentMode {
        type Result = bool;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            Ok(result.contains("1"))
        }
    }

    pub struct Status {}
    impl LaserCommand for Status {
        fn to_string(&self) -> String {
            String::from("?ST")
        }
    }
    impl Query for Status {
        type Result = String;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            Ok(result.to_string())
        }
    }

    /// Setting the wavelength takes time -- laser will begin
    /// tuning to the new wavelength. Recommended to use a 
    /// `while laser.query(Tuning{}) {std::thread::sleep(std::time::Duration::from_millis(100));}` loop
    /// or setting other parameters while it's happening
    pub struct Wavelength {}
    impl LaserCommand for Wavelength {
        fn to_string(&self) -> String {
            String::from("?WV")
        }
    }

    /// Setting the wavelength takes time -- laser will begin
    /// tuning to the new wavelength. Recommended to use a 
    /// `while laser.query(Tuning{}) {std::thread::sleep(std::time::Duration::from_millis(100));}` loop
    /// or setting other parameters while it's happening
    impl Query for Wavelength {
        type Result = f32;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            Ok(result.parse().unwrap())
        }
    }

    pub struct Power {
        pub laser : DiscoveryLaser,
    }
    impl LaserCommand for Power {
        fn to_string(&self) -> String {
            match self.laser {
                DiscoveryLaser::VariableWavelength => String::from("?PVAR"),
                DiscoveryLaser::FixedWavelength => String::from("?PFIXED"),
            }
        }
    }
    impl Query for Power {
        type Result = f32;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            Ok(result.parse().unwrap())
        }
    }

    pub struct GddCurve {}
    impl LaserCommand for GddCurve {
        fn to_string(&self) -> String {
            String::from("?GDDCURVE")
        }
    }
    impl Query for GddCurve {
        type Result = i32;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            Ok(result.parse().unwrap())
        }
    }

    pub struct GddCurveN {}
    impl LaserCommand for GddCurveN {
        fn to_string(&self) -> String {
            String::from("?GDDCURVEN")
        }
    }
    impl Query for GddCurveN {
        type Result = String;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            Ok(result.to_string())
        }
    }

    pub struct Gdd {}
    impl LaserCommand for Gdd {
        fn to_string(&self) -> String {
            String::from("?GDD")
        }
    }
    impl Query for Gdd {
        type Result = f32;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            Ok(result.parse().unwrap())
        }
    }
    
    pub struct Serial {}
    impl LaserCommand for Serial {
        fn to_string(&self) -> String {
            String::from("?SN")
        }
    }
    impl Query for Serial {
        type Result = String;
        fn parse_result(&self, result : &str) -> Result<Self::Result, CoherentError> {
            Ok(result.to_string())
        }
    }
}


impl Laser for Discovery {
    type CommandEnum = DiscoveryNXCommands;

    fn send_serial_command(&mut self, command : &str) -> Result<(), CoherentError> {
        let command = command.to_string() + "\r\n"; // Need to end with <CR><LF>
        self.port.write_all(command.as_bytes()).map_err(
            |e| CoherentError::WriteError(e)
        )?;
        self.port.flush().map_err(
            |e| CoherentError::WriteError(e)
        )?;
        Ok(())
    }

    /// Checks product ID
    fn is_valid_device(serialportinfo : &serialport::SerialPortInfo)->bool {
        match &serialportinfo.port_type {
            serialport::SerialPortType::UsbPort(info) => {
                LaserType::from(info.pid.clone()) == LaserType::DiscoveryNX
            },
            _ => false
        }
    }

    /// Creates a new instance of the Discovery NX laser from a serial port's information.
    /// 
    /// # Arguments
    /// 
    /// * `serialportinfo` - The serial port information to create the laser from.
    /// 
    /// # Returns
    /// 
    /// A new instance of the Discovery NX laser.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// let port_info = serialport::available_ports().unwrap().into_iter().filter(|port| {
    ///    DiscoveryNX::is_valid_device(port)
    /// }).next().unwrap();
    /// 
    /// let discovery = DiscoveryNX::from_port_info(&port_info);
    /// ```
    fn from_port_info(serialportinfo : &serialport::SerialPortInfo)-> Result<Self, CoherentError> {
        let mut serial_port = serialport::new(&serialportinfo.port_name, BAUDRATE)
            .data_bits(DATABITS)
            .stop_bits(STOPBITS)
            .parity(PARITY)
            .timeout(std::time::Duration::from_secs(2))
            .open().unwrap();

        serial_port.clear(serialport::ClearBuffer::Input)
            .map_err(|e| CoherentError::SerialError(e))?;

        // First check if Echo is on
        serial_port.write_all("?E\r\n".to_string().as_bytes()).map_err(
            |e| CoherentError::WriteError(e)
        )?;
        serial_port.flush().unwrap();

        // Read the result
        let mut buf = String::new();
        let mut reader = std::io::BufReader::new(&mut serial_port);
        reader.read_line(&mut buf)
            .map_err(|_| CoherentError::InvalidResponseError("Error reading line".to_string()))?;
        let echo_on = buf.contains("E 1\r\n");
        let prompt_on = buf.contains("Chameleon");
        if !buf.contains("\r\n") { return Err(CoherentError::InvalidResponseError(buf)); }

        // Get the serial number
        serial_port.write_all(
            "?SN\r\n".to_string().as_bytes()
        ).map_err(|e| CoherentError::WriteError(e))?;
        serial_port.flush().map_err(|e| CoherentError::WriteError(e))?;


        let mut buf = String::new();
        let mut reader = std::io::BufReader::new(&mut serial_port);
        reader.read_line(&mut buf)
            .map_err(|_| CoherentError::InvalidResponseError("Error reading line".to_string()))?;
        if !buf.contains("\r\n") { return Err(CoherentError::InvalidResponseError(buf)); }

        let serial_num : &str;
        if echo_on { serial_num = buf.split("?SN ").collect::<Vec<&str>>()[1].trim(); }
        else { serial_num = buf.trim(); }
        
        // serial_port.clear(serialport::ClearBuffer::All)
        //     .map_err(|e| CoherentError::SerialError(e))?; 


        Ok(Discovery{
            port : serial_port,
            serial_number : serial_num.to_string(),
            echo : echo_on,
            _prompt : prompt_on,
        })
    }

    /// Interface for sending a command to change laser settings.
    /// 
    /// # Arguments
    /// 
    /// * `command` - The command to send to the laser.
    /// 
    /// # Returns
    /// 
    /// A `Result` containing the result of the command (nothing if successful).
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// let mut discovery = Discovery::find_first().unwrap();
    /// discovery.send_command(
    ///     DiscoveryNXCommands::Shutter(
    ///         (DiscoveryLaser::VariableWavelength, ShutterState::Closed)
    ///      )
    /// ).unwrap();
    /// ```
    fn send_command(&mut self, command : DiscoveryNXCommands) -> Result<(), CoherentError> {
        let command_str = command.to_string();
        self.send_serial_command(&command_str)?;
        // Confirm the echo
        let mut buf = String::new();
        let mut reader = std::io::BufReader::new(&mut self.port);
        reader.read_line(&mut buf)
            .map_err(|_| CoherentError::InvalidResponseError("Error reading line".to_string()))?;
        if buf.contains("COMMAND NOT EXECUTED") {
            return Err(CoherentError::CommandNotExecutedError);
        }
        if self._prompt {buf = buf.split("Chameleon>").collect::<Vec<&str>>()[1].to_string();}
        if self.echo {
            let split_on_command = buf.split(&(command_str.clone()+" ")).collect::<Vec<&str>>();
            if split_on_command.len() != 2 {
                return Err(
                    CoherentError::InvalidResponseError(
                        format!{"Echo does not match command. Expected : {}, Got : {}", command_str, buf}
                    )
                )
            }
            if split_on_command[1].trim() != "" {
                return Err(CoherentError::InvalidArgumentsError(
                    split_on_command[1].to_string()
                ));
            }
        }
        else {
            if buf.trim() != "" {
                return Err(CoherentError::InvalidResponseError(
                    format!{"Expected no response, Got : {}", buf}
                ));
            }
        }

        Ok(())
    }

    /// Send a query to the laser that expects a response
    /// 
    /// TODO: Allow type inference to determine the return type
    /// through a generic with Query
    /// 
    /// # Arguments
    /// 
    /// * `query` - The query to send to the laser.
    /// 
    /// # Returns
    /// 
    /// The result of the query as an Enum containing the result.
    /// 
    /// # Example
    /// 
    /// ```
    /// let mut discovery = DiscoveryNX::find_first().unwrap();
    /// let wavelength = discovery.query(DiscoveryNXQueries::Wavelength).unwrap();
    /// println!("Wavelength : {:?}", wavelength);
    /// ```
    fn query<Q:Query>(&mut self, query : Q) -> Result<Q::Result, CoherentError> {
        let query_str = query.to_string();
        self.send_serial_command(&query_str)?;
        self.port.flush()
            .map_err(|e| CoherentError::InvalidResponseError(e.to_string()))?;
        let mut buf = String::new();
        let mut reader = std::io::BufReader::new(&mut self.port);
        reader.read_line(&mut buf)
            .map_err(|_| CoherentError::InvalidResponseError("Error reading line".to_string()))?;
        if self._prompt {buf = buf.split("Chameleon>").collect::<Vec<&str>>()[1].to_string();}
        let buf : Vec<&str> = buf.trim().split(&(query_str+" ")).collect();
        let buf = match self.echo {
            false => buf[0],
            true => buf[1],
        };
        self.port.flush().map_err(|e| CoherentError::InvalidResponseError(e.to_string()))?;
        query.parse_result(buf)
    }
}

/// Convenience functions
impl Discovery {

    /// Set the wavelength of the variable-wavelength laser
    /// 
    /// # Arguments
    /// 
    /// * `wavelength` - The wavelength to set the laser to (in nanometers).
    /// 
    /// # Example
    /// 
    /// ```
    /// let mut discovery = Discovery::find_first().unwrap();
    /// discovery.set_wavelength(840.0).unwrap();
    /// ```
    pub fn set_wavelength(&mut self, wavelength : f32) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::Wavelength{wavelength_nm : wavelength})
    }

    pub fn get_wavelength(&mut self) -> Result<f32, CoherentError> {
        self.query(DiscoveryNXQueries::Wavelength{})
    }

    pub fn set_gdd(&mut self, gdd : f32) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::Gdd{gdd_val : gdd})
    }

    pub fn get_gdd(&mut self) -> Result<f32, CoherentError> {
        self.query(DiscoveryNXQueries::Gdd{})
    }

    pub fn set_shutter(&mut self, laser : DiscoveryLaser, state : ShutterState) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::Shutter{laser, state})
    }

    pub fn get_shutter(&mut self, laser : DiscoveryLaser) -> Result<ShutterState, CoherentError> {
        self.query(DiscoveryNXQueries::Shutter{laser})
    }

    pub fn set_gdd_curve(&mut self, curve : u8) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::GddCurve{curve_num : curve})
    }

    pub fn get_gdd_curve(&mut self) -> Result<i32, CoherentError> {
        self.query(DiscoveryNXQueries::GddCurve{})
    }

    pub fn set_gdd_curve_n(&mut self, name : &str) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::GddCurveN{curve_name : name.to_string()})
    }

    pub fn get_gdd_curve_n(&mut self) -> Result<String, CoherentError> {
        self.query(DiscoveryNXQueries::GddCurveN{})
    }
    
    pub fn set_alignment_mode(&mut self, laser : DiscoveryLaser, mode : bool) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::AlignmentMode{laser, alignment_mode_on : mode})
    }

    pub fn get_alignment_mode(&mut self, laser : DiscoveryLaser) -> Result<bool, CoherentError> {
        self.query(DiscoveryNXQueries::AlignmentMode{laser})
    }

    pub fn get_power(&mut self, laser : DiscoveryLaser) -> Result<f32, CoherentError> {
        self.query(DiscoveryNXQueries::Power{laser})
    }

    pub fn get_serial(&mut self) -> Result<String, CoherentError> {
        self.query(DiscoveryNXQueries::Serial{})
    }

    pub fn set_to_standby(&mut self, standby : bool) -> Result<(), CoherentError> {
        self.send_command(
            DiscoveryNXCommands::Laser{state : if standby {LaserState::Standby} else {LaserState::On}}
        )
    }

    pub fn get_standby(&mut self) -> Result<LaserState, CoherentError> {
        self.query(DiscoveryNXQueries::Laser{})
    }

    pub fn get_keyswitch_on(&mut self) -> Result<bool, CoherentError> {
        self.query(DiscoveryNXQueries::Keyswitch{})
    }

    pub fn get_status(&mut self) -> Result<String, CoherentError> {
        self.query(DiscoveryNXQueries::Status{})
    }

    pub fn clear_faults(&mut self) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::FaultClear)
    }

    pub fn get_faults(&mut self) -> Result<u8, CoherentError> {
        self.query(DiscoveryNXQueries::Faults{})
    }

    pub fn get_fault_text(&mut self) -> Result<String, CoherentError> {
        self.query(DiscoveryNXQueries::FaultText{})
    }

    pub fn get_tuning(&mut self) -> Result<TuningStatus, CoherentError> {
        self.query(DiscoveryNXQueries::Tuning{})
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands(){
        let mut discovery = Discovery::find_first().unwrap();

        discovery.send_command(
            DiscoveryNXCommands::Shutter{
                laser: DiscoveryLaser::VariableWavelength,
                state: ShutterState::Open}
        ).unwrap();
    }
    
    #[test]
    fn test_queries() {
        let mut discovery = Discovery::find_first().unwrap();
        let echo = discovery.query(DiscoveryNXQueries::Echo{}).unwrap();
        let laser = discovery.query(DiscoveryNXQueries::Laser{}).unwrap();
        let shutter_var = discovery.query(DiscoveryNXQueries::Shutter{laser : DiscoveryLaser::VariableWavelength}).unwrap();
        let shutter_fixed = discovery.query(DiscoveryNXQueries::Shutter{laser :DiscoveryLaser::FixedWavelength}).unwrap();
        let keyswitch = discovery.query(DiscoveryNXQueries::Keyswitch{}).unwrap();
        let faults = discovery.query(DiscoveryNXQueries::Faults{}).unwrap();
        let fault_text = discovery.query(DiscoveryNXQueries::FaultText{}).unwrap();
        let tuning = discovery.query(DiscoveryNXQueries::Tuning{}).unwrap();
        let alignment_var = discovery.query(DiscoveryNXQueries::AlignmentMode {laser : DiscoveryLaser::VariableWavelength}).unwrap();
        let alignment_fixed = discovery.query(DiscoveryNXQueries::AlignmentMode{ laser : DiscoveryLaser::FixedWavelength}).unwrap();
        let status = discovery.query(DiscoveryNXQueries::Status{}).unwrap();
        let wavelength = discovery.query(DiscoveryNXQueries::Wavelength{}).unwrap();
        let power_var = discovery.query(DiscoveryNXQueries::Power{laser : DiscoveryLaser::VariableWavelength}).unwrap();
        let power_fixed = discovery.query(DiscoveryNXQueries::Power{laser : DiscoveryLaser::FixedWavelength}).unwrap();
        let gdd_curve = discovery.query(DiscoveryNXQueries::GddCurve{}).unwrap();
        let gdd_curve_n = discovery.query(DiscoveryNXQueries::GddCurveN{}).unwrap();
        let gdd = discovery.query(DiscoveryNXQueries::Gdd{}).unwrap();
        let serial = discovery.query(DiscoveryNXQueries::Serial{}).unwrap();

        println!{"Echo : {:?}, Laser : {:?}, Shutter Var : {:?}, Shutter Fixed : {:?}, Keyswitch : {:?}, Faults : {:?}, Fault Text : {:?}, Tuning : {:?}, Alignment Var : {:?}, Alignment Fixed : {:?}, Status : {:?}, Wavelength : {:?}, Power Var : {:?}, Power Fixed : {:?}, GDD Curve : {:?}, GDD Curve N : {:?}, GDD : {:?}, Serial : {:?}",
        echo, laser, shutter_var, shutter_fixed, keyswitch, faults, fault_text, tuning, alignment_var, alignment_fixed, status, wavelength, power_var, power_fixed, gdd_curve, gdd_curve_n, gdd, serial
        };
    }

    #[test]
    fn test_shutter() {
        use std::thread;
        let mut discovery = Discovery::find_first().unwrap();
        
        let mut shutter_state = discovery.query(
            DiscoveryNXQueries::Shutter{
                laser : DiscoveryLaser::VariableWavelength
            }
        ).unwrap();

        println!("Variable shutter state: {:?}... setting to {:?}", shutter_state, !shutter_state);

        discovery.send_command(
            DiscoveryNXCommands::Shutter{
                laser : DiscoveryLaser::VariableWavelength,
                state : !shutter_state
            }
        )
        .unwrap();

        thread::sleep(std::time::Duration::from_millis(300));

        shutter_state = discovery.query(
            DiscoveryNXQueries::Shutter{
                laser : DiscoveryLaser::VariableWavelength
            }
        ).unwrap();

        thread::sleep(std::time::Duration::from_millis(300));

        println!("Variable shutter state: {:?}... setting to {:?}", shutter_state, !shutter_state);

        discovery.send_command(
            DiscoveryNXCommands::Shutter{
                laser : DiscoveryLaser::VariableWavelength,
                state : !shutter_state
            }
        ).unwrap();

        thread::sleep(std::time::Duration::from_millis(300));

        shutter_state = discovery.query(
            DiscoveryNXQueries::Shutter{
                laser : DiscoveryLaser::VariableWavelength
            }
        ).unwrap();

        println!("Variable shutter state: {:?}", shutter_state);
    }

    #[test]
    fn test_wavelength(){
        let mut discovery = Discovery::find_first().unwrap();

        let wv = discovery.query(
            DiscoveryNXQueries::Wavelength{}
        ).unwrap();

        println!("Wavelength: {:?}", wv);

        discovery.send_command(
            DiscoveryNXCommands::Wavelength{wavelength_nm : 840.0}
        ).unwrap();

        while discovery.query(DiscoveryNXQueries::Tuning{}).unwrap().into() {
            std::thread::sleep(std::time::Duration::from_millis(50));
            println!{
                "Laser power : {:?}",
                discovery.query(DiscoveryNXQueries::Power{laser : DiscoveryLaser::VariableWavelength}).unwrap()
            }
        }

        let new_wv = discovery.query(
            DiscoveryNXQueries::Wavelength{}
        ).unwrap();

        println!("Wavelength: {:?}", new_wv);

        discovery.send_command(
            DiscoveryNXCommands::Wavelength{wavelength_nm : wv}
        ).unwrap();

        while discovery.query(DiscoveryNXQueries::Tuning{}).unwrap().into() {
            std::thread::sleep(std::time::Duration::from_millis(50));
            println!{
                "Laser power : {:?}",
                discovery.query(DiscoveryNXQueries::Power{laser : DiscoveryLaser::VariableWavelength}).unwrap()
            }
        }

        let new_wv = discovery.query(
            DiscoveryNXQueries::Wavelength{}
        ).unwrap();

        println!("Wavelength: {:?}", new_wv);

    }

    #[test]
    fn test_invalid_args() {
        let mut discovery = Discovery::find_first().unwrap();

        println!("Testing invalid wavelength");

        let result = discovery.send_command(
            DiscoveryNXCommands::Wavelength{wavelength_nm : 0.0}
        );

        assert!(result.is_err());

        let wv = discovery.query(
            DiscoveryNXQueries::Wavelength{}
        ).unwrap();

        println!("Wavelength: {:?}", wv);

        println!("Testing invalid GDD");

        let result = discovery.send_command(
            DiscoveryNXCommands::Gdd{gdd_val : 50000.0}
        );

        assert!(result.is_err());

        let gdd = discovery.query(
            DiscoveryNXQueries::Gdd{}
        ).unwrap();

        println!("GDD: {:?}", gdd);



    }

    #[test]
    fn test_gdd() {
        let mut discovery = Discovery::find_first().unwrap();

        let current_gdd = discovery.query(
            DiscoveryNXQueries::Gdd{}
        ).unwrap();

        println!("GDD: {:?}... Setting to 0", current_gdd);

        discovery.send_command(
            DiscoveryNXCommands::Gdd{gdd_val : 0.0}
        ).unwrap();

        let new_gdd = discovery.query(
            DiscoveryNXQueries::Gdd{}
        ).unwrap();

        println!("New GDD: {:?}", new_gdd);

        discovery.send_command(
            DiscoveryNXCommands::Gdd{gdd_val : current_gdd}
        ).unwrap();

        let new_gdd = discovery.query(
            DiscoveryNXQueries::Gdd{}
        ).unwrap();

        println!("Returned GDD: {:?}", new_gdd);
    }

    #[test]
    fn test_convenience_funcs() {
        let mut discovery = Discovery::find_first().unwrap();

        let current_gdd = discovery.get_gdd().unwrap();
        println!("GDD: {:?}... Setting to 0", current_gdd);


        std::thread::sleep(std::time::Duration::from_millis(100));
        discovery.set_gdd(0.0).unwrap();

        let new_gdd = discovery.get_gdd().unwrap();
        println!("New GDD: {:?}", new_gdd);

        std::thread::sleep(std::time::Duration::from_millis(100));

        discovery.set_gdd(current_gdd).map_err(
            |e| {match e {
                CoherentError::CommandNotExecutedError => discovery.set_gdd(current_gdd).unwrap(),
                _ => println!("Error : {:?}", e)
            }}
        ).unwrap();
        println!("Returned GDD: {:?}", discovery.get_gdd().unwrap());

        let current_wv = discovery.get_wavelength().unwrap();
        println!("Wavelength: {:?}... Setting to 840", current_wv);

        discovery.set_wavelength(840.0).unwrap();

        while discovery.query(DiscoveryNXQueries::Tuning{}).unwrap().into() {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let new_wv = discovery.get_wavelength().unwrap();
        println!("New Wavelength: {:?}", new_wv);

        discovery.set_wavelength(current_wv).unwrap();

        while discovery.query(DiscoveryNXQueries::Tuning{}).unwrap().into() {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        println!("Returned Wavelength: {:?}", discovery.get_wavelength().unwrap());

        println!("Opening variable shutter");
        discovery.set_shutter(DiscoveryLaser::VariableWavelength, ShutterState::Open).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("Shutter state: {:?}", discovery.get_shutter(DiscoveryLaser::VariableWavelength).unwrap());
        println!("Closing variable shutter");
        discovery.set_shutter(DiscoveryLaser::VariableWavelength, ShutterState::Closed).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("Shutter state: {:?}", discovery.get_shutter(DiscoveryLaser::VariableWavelength).unwrap());

        println!("Opening fixed shutter");
        discovery.set_shutter(DiscoveryLaser::FixedWavelength, ShutterState::Open).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("Shutter state: {:?}", discovery.get_shutter(DiscoveryLaser::FixedWavelength).unwrap());
        println!("Closing fixed shutter");
        discovery.set_shutter(DiscoveryLaser::FixedWavelength, ShutterState::Closed).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("Shutter state: {:?}", discovery.get_shutter(DiscoveryLaser::FixedWavelength).unwrap());

        println!("Setting variable alignment mode to true");
        discovery.set_alignment_mode(DiscoveryLaser::VariableWavelength, true).unwrap();
        println!("Alignment mode: {:?}", discovery.get_alignment_mode(DiscoveryLaser::VariableWavelength).unwrap());

        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("Setting variable alignment mode to false");
        discovery.set_alignment_mode(DiscoveryLaser::VariableWavelength, false).unwrap();
        println!("Alignment mode: {:?}", discovery.get_alignment_mode(DiscoveryLaser::VariableWavelength).unwrap());

        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("Setting fixed alignment mode to true");
        discovery.set_alignment_mode(DiscoveryLaser::FixedWavelength, true).unwrap();
        println!("Alignment mode: {:?}", discovery.get_alignment_mode(DiscoveryLaser::FixedWavelength).unwrap());
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("Setting fixed alignment mode to false");
        discovery.set_alignment_mode(DiscoveryLaser::FixedWavelength, false).unwrap();
        println!("Alignment mode: {:?}", discovery.get_alignment_mode(DiscoveryLaser::FixedWavelength).unwrap());




    }
}