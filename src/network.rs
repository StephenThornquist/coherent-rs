//! `network.rs`
//! 
//! This module creates a network listener that is thread safe
//! and can be used to control a Coherent laser over a network.
//! It uses `serde` to communicate `Command`s and `Query`s over
//! the network.

use std::io::{Read,Write};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use std::net::{TcpListener, TcpStream};
use crate::{
    laser::{Laser, Query, LaserType},
    CoherentError,
};

// Do I need these? I want to be fully generic if possible...
use crate::laser::{Discovery, debug::DebugLaser};

use serde::{Serialize, Deserialize};
use rmp_serde::Serializer;

pub const COMMAND_MARKER : &[u8] = b"Command: ";
pub const STATUS_MARKER : &[u8] = b"Status: ";
pub const TERMINATOR : &[u8] = b"\n";
pub const LASER_ID : &[u8] = b"Laser ID: ";
pub const COMMAND_SUCCESSFUL : &[u8] = b"COMMAND SUCCESSFUL\n";
pub const COMMAND_FAILED : &[u8] = b"COMMAND FAILED\n";
pub const NOT_PRIMARY_CLIENT : &[u8] = b"NOT PRIMARY CLIENT\n";

/// Errors during communication with the laser over the network.
#[derive(Debug)]
pub enum TcpError {
    MultipleReferencesToLaser,
    // MutexPoisoned<L>,
    CoherentError(CoherentError),
    IoError(std::io::Error),
    SerializationEncodeError(rmp_serde::encode::Error),
    SerializationDecodeError(rmp_serde::decode::Error),
    CommandError,
    NoLaserStatus,
    NotPrimaryClient,
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
pub struct NetworkLaser<L : Laser + 'static> {
    _listener : TcpListener,
    _clients : Arc<Mutex<Vec<TcpStream>>>,
    _client_connection_thread : Option<std::thread::JoinHandle<()>>,
    _laser : Arc<Mutex<L>>,
    _polling_interval : Arc<Mutex<f32>>, // seconds
    _polling_thread : Option<std::thread::JoinHandle<()>>,
    _polling : Arc<AtomicBool>,
    _command_thread : Option<std::thread::JoinHandle<()>>, // polls for commands -- runs faster to ensure commands are executed.
    _primary_client : Option<Arc<Mutex<TcpStream>>>, // defines a primary client -- if defined, only the primary client can issue commands.
}

/// Reads a laser status from a stream returns a `Result` with the `LaserStatus`
/// or a `TcpError`. Looks for the `STATUS_MARKER` and the `TERMINATOR` in the stream.
/// Searches backwards from the end of the stream to find the `STATUS_MARKER`.
/// 
/// # Example
/// 
/// ```rust
/// use coherent_rs::laser::{Laser, debug::DebugLaser};
/// use coherent_rs::network::{STATUS_MARKER, deserialize_laser_status, TERMINATOR};
/// 
/// let mut laser = DebugLaser::default();
/// let status_serialized = laser.serialized_status().unwrap();
/// 
/// let mut sent_message = STATUS_MARKER.to_vec();
/// sent_message.extend(status_serialized);
/// sent_message.extend(TERMINATOR);
/// 
/// let status = deserialize_laser_status::<DebugLaser>(&sent_message).unwrap();
/// println!{"Deserialized : {:?}", status};
/// ```
fn deserialize_laser_status<L : Laser>(stream : &[u8]) -> Result<L::LaserStatus, TcpError> {
    if let Some(start_idx) = stream.windows(STATUS_MARKER.len()).rposition(
        |window| window == STATUS_MARKER
    ){
        let status = &stream[start_idx + STATUS_MARKER.len()..];
        if let Some(end) = status.split(|&x| x == TERMINATOR[0]).next() {
            let serialized = &status[..end.len()];
            L::LaserStatus::deserialize(
                &mut rmp_serde::Deserializer::new(serialized)
            ).map_err(|e| TcpError::SerializationDecodeError(e))
        }
        else {
            Err(TcpError::NoLaserStatus)
        }
    }
    else {
        Err(TcpError::NoLaserStatus)
    }
}

