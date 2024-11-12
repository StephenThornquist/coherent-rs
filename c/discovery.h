#ifndef COHERENT_RS_DISCOVERY_HPP
#define COHERENT_RS_DISCOVERY_HPP


// Cross-platform bootstrap
#if defined(_WIN32) || defined(_WIN64)
#	ifdef __GNUC__
#		define API_EXPORT __attribute__ ((dllexport))
#		define API_IMPORT __attribute__ ((dllimport))
#	else
#		define API_EXPORT __declspec(dllexport)
#		define API_IMPORT __declspec(dllimport)
#	endif
#       define API_STATIC
#else
#	ifdef __GNUC__
#		define API_EXPORT __attribute__((visibility ("default")))
#		define API_IMPORT __attribute__((visibility ("default")))
#	else
#		define API_EXPORT
#		define API_IMPORT
#	endif
#   define API_STATIC
#endif


#include<cstddef>

typedef void* Discovery;
extern "C" {
    // If unable to find a device, returns nullptr
    API_IMPORT Discovery discovery_find_first();
    // If unable to find a device, returns nullptr
    API_IMPORT Discovery discovery_by_port_name(const char* port_name, size_t port_name_len);
    // If unable to find a device, returns nullptr
    API_IMPORT Discovery discovery_by_serial_number(const char* serial_number, size_t serial_number_len);
    // Must be called to avoid leaks!
    API_IMPORT void free_discovery(Discovery discovery);

    /**
     * Set the wavelength of the variable-wavelength laser. Returns
     * 0 if successful, -1 if the wavelength is out of bounds.
     */
    API_IMPORT int discovery_set_wavelength(Discovery discovery, float wavelength);
    API_IMPORT float discovery_get_wavelength(Discovery discovery);

    API_IMPORT float discovery_get_power_variable(Discovery discovery);
    API_IMPORT float discovery_get_power_fixed(Discovery discovery);

    /**
     * Set the GDD of the laser. Returns 0 if successful, -1 if the GDD is out of bounds.
     */
    API_IMPORT int discovery_set_gdd(Discovery discovery, float gdd);
    API_IMPORT float discovery_get_gdd(Discovery discovery);

    /**
     * Set the alignment mode of the variable-wavelength laser. Returns 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int discovery_set_alignment_variable(Discovery discovery, bool alignment_variable);
    API_IMPORT bool discovery_get_alignment_variable(Discovery discovery);

    /**
     * Set the alignment mode of the fixed-wavelength laser. Returns 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int discovery_set_alignment_fixed(Discovery discovery, bool alignment_fixed);
    API_IMPORT bool discovery_get_alignment_fixed(Discovery discovery);

    /**
     * Get the serial number of the device. Returns nullptr if an error occurred.
     * The serial number is stored in the provided buffer, and the length of the serial number is stored in serial_len.
     */
    API_IMPORT void discovery_get_serial(Discovery discovery, char* serial, size_t* serial_len);

    API_IMPORT int discovery_set_shutter_variable(Discovery discovery, bool shutter_variable);
    API_IMPORT bool discovery_get_shutter_variable(Discovery discovery);

    API_IMPORT int discovery_set_shutter_fixed(Discovery discovery, bool shutter_fixed);
    API_IMPORT bool discovery_get_shutter_fixed(Discovery discovery);

    API_IMPORT int discovery_set_laser_to_standby(Discovery discovery, bool laser_standby);
    API_IMPORT bool discovery_get_laser_standby(Discovery discovery);

    API_IMPORT bool discovery_get_keyswitch(Discovery discovery);

    API_IMPORT bool discovery_get_tuning(Discovery discovery);

    API_IMPORT void discovery_get_status(Discovery discovery, char* status, size_t* status_len);
    API_IMPORT void discovery_get_fault_text(Discovery discovery, char* fault_text, size_t* fault_text_len);
    API_IMPORT int discovery_clear_faults(Discovery discovery);
    
}

#endif // COHERENT_RS_DISCOVERY_HPP