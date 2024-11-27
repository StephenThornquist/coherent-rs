//! Thin C ABI layer for the `coherent_rs` crate

use coherent_rs::{laser, Discovery, laser::Laser};
use coherent_rs::{DiscoveryNXCommands, discoverynx::DiscoveryLaser};
#[cfg(feature="network")]
use coherent_rs::network::{BasicNetworkLaserClient, NetworkLaserClient, TcpError};

/// C ABI
#[no_mangle]
pub unsafe extern "C" fn discovery_find_first() -> *mut Discovery {
    match Discovery::find_first() {
        Ok(discovery) => Box::into_raw(Box::new(discovery)),
        Err(_) => std::ptr::null_mut()
    }
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
    match Discovery::from_port_name(port_name) {
        Ok(discovery) => Box::into_raw(Box::new(discovery)),
        Err(_) => std::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn discovery_by_serial_number(serial_number : *const u8, serial_number_len : usize) -> *mut Discovery {
    let serial_number = unsafe {
        std::str::from_utf8(std::slice::from_raw_parts(serial_number, serial_number_len)).unwrap()
    };
    match Discovery::new(None, Some(serial_number)) {
        Ok(discovery) => Box::into_raw(Box::new(discovery)),
        Err(_) => std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn discovery_set_wavelength(discovery : *mut Discovery, wavelength : f32) -> i32 {
    unsafe {match discovery.as_mut().unwrap().set_wavelength(wavelength) {
        Ok(()) => 0,
        Err(_) => -1,
    }}
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
pub extern "C" fn discovery_set_gdd(discovery : *mut Discovery, gdd : f32) -> i32 {
    unsafe {match (*discovery).set_gdd(gdd) {
        Ok(()) => 0,
        Err(_) => -1,
    }}
}

#[no_mangle]
pub extern "C" fn discovery_get_gdd(discovery : *mut Discovery) -> f32 {
    unsafe {(*discovery).get_gdd().unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_set_alignment_variable(discovery : *mut Discovery, alignment : bool) -> i32 {
    unsafe {match (*discovery).set_alignment_mode(laser::DiscoveryLaser::VariableWavelength, alignment) {
        Ok(()) => 0,
        Err(_) => -1,
    }}
}

#[no_mangle]
pub extern "C" fn discovery_get_alignment_variable(discovery : *mut Discovery) -> bool {
    unsafe {(*discovery).get_alignment_mode(laser::DiscoveryLaser::VariableWavelength).unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_set_alignment_fixed(discovery : *mut Discovery, alignment : bool) -> i32 {
    unsafe {match (*discovery).set_alignment_mode(laser::DiscoveryLaser::FixedWavelength, alignment) {
        Ok(()) => 0,
        Err(_) => -1,
    }}
}

#[no_mangle]
pub extern "C" fn discovery_get_alignment_fixed(discovery : *mut Discovery) -> bool {
    unsafe {(*discovery).get_alignment_mode(laser::DiscoveryLaser::FixedWavelength).unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_get_status_string(discovery : *mut Discovery, status : *mut u8, status_len : *mut usize) -> () {
    unsafe {
        let status_string = (*discovery).get_status().unwrap();
        let status_string = status_string.as_bytes();
        let status_string_len = status_string.len();
        std::ptr::copy_nonoverlapping(status_string.as_ptr(), status, status_string_len);
        *status_len = status_string_len;
    }
}

#[no_mangle]
pub extern "C" fn discovery_get_tuning(discovery : *mut Discovery) -> bool {
    unsafe { match (*discovery).get_tuning().unwrap() {
        laser::TuningStatus::Tuning => true,
        laser::TuningStatus::Ready => false,
    }}
}

#[no_mangle]
pub extern "C" fn discovery_set_shutter_variable(discovery : *mut Discovery, state : bool) -> i32 {
    unsafe {match (*discovery).set_shutter(laser::DiscoveryLaser::VariableWavelength, if state {laser::ShutterState::Open} else {laser::ShutterState::Closed}) {
        Ok(()) => 0,
        Err(_) => -1,
    }}
}

#[no_mangle]
pub extern "C" fn discovery_get_shutter_variable(discovery : *mut Discovery) -> bool {
    unsafe {(*discovery).get_shutter(laser::DiscoveryLaser::VariableWavelength).unwrap() == laser::ShutterState::Open}
}

#[no_mangle]
pub extern "C" fn discovery_set_shutter_fixed(discovery : *mut Discovery, state : bool) -> i32 {
    unsafe {match (*discovery).set_shutter(laser::DiscoveryLaser::FixedWavelength, if state {laser::ShutterState::Open} else {laser::ShutterState::Closed}) {
        Ok(()) => 0,
        Err(_) => -1,
    }}
}

#[no_mangle]
pub extern "C" fn discovery_get_shutter_fixed(discovery : *mut Discovery) -> bool {
    unsafe {(*discovery).get_shutter(laser::DiscoveryLaser::FixedWavelength).unwrap() == laser::ShutterState::Open}
}

#[no_mangle]
pub extern "C" fn discovery_set_laser_to_standby(discovery : *mut Discovery, state : bool) -> i32 {
    unsafe {match (*discovery).set_to_standby(state) {
        Ok(()) => 0,
        Err(_) => -1,
    }}
}

#[no_mangle]
pub extern "C" fn discovery_get_laser_standby(discovery : *mut Discovery) -> bool {
    unsafe {match (*discovery).get_standby().unwrap()
    {
        laser::LaserState::Standby => true,
        laser::LaserState::On => false,
    }}
}

#[no_mangle]
pub extern "C" fn discovery_get_keyswitch(discovery : *mut Discovery) -> bool {
    unsafe {(*discovery).get_keyswitch_on().unwrap()}
}

#[no_mangle]
pub extern "C" fn discovery_get_serial(discovery : *mut Discovery, serial: *mut u8, serial_len : *mut usize) -> () {
    unsafe {
        let serial_number = (*discovery).get_serial().unwrap();
        let serial_number = serial_number.as_bytes();
        let serial_number_len = serial_number.len();
        std::ptr::copy_nonoverlapping(serial_number.as_ptr(), serial, serial_number_len);
        *serial_len = serial_number_len;
    }
}

#[no_mangle]
pub extern "C" fn discovery_get_status(discovery : *mut Discovery, status: *mut u8, status_len : *mut usize) {
    unsafe {
        let status_string = (*discovery).get_status().unwrap();
        let status_string = status_string.as_bytes();
        let status_string_len = status_string.len();
        std::ptr::copy_nonoverlapping(status_string.as_ptr(), status, status_string_len);
        *status_len = status_string_len;
    }
}

#[no_mangle]
pub extern "C" fn discovery_get_fault_text(discovery : *mut Discovery, error: *mut u8, error_len : *mut usize) {
    unsafe {
        let error_string = (*discovery).get_fault_text().unwrap();
        let error_string = error_string.as_bytes();
        let error_string_len = error_string.len();
        std::ptr::copy_nonoverlapping(error_string.as_ptr(), error, error_string_len);
        *error_len = error_string_len;
    }
}

#[no_mangle]
pub extern "C" fn discovery_clear_faults(discovery : *mut Discovery) -> i32 {
    unsafe {match (*discovery).clear_faults() {
        Ok(()) => 0,
        Err(_) => -1,
    }}
}

//////////
//
// NETWORK FUNCTIONS
//
//////////

#[cfg(feature="network")]
#[no_mangle]
/// Returns a pointer to a `NetworkLaserServer` object,
/// or `std::ptr::null_mut()` if the server could not be created.
pub extern "C" fn connect_discovery_client(port : *const u8, port_len : usize) -> *mut BasicNetworkLaserClient<Discovery> {
    let port = unsafe {
        std::str::from_utf8(std::slice::from_raw_parts(port, port_len)).unwrap()
    };

    match BasicNetworkLaserClient::connect(port) {
        Ok(client) => Box::into_raw(Box::new(client)),
        Err(_) => std::ptr::null_mut()
    }
}

#[cfg(feature = "network")]
#[no_mangle]
pub extern "C" fn free_discovery_client(client : *mut BasicNetworkLaserClient<Discovery>) {
    if client.is_null() {return}
    drop(unsafe {Box::from_raw(client)});
}

#[cfg(feature = "network")]
#[no_mangle]
pub extern "C" fn set_discovery_client_variable_shutter(
    client : *mut BasicNetworkLaserClient<Discovery>,
    state : bool
) -> i32 {
    match unsafe {(*client).command(
            DiscoveryNXCommands::Shutter{
                laser : DiscoveryLaser::VariableWavelength,
                state : if state {laser::ShutterState::Open} else {laser::ShutterState::Closed}
            }
    )} {
        Ok(()) => 0,
        Err(TcpError::NotPrimaryClient) => -2,
        Err(_) => -1,
    }
}

#[cfg(feature = "network")]
#[no_mangle]
pub extern "C" fn set_discovery_client_fixed_shutter(
    client : *mut BasicNetworkLaserClient<Discovery>,
    state : bool
) -> i32 {
    match unsafe {(*client).command(
            DiscoveryNXCommands::Shutter{
                laser : DiscoveryLaser::FixedWavelength,
                state : if state {laser::ShutterState::Open} else {laser::ShutterState::Closed}
            }
    )} {
        Ok(()) => 0,
        Err(TcpError::NotPrimaryClient) => -2,
        Err(_) => -1,
    }
}

#[cfg(feature = "network")]
#[no_mangle]
pub extern "C" fn set_discovery_client_wavelength(
    client : *mut BasicNetworkLaserClient<Discovery>,
    wavelength : f32,
) -> i32 {
    match unsafe {(*client).command(DiscoveryNXCommands::Wavelength{wavelength_nm : wavelength})} {
        Ok(()) => 0,
        Err(TcpError::NotPrimaryClient) => -2,
        Err(_) => -1,
    }
}

#[cfg(feature = "network")]
#[no_mangle]
pub extern "C" fn set_discovery_client_to_standby(
    client : *mut BasicNetworkLaserClient<Discovery>,
    to_standby : bool
) -> i32 {
    match unsafe {(*client).command(DiscoveryNXCommands::Laser { state: 
        if to_standby {laser::LaserState::Standby} else {laser::LaserState::On}})} {
        Ok(()) => 0,
        Err(TcpError::NotPrimaryClient) => -2,
        Err(_) => -1,
    }
}

#[cfg(feature = "network")]
#[no_mangle]
pub extern "C" fn set_discovery_client_variable_alignment(
    client : *mut BasicNetworkLaserClient<Discovery>,
    alignment : bool
) -> i32 {
    match unsafe {(*client).command(DiscoveryNXCommands::AlignmentMode{
        laser : DiscoveryLaser::VariableWavelength,
        alignment_mode_on : alignment
    })} {
        Ok(()) => 0,
        Err(TcpError::NotPrimaryClient) => -2,
        Err(_) => -1,
    }
}

#[cfg(feature = "network")]
#[no_mangle]
pub extern "C" fn set_discovery_client_fixed_alignment(
    client : *mut BasicNetworkLaserClient<Discovery>,
    alignment : bool
) -> i32 {
    match unsafe {(*client).command(DiscoveryNXCommands::AlignmentMode{
        laser : DiscoveryLaser::FixedWavelength,
        alignment_mode_on : alignment
    })} {
        Ok(()) => 0,
        Err(TcpError::NotPrimaryClient) => -2,
        Err(_) => -1,
    }
}

#[cfg(feature = "network")]
#[no_mangle]
pub extern "C" fn set_discovery_client_gdd(
    client : *mut BasicNetworkLaserClient<Discovery>,
    gdd : f32
) -> i32 {
    match unsafe {(*client).command(DiscoveryNXCommands::Gdd{gdd_val : gdd})}{
        Ok(()) => 0,
        Err(TcpError::NotPrimaryClient) => -2,
        Err(_) => -1,
    }
}

#[cfg(feature = "network")]
#[no_mangle]
pub extern "C" fn set_discovery_client_gdd_curve(
    client : *mut BasicNetworkLaserClient<Discovery>,
    curve : i32
) -> i32 {
    if curve > 255 || curve < 0 {
        return -1;
    }
    match unsafe {(*client).command(DiscoveryNXCommands::GddCurve {curve_num : curve as u8})} {
        Ok(()) => 0,
        Err(TcpError::NotPrimaryClient) => -2,
        Err(_) => -1,
    }
}

#[cfg(feature = "network")]
#[no_mangle]
pub extern "C" fn demand_primary_client(
    client : *mut BasicNetworkLaserClient<Discovery>
) -> i32 {
    match unsafe {(*client).demand_primary_client()} {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[cfg(feature = "network")]
#[no_mangle]
pub extern "C" fn release_primary_client(
    client : *mut BasicNetworkLaserClient<Discovery>
) -> i32 {
    match unsafe {(*client).forget_me()} {
        Ok(()) => 0,
        Err(_) => -1,
    }
}



#[cfg(feature = "network")]
#[repr(C)]
pub struct CDiscoveryStatus {
    echo : bool,
    laser : bool,
    variable_shutter : bool,
    fixed_shutter : bool,
    keyswitch : bool,
    faults : u8,
    fault_text : Box<String>,
    fault_text_len : usize,
    tuning : bool,
    alignment_var : bool,
    alignment_fixed : bool,
    status : Box<String>,
    status_len : usize,
    wavelength : f32,
    power_var : f32,
    power_fixed : f32,
    gdd_curve : i32,
    gdd_curve_n : Box<String>,
    gdd_curve_n_len : usize,
    gdd : f32,
}

#[cfg(feature = "network")]
fn discovery_status_to_csafe(status : <Discovery as Laser>::LaserStatus) -> CDiscoveryStatus {
    CDiscoveryStatus{
        echo : status.echo,
        laser : if status.laser == laser::LaserState::On {true} else {false},
        variable_shutter : if status.variable_shutter == laser::ShutterState::Open {true} else {false},
        fixed_shutter : if status.fixed_shutter == laser::ShutterState::Open {true} else {false},
        keyswitch : status.keyswitch,
        faults : status.faults,
        fault_text : Box::new(status.fault_text.clone()),
        fault_text_len : status.fault_text.len(),
        tuning : if status.tuning == laser::TuningStatus::Tuning {true} else {false},
        alignment_var : status.alignment_var,
        alignment_fixed : status.alignment_fixed,
        status : Box::new(status.status.clone()),
        status_len : status.status.len(),
        wavelength : status.wavelength,
        power_var : status.power_var,
        power_fixed : status.power_fixed,
        gdd_curve : status.gdd_curve,
        gdd_curve_n : Box::new(status.gdd_curve_n.clone()),
        gdd_curve_n_len : status.gdd_curve_n.len(),
        gdd : status.gdd,
    }
}

#[cfg(feature = "network")]
#[no_mangle]
pub extern "C" fn discovery_client_query_status(client : *mut BasicNetworkLaserClient<Discovery>)
 -> CDiscoveryStatus {
    match unsafe {(*client).query_status()} {
        Ok(status) => discovery_status_to_csafe(status),
        Err(_) => panic!("Error querying status")
    }
}