fn deserialize_command<L : Laser>(stream : &[u8]) -> Result<L::CommandEnum, TcpError> {
    if let Some(start_idx) = stream.windows(COMMAND_MARKER.len()).position(
        |window| window == COMMAND_MARKER
    ){
        let command = &stream[start_idx + COMMAND_MARKER.len()..];
        if let Some(end) = command.split(|&x| x == TERMINATOR[0]).next() {
            let serialized = &command[..end.len()];
            L::CommandEnum::deserialize(
                &mut rmp_serde::Deserializer::new(serialized)
            ).map_err(|e| TcpError::SerializationDecodeError(e))
        }
        else {
            Err(TcpError::NoLaserStatus)
        }
    }
    else {
        Err(TcpError::NoLaserStatus)
    }
}

/// Reads a laser type from a stream and returns a `Result` with the `LaserType`
/// or a `TcpError`. Looks for the `LASER_ID` and the `TERMINATOR` in the stream.
/// 
/// # Example
/// ```rust
/// use coherent_rs::laser::LaserType;
/// use coherent_rs::network::{LASER_ID, deserialize_laser_type, TERMINATOR};
/// use serde::Serialize;
/// use rmp_serde::Serializer;
/// 
/// let tp = LaserType::DebugLaser;
/// 
/// let mut buf = Vec::new();
/// buf.extend(LASER_ID);
/// tp.serialize(&mut Serializer::new(&mut buf)).unwrap();
/// buf.extend(TERMINATOR);
/// 
/// let laser_type = deserialize_laser_type(&buf).unwrap();
/// 
/// assert_eq!(laser_type, LaserType::DebugLaser);
/// 
/// ```
fn deserialize_laser_type(stream : &[u8]) -> Result<LaserType, TcpError> {
    if let Some(start_idx) = stream.windows(LASER_ID.len()).position(
        |window| window == LASER_ID
    ){
        let laser_type = &stream[start_idx + LASER_ID.len()..];
        if let Some(end) = laser_type.split(|&x| x == TERMINATOR[0]).next() {
            let serialized = &laser_type[..end.len()];
            LaserType::deserialize(
                &mut rmp_serde::Deserializer::new(serialized)
            ).map_err(|e| TcpError::SerializationDecodeError(e))
        }
        else {
            Err(TcpError::NoLaserStatus)
        }
    }
    else {
        Err(TcpError::NoLaserStatus)
    }
}


/// Create a network listener that listens on the specified port.
/// Takes ownership over the `Laser` so that it can be polled and
/// shared between threads, and maintains exclusive access through
/// a `Mutex`.
/// 
/// Default polling interval is 1 second.
pub fn create_listener<L : Laser + 'static>(laser : L, port : &str, polling_interval : Option<f32>) -> Result<NetworkLaser<L>, TcpError> {
    NetworkLaser::new(laser, port, polling_interval)
}

impl<L : Laser + 'static> Clone for NetworkLaser<L> {
    fn clone(&self) -> Self {
        NetworkLaser {
            _listener : self._listener.try_clone().unwrap(),
            _laser : self._laser.clone(),
            _polling_interval : self._polling_interval.clone(),
            _polling_thread : None,
            _polling : Arc::new(AtomicBool::new(false)),
            _clients : Arc::new(Mutex::new(Vec::new())),
            _client_connection_thread : None,
            _command_thread : None,
            _primary_client : self._primary_client.clone(),
        }
    }
}

