#ifndef COHERENT_RS_DISCOVERY_HPP
#define COHERENT_RS_DISCOVERY_HPP
#include<cstddef>

typedef void* Discovery;
extern "C" {
    // If unable to find a device, returns nullptr
    Discovery discovery_find_first();
    // If unable to find a device, returns nullptr
    Discovery discovery_by_port_name(const char* port_name, size_t port_name_len);
    // If unable to find a device, returns nullptr
    Discovery discovery_by_serial_number(const char* serial_number, size_t serial_number_len);
    // Must be called to avoid leaks!
    void free_discovery(Discovery discovery);

    /**
     * Set the wavelength of the variable-wavelength laser. Returns
     * 0 if successful, -1 if the wavelength is out of bounds.
     */
    int discovery_set_wavelength(Discovery discovery, float wavelength);
    float discovery_get_wavelength(Discovery discovery);

    float discovery_get_power_variable(Discovery discovery);
    float discovery_get_power_fixed(Discovery discovery);

    /**
     * Set the GDD of the laser. Returns 0 if successful, -1 if the GDD is out of bounds.
     */
    int discovery_set_gdd(Discovery discovery, float gdd);
    float discovery_get_gdd(Discovery discovery);

    /**
     * Set the alignment mode of the variable-wavelength laser. Returns 0 if successful, -1 if an error occurred.
     */
    int discovery_set_alignment_variable(Discovery discovery, bool alignment_variable);
    bool discovery_get_alignment_variable(Discovery discovery);

    /**
     * Set the alignment mode of the fixed-wavelength laser. Returns 0 if successful, -1 if an error occurred.
     */
    int discovery_set_alignment_fixed(Discovery discovery, bool alignment_fixed);
    bool discovery_get_alignment_fixed(Discovery discovery);
    
}

#endif // COHERENT_RS_DISCOVERY_HPP