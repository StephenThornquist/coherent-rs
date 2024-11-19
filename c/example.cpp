/*
Example code to demonstrate the FFI of the Rust code with C++
*/
#include "discovery.h"
#include <iostream>
#include <chrono>
#include <thread>

int main() {
    Discovery discovery = discovery_find_first();
    if (discovery == nullptr) {
        return 1;
    }

    std::cout << "Device found!" << std::endl;
    char* serial = new char[256];
    size_t *serial_len = new size_t;
    discovery_get_serial(discovery, serial, serial_len);
    // Print only the `serial_len` characters of the serial number
    std::cout << "Serial: "; std::cout.write(serial, *serial_len); std::cout << std::endl;

    discovery_get_status(discovery, serial, serial_len);
    std::cout << "Status: "; std::cout.write(serial, *serial_len); std::cout << std::endl;

    discovery_get_fault_text(discovery, serial, serial_len);
    std::cout << "Fault Text: " << serial << std::endl;

    delete[] serial;
    delete serial_len;

    float wavelength = discovery_get_wavelength(discovery);
    float power_variable = discovery_get_power_variable(discovery);
    float power_fixed = discovery_get_power_fixed(discovery);
    float gdd = discovery_get_gdd(discovery);
    bool alignment_variable = discovery_get_alignment_variable(discovery);
    bool alignment_fixed = discovery_get_alignment_fixed(discovery);

    std::cout << "Wavelength: " << wavelength << " nm" << std::endl;
    std::cout << "Variable Power: " << power_variable << " mW" << std::endl;
    std::cout << "Fixed Power: " << power_fixed << " mW" << std::endl;
    std::cout << "GDD: -" << gdd << " fs^2" << std::endl;
    std::cout << "Variable Alignment: " << alignment_variable << std::endl;
    std::cout << "Fixed Alignment: " << alignment_fixed << std::endl;

    discovery_set_wavelength(discovery, 800.0);
    std::cout << "New wavelength: " << discovery_get_wavelength(discovery) << " nm" << std::endl;

    while (discovery_get_tuning(discovery)) {
        std::cout << "Tuning..." << std::endl;
        std::this_thread::sleep_for(std::chrono::milliseconds(500));
    }

    std::cout << "New power: " << discovery_get_power_variable(discovery) << " mW" << std::endl;


    discovery_set_wavelength(discovery, wavelength);
    std::cout << "Restoring to: " << discovery_get_wavelength(discovery) << " nm" << std::endl;

    while (discovery_get_tuning(discovery)) {
        std::cout << "Tuning..." << std::endl;
        std::this_thread::sleep_for(std::chrono::milliseconds(500));
    }

    std::cout << "Restored power: " << discovery_get_power_variable(discovery) << " mW" << std::endl;

    std::cout << "Opening variable shutter..." << std::endl;
    discovery_set_shutter_variable(discovery, true);

    std::cout << "Variable shutter open: " << discovery_get_shutter_variable(discovery) << std::endl;
    std::this_thread::sleep_for(std::chrono::milliseconds(300));
    std::cout << "Closing variable shutter..." << std::endl;
    discovery_set_shutter_variable(discovery, false);

    std::cout << "Variable shutter open: " << discovery_get_shutter_variable(discovery) << std::endl;

    int error_code = discovery_set_wavelength(discovery, 2.0);
    std::cout << "Trying to set wavelength to 2.0 nm results in error code: " << error_code << std::endl;

    free_discovery(discovery);
    return 0;
}