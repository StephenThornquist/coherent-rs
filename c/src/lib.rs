//! Thin C ABI layer for the `coherent_rs` crate

use coherent_rs::{laser, Discovery, laser::Laser};

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