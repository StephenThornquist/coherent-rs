//! debug.rs
//! 
//! Spoofs a DiscoveryNX without needing to actually connect to one.
#[cfg(feature = "network")]
use serde::Serialize;

use crate::{CoherentError, Laser};
use crate::laser::discoverynx::{DiscoveryNXCommands, DiscoveryNXStatus, DiscoveryLaser};
use crate::laser::{Query, LaserState, ShutterState, LaserType, TuningStatus};


/// The Coherent laser model Discovery NX.
#[derive(Debug)]
#[repr(C)]
pub struct DebugLaser{
    pub serial_number : String,
    echo : bool, // whether or not the laser will echo commands, which affects parsing
    _prompt : bool, // whether or not the laser will echo prompts, which affects parsing
    _variable_shutter : bool,
    _fixed_shutter : bool,
    _variable_alignment : bool,
    _fixed_alignment : bool,
    _variable_power : f32,
    _fixed_power : f32,
    _variable_wavelength : f32,
    _tuning_status : bool,
    _gdd : f32,
    _gdd_curve_n : String,
    _gdd_curve : i32,
    _status : String,
    _fault_text : String,
}

impl Into<LaserType> for DebugLaser {
    fn into(self) -> LaserType {
        LaserType::DebugLaser
    }
}

impl Default for DebugLaser{
    fn default() -> Self {
        DebugLaser{
            serial_number : "DEBUG".to_string(),
            echo : true,
            _prompt : false,
            _variable_shutter : false,
            _fixed_shutter : false,
            _variable_alignment : false,
            _fixed_alignment : false,
            _variable_power : 1000.0,
            _fixed_power : 5000.0,
            _variable_wavelength : 920.0,
            _tuning_status : false,
            _gdd : 0.0,
            _gdd_curve_n : "Default".to_string(),
            _gdd_curve : 0,
            _status : "OK".to_string(),
            _fault_text : "No faults".to_string(),
        }
    }
}

impl Laser for DebugLaser {
    type CommandEnum = DiscoveryNXCommands;
    #[cfg(feature = "network")]
    type LaserStatus = DiscoveryNXStatus;

    /// Does nothing.
    fn send_serial_command(&mut self, _command : &str) -> Result<(), CoherentError> {
        Ok(())
    }

    /// Always true
    fn is_valid_device(_serialportinfo : &serialport::SerialPortInfo)->bool {
        true
    }

