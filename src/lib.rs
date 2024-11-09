//! Coherent-RS is a `Rust` library to control Coherent brand lasers common
//! in two-photon microscopy experiments. It implements simple serial control
//! of the laser and provides an asynchronous API to read out and control laser
//! parameters.

pub mod laser;
use laser::Laser;

/// The error types that can be returned by the Coherent-RS library.
pub enum CoherentError {
    SerialError(serialport::Error),
    WriteError(std::io::Error),
    TimeoutError,
    InvalidArgumentsError(String),
    InvalidResponseError,
    LaserUnavailableError,
    UnrecognizedDevice,
}

impl From<serialport::Error> for CoherentError {
    fn from(error : serialport::Error) -> Self {
        CoherentError::SerialError(error)
    }
}

/// Open a serial connection to the Coherent laser.
/// 
/// # Arguments
/// 
/// * `port` - The serial port to connect to.
/// 
/// 
pub fn open<L : Laser>(port : &str) -> Result<L, CoherentError> {
    // Open serial port
    Err(CoherentError::UnrecognizedDevice)
}