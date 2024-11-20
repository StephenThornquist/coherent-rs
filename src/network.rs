//! `network.rs`
//! 
//! This module creates a network listener that is thread safe
//! and can be used to control a Coherent laser over a network.
//! It uses `serde` to communicate `Command`s and `Query`s over
//! the network.

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::net::{TcpListener, TcpStream};
use crate::{
    laser::{Laser, Query},
    CoherentError,
};

use serde::{Serialize, Deserialize};
use rmp_serde::{Serializer, Deserializer};

/// Errors during communication with the laser over the network.
pub enum TcpError {
    MultipleReferencesToLaser,
    // MutexPoisoned<L>,
    CoherentError(CoherentError),
    IoError(std::io::Error),
    SerializationError(rmp_serde::encode::Error),
}

/// A `Laser` with a network listener that can be used to control
/// the laser in addition to the normal `Laser` methods. Takes ownership
/// of the `Laser` and maintains exclusive access through a `Mutex`.
/// 
/// # Example
/// 
/// ```rust
/// use coherent_rs::{Discovery, create_listener};
/// let laser = Discovery::find_first();
/// // To be continued...
/// ```
pub struct NetworkLaser<L : Laser> {
    _listener : TcpListener,
    _laser : Arc<Mutex<L>>,
    _polling_interval : Arc<Mutex<f32>>, // seconds
}

/// Create a network listener that listens on the specified port.
/// Takes ownership over the `Laser` so that it can be polled and
/// shared between threads, and maintains exclusive access through
/// a `Mutex`.
/// 
/// Default polling interval is 1 second.
pub fn create_listener<L : Laser>(laser : L, port : &str, polling_interval : Option<f32>) -> Result<NetworkLaser<L>, TcpError> {
    let listener = TcpListener::bind(port)
        .map_err(|e| TcpError::IoError(e))?;
    Ok(NetworkLaser {
        _listener : listener,
        _laser : Arc::new(Mutex::new(laser)),
        _polling_interval : Arc::new(Mutex::new(polling_interval.unwrap_or(1.0))),
    })
}

impl<L : Laser> NetworkLaser<L> {
    /// Returns the name of the port that the listener is listening on.
    pub fn get_port(&self) -> String {
        self._listener.local_addr().unwrap().port().to_string()
    }

    /// Sets the polling interval in seconds
    pub fn set_polling_interval(&mut self, interval : f32) {
        let mut polling_interval = self._polling_interval.lock().unwrap();
        *polling_interval = interval;
    }

    /// Returns the laser and kills the `NetworkLaser`
    pub fn get_laser(self) -> Result<L, TcpError> {
        Arc::try_unwrap(self._laser)
            .map_err(|_| TcpError::MultipleReferencesToLaser)
            .and_then(
                |mutex| 
                mutex.into_inner().map_err(|_| TcpError::MultipleReferencesToLaser)
            )
    }

    pub fn listen(&self) -> () {}

    /// Send a command to the laser through the mutex
    pub fn command(&self, command : L::CommandEnum) -> Result<(), TcpError> {
        let mut laser = self._laser.lock().unwrap();
        laser.send_command(command).map_err(|e| TcpError::CoherentError(e))
    }

    /// Send a query to the laser through the mutex
    pub fn query<Q : Query> (&self, query : Q) -> Result<Q::Result, TcpError> {
        let mut laser = self._laser.lock().unwrap();
        laser.query(query).map_err(|e| TcpError::CoherentError(e))
    }
}

/// A struct to connect to and communicate with a
/// `NetworkLaser` over the network.
pub struct NetworkLaserInterface {
    _stream : TcpStream,
}

impl NetworkLaserInterface {
    /// Connect to a `NetworkLaser` over the network, if it exists
    /// 
    /// # Example
    /// ```rust
    /// use coherent_rs::{Discovery, create_listener, NetworkLaserInterface};
    /// ```
    fn connect(&self, port : &str) -> Result<Self, TcpError> {
        let stream = TcpStream::connect(port)
            .map_err(|e| TcpError::IoError(e))?;

        Ok(NetworkLaserInterface {
            _stream : stream,
        })
    }

    /// Issue a command over the stream
    fn command<L : Laser>(&mut self, command : L::CommandEnum) -> Result<(), TcpError> {
        
        let mut buf = Vec::new();
        command.serialize(&mut Serializer::new(&mut buf))
            .map_err(|e| TcpError::SerializationError(e))?;
        // Send command over the network

        self._stream.write_all(buf.as_slice())
            .map_err(|e| TcpError::IoError(e))
    }
}