    /// Creates a new instance of the `DebugLaser`.
    /// 
    /// # Arguments
    /// 
    /// * `serialportinfo` - Always succeeds
    /// 
    /// # Returns
    /// 
    /// A new instance of the `DebugLaser`.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// let laser = DebugLaser::from_port_info(&serialportinfo);
    /// ```
    fn from_port_info(_serialportinfo : &serialport::SerialPortInfo)-> Result<Self, CoherentError> {
        Ok(DebugLaser::default())
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
    ///
    /// ```
    fn send_command(&mut self, command : DiscoveryNXCommands) -> Result<(), CoherentError> {
        
        match command {
            DiscoveryNXCommands::Echo{echo_on} => {
                self.echo = echo_on;
            },
            DiscoveryNXCommands::Wavelength{wavelength_nm} => {
                if wavelength_nm < 700.0 || wavelength_nm > 1000.0 {
                    return Err(CoherentError::CommandNotExecutedError);
                }
                self._variable_wavelength = wavelength_nm;
            },
            DiscoveryNXCommands::Gdd{gdd_val} => {
                if gdd_val < -10000.0 || gdd_val > 10000.0 {
                    return Err(CoherentError::CommandNotExecutedError);
                }
                self._gdd = gdd_val;
            },
            DiscoveryNXCommands::Shutter{laser, state} => {
                match laser {
                    DiscoveryLaser::VariableWavelength => {
                        self._variable_shutter = state == ShutterState::Open;
                    },
                    DiscoveryLaser::FixedWavelength => {
                        self._fixed_shutter = state == ShutterState::Open;
                    }
                }
            },
            DiscoveryNXCommands::GddCurve{curve_num} => {
                self._gdd_curve = curve_num.into();
            },
            DiscoveryNXCommands::GddCurveN{curve_name} => {
                self._gdd_curve_n = curve_name;
            },
            DiscoveryNXCommands::AlignmentMode{laser, alignment_mode_on} => {
                match laser {
                    DiscoveryLaser::VariableWavelength => {
                        self._variable_alignment = alignment_mode_on;
                    },
                    DiscoveryLaser::FixedWavelength => {
                        self._fixed_alignment = alignment_mode_on;
                    }
                }
            },
            DiscoveryNXCommands::Laser{state} => {
                match state {
                    LaserState::Standby => {
                        self._status = "Standby".to_string();
                    },
                    LaserState::On => {
                        self._status = "On".to_string();
                    }
                }
            },
            DiscoveryNXCommands::FaultClear => {
                self._fault_text = "No faults".to_string();
            }
            _ => {}
        }

        Ok(())
    }

    /// Always fails! Queries are implemented using the actual serial communication,
    /// and so with a dummy laser they cannot be used. Please use the convenience functions
    /// instead.
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
    /// ```
    fn query<Q:Query>(&mut self, _query : Q) -> Result<Q::Result, CoherentError> {
        Err(CoherentError::CommandNotExecutedError)
    }

    #[cfg(feature = "network")]
    fn status(&mut self) -> Result<Self::LaserStatus, CoherentError> {
        Ok(DiscoveryNXStatus {
            echo : self.echo,
            laser : LaserState::On,
            variable_shutter : self._variable_shutter.into(),
            fixed_shutter : self._fixed_shutter.into(),
            keyswitch : true,
            faults : 0,
            fault_text : self._fault_text.clone(),
            tuning : self._tuning_status.into(),
            alignment_var : self._variable_alignment,
            alignment_fixed : self._fixed_alignment,
            power_var : self._variable_power,
            power_fixed : self._fixed_power,
            wavelength : self._variable_wavelength,
            gdd : self._gdd,
            gdd_curve_n : self._gdd_curve_n.clone(),
            gdd_curve : self._gdd_curve,
            status : self._status.clone(),
        })
    }

    #[cfg(feature = "network")]
    fn serialized_status(&mut self) -> Result<Vec<u8>, CoherentError> {
        let laser_status = self.status()?;

        let mut buf = Vec::new();
        laser_status.serialize(&mut rmp_serde::Serializer::new(&mut buf)).unwrap();
        Ok(buf)
    } 

    fn into_laser_type() -> LaserType {
        LaserType::DebugLaser
    }
}

/// Convenience functions
impl DebugLaser {

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
        Ok(self._variable_wavelength)
    }