impl<L : Laser + 'static> NetworkLaser<L> {


    /// Create a network listener that listens on the specified port.
    /// Takes ownership over the `Laser` so that it can be polled and
    /// shared between threads, and maintains exclusive access through
    /// a `Mutex`.
    /// 
    /// Default polling interval is 1 second.
    /// 
    /// # Arguments
    /// 
    /// * `laser` - The laser to control over the network.
    /// * `port` - The port to listen on.
    /// * `polling_interval` - The interval in seconds to poll the laser.
    pub fn new(laser : L, port : &str, polling_interval : Option<f32>) -> Result<Self, TcpError> {
        let listener = TcpListener::bind(port)
        .map_err(|e| TcpError::IoError(e))?;

        let nl = NetworkLaser {
            _listener : listener,
            _laser : Arc::new(Mutex::new(laser)),
            _polling_interval : Arc::new(Mutex::new(polling_interval.unwrap_or(1.0))),
            _polling_thread : None,
            _polling : Arc::new(AtomicBool::new(false)),
            _clients : Arc::new(Mutex::new(Vec::new())),
            _client_connection_thread : None,
            _command_thread : None,
            _primary_client : None,
        };

        Ok(nl)
    }

    /// Returns the name of the port that the listener is listening on.
    pub fn get_port(&self) -> String {
        self._listener.local_addr().unwrap().port().to_string()
    }

    /// Sets the polling interval in seconds
    pub fn set_polling_interval(&mut self, interval : f32) {
        let mut polling_interval = self._polling_interval.lock().unwrap();
        *polling_interval = interval;
    }

    // /// Returns the laser and kills the `NetworkLaser`
    // /// TODO implement correctly -- the `drop` method gets in the way
    // /// of prematurely killing the `Arc`.
    // pub fn get_laser(self) -> Result<L, TcpError> {
    //     let laser = Arc::into_inner(self._laser)
    //         .ok_or(TcpError::MultipleReferencesToLaser)
    //         .and_then(
    //             |mutex| 
    //             mutex.into_inner().map_err(|_| TcpError::MultipleReferencesToLaser)
    //         )?;
    //     drop(self);
    //     Ok(laser)
    // }

    /// Initializes the polling thread. Does nothing if already listening for connections.
    pub fn poll(&mut self) -> Result<(), TcpError> {
        if self._polling_thread.is_some() {
            return Ok(())
        }

        let _listener = self._listener.try_clone().map_err(|e| TcpError::IoError(e))?;
        _listener.set_nonblocking(true).map_err(|e| TcpError::IoError(e))?;

        self._polling.store(true, std::sync::atomic::Ordering::SeqCst);
        let _polling = self._polling.clone();
        let _clients = Arc::clone(&self._clients);
        
        // Looks for new clients, identifies the type of laser and sends the status.
        self._client_connection_thread = Some(std::thread::spawn( move || {
            while _polling.load(std::sync::atomic::Ordering::SeqCst) {
                for stream in _listener.incoming() {
                    match stream {
                        Ok(mut stream) => {
                            let mut self_id = LASER_ID.to_vec();
                            L::into_laser_type().serialize(
                                &mut Serializer::new(&mut self_id))
                                .map_err(|e| TcpError::SerializationEncodeError(e)).unwrap();
                            self_id.extend(TERMINATOR);
                            stream.write_all(&self_id).unwrap();
                            let mut clients = _clients.lock().unwrap();
                            clients.push(stream);
                        },
                        Err(e) => {}   
                    }
                    if !_polling.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                }   
            }
        }));

        let _polling_interval = self._polling_interval.clone();
        let _laser = self._laser.clone();
        let _polling = self._polling.clone();
        let clients = Arc::clone(&self._clients);
        
        // Polls the laser, passes it to all the clients.
        self._polling_thread = Some(std::thread::spawn( move || {
            while _polling.load(std::sync::atomic::Ordering::SeqCst) { 
                {
                    let mut laser_lock = _laser.lock().unwrap();
                    let serialized = laser_lock.serialized_status().unwrap();
                    let mut clients = clients.lock().unwrap();
                    clients.retain(|mut client| {
                        // Write all in one line
                        let mut to_write = STATUS_MARKER.to_vec();
                        to_write.extend(serialized.clone());
                        to_write.extend(TERMINATOR);
                        client.write_all(&to_write).is_ok()
                    });
                    
                }
                
                std::thread::sleep(std::time::Duration::from_millis(
                    (*_polling_interval.lock().unwrap() * 1000.0) as u64
                ));
            }
        }));

        // Investigates the clients for commands, deserializes them, then executes
        // them on the laser.

        let _command_interval_ms = 50; //milliseconds
        let _laser = Arc::clone(&self._laser);
        let _clients = Arc::clone(&self._clients);
        let _polling = self._polling.clone();
        let _primary_client = self._primary_client.clone();

        self._command_thread = Some(std::thread::spawn( move || {
            while _polling.load(std::sync::atomic::Ordering::SeqCst) {
                let mut clients = _clients.lock().unwrap();
                for client in clients.iter_mut() {
                    // if let Some(ref primary_client) = _primary_client {
                    //     if !Arc::ptr_eq(primary_client, client) {
                    //         continue;
                    //     }
                    // }
                    let mut buf = [0u8; 1024];
                    match client.read(&mut buf) {
                        Ok(_) => {
                            // If a command is in the buffer, execute it.
                            if let Ok(command) = deserialize_command::<L>(&buf) {
                                println!{"Received command : {:?}", command};
                                let mut laser = _laser.lock().unwrap();
                                match laser.send_command(command) {
                                    Ok(_) => {
                                        client.write_all(COMMAND_SUCCESSFUL).unwrap();
                                    },
                                    Err(e) => {
                                        client.write_all(COMMAND_FAILED).unwrap();
                                    }
                                }
                            }
                        },
                        Err(e) => {}
                    }
                };
                // sleep prevents over-locking the mutexes
                std::thread::sleep(std::time::Duration::from_millis(_command_interval_ms));   
            }
        }));

        Ok(())
    }

    pub fn stop_polling(&mut self){
        if self._polling_thread.is_none() {
            return;
        }
        self._polling.store(false, std::sync::atomic::Ordering::SeqCst);
        match self._client_connection_thread.take() {
            Some(thread) => thread.join().unwrap(),
            None => {}
        }
        match self._polling_thread.take() {
            Some(thread) => thread.join().unwrap(),
            None => {}
        }
        match self._command_thread.take() {
            Some(thread) => thread.join().unwrap(),
            None => {}
        }
    }

    /// Returns whether the poll thread is polling
    pub fn polling(&self) -> bool {
        self._polling.load(std::sync::atomic::Ordering::SeqCst)
    }

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

