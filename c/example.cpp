/*
Example code to demonstrate the FFI of the Rust code with C++
*/
#include "discovery.h"
#include <iostream>

int main() {
    Discovery discovery = discovery_find_first();
    if (discovery == nullptr) {
        return 1;
    }
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
    discovery_set_wavelength(discovery, wavelength);
    std::cout << "Restored to: " << discovery_get_wavelength(discovery) << " nm" << std::endl;

    int error_code = discovery_set_wavelength(discovery, 2.0);
    std::cout << "Trying to set wavelength to 2.0 nm results in error code: " << error_code << std::endl;

    free_discovery(discovery);
    return 0;
}