    pub fn set_gdd(&mut self, gdd : f32) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::Gdd{gdd_val : gdd})
    }

    pub fn get_gdd(&mut self) -> Result<f32, CoherentError> {
        Ok(self._gdd)
    }

    pub fn set_shutter(&mut self, laser : DiscoveryLaser, state : ShutterState) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::Shutter{laser, state})
    }

    pub fn get_shutter(&mut self, laser : DiscoveryLaser) -> Result<ShutterState, CoherentError> {
        match laser {
            DiscoveryLaser::VariableWavelength => {
                if self._variable_shutter {
                    Ok(ShutterState::Open)
                } else {
                    Ok(ShutterState::Closed)
                }
            },
            DiscoveryLaser::FixedWavelength => {
                if self._fixed_shutter {
                    Ok(ShutterState::Open)
                } else {
                    Ok(ShutterState::Closed)
                }
            }
        }
    }

    pub fn set_gdd_curve(&mut self, curve : u8) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::GddCurve{curve_num : curve})
    }

    pub fn get_gdd_curve(&mut self) -> Result<i32, CoherentError> {
        Ok(self._gdd_curve)
    }

    pub fn set_gdd_curve_n(&mut self, name : &str) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::GddCurveN{curve_name : name.to_string()})
    }

    pub fn get_gdd_curve_n(&mut self) -> Result<String, CoherentError> {
        Ok(self._gdd_curve_n.clone())
    }
    
    pub fn set_alignment_mode(&mut self, laser : DiscoveryLaser, mode : bool) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::AlignmentMode{laser, alignment_mode_on : mode})
    }

    pub fn get_alignment_mode(&mut self, laser : DiscoveryLaser) -> Result<bool, CoherentError> {
        match laser {
            DiscoveryLaser::VariableWavelength => Ok(self._variable_alignment),
            DiscoveryLaser::FixedWavelength => Ok(self._fixed_alignment)
        }
    }

    pub fn get_power(&mut self, laser : DiscoveryLaser) -> Result<f32, CoherentError> {
        match laser {
            DiscoveryLaser::VariableWavelength => Ok(self._variable_power),
            DiscoveryLaser::FixedWavelength => Ok(self._fixed_power)
        }
    }

    pub fn get_serial(&mut self) -> Result<String, CoherentError> {
        Ok(self.serial_number.clone())
    }

    pub fn set_to_standby(&mut self, standby : bool) -> Result<(), CoherentError> {
        self.send_command(
            DiscoveryNXCommands::Laser{state : if standby {LaserState::Standby} else {LaserState::On}}
        )
    }

    pub fn get_standby(&mut self) -> Result<LaserState, CoherentError> {
        if self._status == "Standby" {
            Ok(LaserState::Standby)
        } else {
            Ok(LaserState::On)
        }
    }

    pub fn get_keyswitch_on(&mut self) -> Result<bool, CoherentError> {
        Ok(true)
    }

    pub fn get_status(&mut self) -> Result<String, CoherentError> {
        Ok(self._status.clone())
    }

    pub fn clear_faults(&mut self) -> Result<(), CoherentError> {
        self.send_command(DiscoveryNXCommands::FaultClear)
    }

    pub fn get_faults(&mut self) -> Result<u8, CoherentError> {
        Ok(0)
    }

    pub fn get_fault_text(&mut self) -> Result<String, CoherentError> {
        Ok(self._fault_text.clone())
    }

    pub fn get_tuning(&mut self) -> Result<TuningStatus, CoherentError> {
        match self._tuning_status {
            true => Ok(TuningStatus::Tuning),
            false => Ok(TuningStatus::Ready),
        }
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands(){
        let mut discovery = DebugLaser::find_first().unwrap();

        discovery.send_command(
            DiscoveryNXCommands::Shutter{
                laser: DiscoveryLaser::VariableWavelength,
                state: ShutterState::Open}
        ).unwrap();
    }

    #[test]
    fn test_shutter() {
        use std::thread;
        let mut discovery = DebugLaser::find_first().unwrap();
        
        let mut shutter_state = discovery.get_shutter(
            DiscoveryLaser::VariableWavelength,
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

        shutter_state = discovery.get_shutter(
            DiscoveryLaser::VariableWavelength,
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

        shutter_state = discovery.get_shutter(
            DiscoveryLaser::VariableWavelength,
        ).unwrap();

        println!("Variable shutter state: {:?}", shutter_state);
    }

    #[test]
    fn test_wavelength(){
        let mut discovery = DebugLaser::find_first().unwrap();

        let wv = discovery.get_wavelength().unwrap();

        println!("Wavelength: {:?}", wv);

        discovery.send_command(
            DiscoveryNXCommands::Wavelength{wavelength_nm : 840.0}
        ).unwrap();

        let new_wv = discovery.get_wavelength().unwrap();

        println!("Wavelength: {:?}", new_wv);

        discovery.send_command(
            DiscoveryNXCommands::Wavelength{wavelength_nm : wv}
        ).unwrap();

        let new_wv = discovery.get_wavelength().unwrap();

        println!("Wavelength: {:?}", new_wv);

    }


    #[cfg(feature = "network")]
    #[test]
    fn test_serde_command(){
        use rmp_serde::Serializer;
        use serde::{Serialize, Deserialize};
        let command = DiscoveryNXCommands::Echo{echo_on : true};

        let mut buf = Vec::new();
        command.serialize(&mut Serializer::new(&mut buf)).unwrap();
    
        println!("Serialized : {:?}", buf);

        match DiscoveryNXCommands::deserialize(
            &mut rmp_serde::Deserializer::new(&buf[..])) {
            Ok(DiscoveryNXCommands::Echo{echo_on}) => assert_eq!(echo_on, true),
            _ => panic!("Wrong command type")
            }
        
        let command = DiscoveryNXCommands::Shutter {
            laser : DiscoveryLaser::VariableWavelength,
            state : ShutterState::Open
        };

        buf.clear();
        command.serialize(&mut Serializer::new(&mut buf)).unwrap();

        match DiscoveryNXCommands::deserialize(
            &mut rmp_serde::Deserializer::new(&buf[..])) {
            Ok(DiscoveryNXCommands::Shutter{laser, state}) => {
                assert_eq!(laser, DiscoveryLaser::VariableWavelength);
                assert_eq!(state, ShutterState::Open);
            },
            _ => panic!("Wrong command type")
        }
    }
}