impl<L : Laser + 'static> Drop for NetworkLaser<L> {
    fn drop(&mut self) {
        self.stop_polling();
    }
}

/// A trait for a network interface to a laser. The laser type is determined
/// by the `Laser` type parameter. Individual structs that implement this trait
/// can also implement `Laser`-specific methods. The actual implementation of the
/// network connection is left to the implementing struct.
pub trait NetworkLaserInterface<L : Laser> : Sized {
    
    /// Must be implemented for each struct -- defined how to
    /// connect to the laser over the network.
    fn connect(port : &str) -> Result<Self, TcpError>;
    
    /// Access the underlying `TcpStream`
    fn access_stream(&mut self) -> &TcpStream;
    
    /// Access a laser type parameter
    fn get_laser_type(&self) -> LaserType {L::into_laser_type()}
    
    /// Generically sends a command to the laser over the network. Blocks
    /// until it receives confirmation that the command was sent or failed.
    fn command(&mut self, command : L::CommandEnum) -> Result<(), TcpError> {

        // self.access_stream().flush().map_err(|e| TcpError::IoError(e))?;
        
        let mut buf = Vec::new();
        buf.extend(COMMAND_MARKER);
        command.serialize(&mut Serializer::new(&mut buf))
            .map_err(|e| TcpError::SerializationEncodeError(e))?;
        buf.extend(TERMINATOR);
        self.access_stream().write_all(buf.as_slice())
            .map_err(|e| TcpError::IoError(e))?;

        // Wait for command evaluation
        let mut response = [0u8; 1024];
        loop {
            match self.access_stream().read(&mut response) {
                Ok(_) => {
                    if response.starts_with(COMMAND_SUCCESSFUL) {
                        return Ok(());
                    }
                    else if response.starts_with(COMMAND_FAILED) {
                        return Err(TcpError::CommandError);
                    }
                    else if response.starts_with(NOT_PRIMARY_CLIENT) {
                        return Err(TcpError::NotPrimaryClient);
                    }
                },
                Err(e) => { // stream is dead, or I/O error occurred
                    return Err(TcpError::IoError(e));
                }
            }
        }

    
    }
    
    /// Returns a full status of the laser from the network. Warning: blocking!
    fn query_status(&mut self) -> Result<L::LaserStatus, TcpError>{
        let mut buf = [0u8; 1024]; // Fixed-size buffer for reading from the stream
        let mut data = Vec::new(); // Accumulated data

        // self.access_stream().flush().map_err(|e| TcpError::IoError(e))?;

        loop {
            // Attempt to deserialize the current data
            if let Ok(status) = deserialize_laser_status::<L>(&data) {
                return Ok(status);
            }

            // Read more data from the stream
            match self.access_stream().read(&mut buf) {
                Ok(n) => {
                    // Append the new data to the accumulated buffer
                    data.extend_from_slice(&buf[..n]);
                }
                Err(e) => {
                    // Handle I/O errors
                    return Err(TcpError::IoError(e));
                }
            }
        }
    }
}

/// A struct to generically connect to and communicate with a
/// `NetworkLaser` over the network. Doesn't have any unique functionality
/// or ability to query specific details, but can be used to send commands
/// or get the full status of the laser.
pub struct BasicNetworkLaserInterface<L : Laser>{
    _stream : TcpStream,
    _laser : PhantomData<L>,
}

impl<L : Laser> NetworkLaserInterface<L> for  BasicNetworkLaserInterface<L> {
    /// Connect to a `NetworkLaser` over the network, if it exists
    /// 
    /// # Example
    /// ```rust
    /// use coherent_rs::{Discovery, create_listener, NetworkLaserInterface};
    /// ```
    fn connect(port : &str) -> Result<Self, TcpError> {
        let mut stream = TcpStream::connect(port)
            .map_err(|e| TcpError::IoError(e))?;

        let mut state_stream_buf = [0u8; 1024];
        while 0 == stream.read(&mut state_stream_buf)
            .map_err(|e| TcpError::IoError(e))?{};

        let laser_type = deserialize_laser_type(&state_stream_buf).unwrap();

        if !(laser_type == L::into_laser_type()) {
            return Err(TcpError::CoherentError(CoherentError::UnrecognizedDevice))
        }

        Ok(
            BasicNetworkLaserInterface::<L> {
                _stream : stream,
                _laser : PhantomData
            }
        )
    }

    /// Allows access to the underlying `TcpStream`
    fn access_stream(&mut self) -> &TcpStream {
        &self._stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::laser::{Discovery, DiscoveryNXCommands, DiscoveryLaser};
    use crate::laser::debug::DebugLaser;

    /// Tests that don't rely on a laser being connected.
    #[test]
    fn test_serial_comm_threading() {

    }

    #[test]
    fn test_deserialize_laser_type(){
        use crate::laser::LaserType;
        let tp = LaserType::DebugLaser;
        
        let mut buf = Vec::new();
        buf.extend(LASER_ID);
        tp.serialize(&mut Serializer::new(&mut buf)).unwrap();
        buf.extend(TERMINATOR);

        let laser_type = deserialize_laser_type(&buf).unwrap();

        assert_eq!(laser_type, LaserType::DebugLaser);
    }

    #[test]
    fn test_deserialize_laser_status(){
        use crate::laser::{Laser, debug::DebugLaser};
        use crate::network::{STATUS_MARKER, deserialize_laser_status, TERMINATOR};

        let mut laser = DebugLaser::default();
        let status_serialized = laser.serialized_status().unwrap();

        let mut sent_message = STATUS_MARKER.to_vec();
        sent_message.extend(status_serialized);
        sent_message.extend(TERMINATOR);

        let status = deserialize_laser_status::<DebugLaser>(&sent_message).unwrap();
        println!{"Deserialized : {:?}", status};
    }

    #[test]
    fn get_laser() {
        let mut discovery = DebugLaser::find_first().unwrap();
        let mut network_laser = NetworkLaser::new(discovery, "127.0.0.1:907", None);
        // let laser_again = network_laser.unwrap().get_laser().unwrap();
        // println!("{:?}", laser_again);
    }

    /// Simple tests of whether the laser control stuff still functions while
    /// listening on a network port.
    #[test]
    fn test_network_laser() {
        use crate::network::NetworkLaserInterface;
        // let mut discovery = Discovery::find_first().unwrap();
        let discovery = DebugLaser::find_first().unwrap();

        let mut network_laser = NetworkLaser::new(
            discovery, "127.0.0.1:907", 
            Some(0.5),
            // None
            ).unwrap();

        network_laser.poll().unwrap();

        network_laser.command(
            DiscoveryNXCommands::Shutter{laser : DiscoveryLaser::VariableWavelength, state : false.into()}
        ).unwrap();

        println!("Is it polling? {}! Listening on port {}", network_laser.polling(), network_laser.get_port());

        let mut my_interface = BasicNetworkLaserInterface::<DebugLaser>::connect("127.0.0.1:907").unwrap();
        println!{"Connected to laser of type {:?}", my_interface.get_laser_type()};

        // print how long the query takes
        let start = std::time::Instant::now();
        let read_status = my_interface.query_status().unwrap();
        println!{"Query took {:?}", start.elapsed()};
        assert_eq!(read_status.variable_shutter, false.into());

        // spam the laser!
        // for i in 0..50 {
        //     std::thread::sleep(std::time::Duration::from_millis(1));
        //     my_interface.command(
        //         DiscoveryNXCommands::Shutter{laser : DiscoveryLaser::VariableWavelength, state : false.into()}
        //     ).unwrap();
        // }

        let mut second_interface = BasicNetworkLaserInterface::<DebugLaser>::connect("127.0.0.1:907").unwrap();

        //print how long the command takes
        let start = std::time::Instant::now();
        second_interface.command(
            DiscoveryNXCommands::Shutter{laser : DiscoveryLaser::VariableWavelength, state : true.into()}
        ).unwrap();
        println!{"Command took {:?}", start.elapsed()};

        let start = std::time::Instant::now();
        my_interface.command(
            DiscoveryNXCommands::Shutter{laser : DiscoveryLaser::VariableWavelength, state : true.into()}
        ).unwrap();
        println!{"Command took {:?}", start.elapsed()};

        std::thread::sleep(std::time::Duration::from_secs(1));

        let start = std::time::Instant::now();
        let read_status = my_interface.query_status().unwrap();
        println!{"Query took {:?}", start.elapsed()};
        println!{"Read status : {:?}", read_status};
        assert_eq!(read_status.variable_shutter, true.into());

        network_laser.command(
            DiscoveryNXCommands::Shutter{laser : DiscoveryLaser::VariableWavelength, state : false.into()}
        ).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(500));

        let second_status = second_interface.query_status().unwrap();
        println!{"Second status : {:?}", second_status};
        assert_eq!(second_status.variable_shutter, false.into());

        network_laser.command(
            DiscoveryNXCommands::Shutter{laser : DiscoveryLaser::VariableWavelength, state : false.into()}
        ).unwrap();
        println!("About to stop polling");

        network_laser.stop_polling();

        println!("Stopped polling");
    }
}