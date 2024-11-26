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

/**
 * @brief Raw pointer to a `Discovery` object
 * for calling `Rust` code
 */
typedef void *Discovery;

/**
 * @brief Raw pointer to a `DiscoveryClient` object,
 * a `BasicLaserNetworkClient<Discovery>` in `Rust`.
 */
typedef void *DiscoveryClient;
// typedef void *DiscoveryStatus;

typedef bool SHUTTER_STATE;

typedef enum {
    OPEN = true,
    CLOSED = false
} ShutterState;

/**
 * @brief A struct to hold the status of a Discovery device,
 * closely matching the `DiscoveryStatus` struct in `Rust`.
 */
typedef struct DiscoveryStatus {
    bool echo;
    bool laser;
    bool variable_shutter;
    bool fixed_shutter;
    bool keyswitch;
    bool faults;
    char *fault_text;
    size_t fault_text_len;
    bool tuning;
    bool alignment_var;
    bool alignment_fixed;
    char *status;
    size_t status_len;
    float wavelength;
    float power_variable;
    float power_fixed;
    int gdd_curve;
    char *gdd_curve_n;
    size_t gdd_curve_n_len;
    float gdd;
} DiscoveryStatus;

extern "C" {
    /**
     * @brief If unable to find a device, returns nullptr.
     * Caller is responsible for freeing the returned Discovery.
     * 
     * @return Discovery or nullptr
     */
    API_IMPORT Discovery discovery_find_first();
    // If unable to find a device, returns nullptr
    API_IMPORT Discovery discovery_by_port_name(const char* port_name, size_t port_name_len);
    // If unable to find a device, returns nullptr
    API_IMPORT Discovery discovery_by_serial_number(const char* serial_number, size_t serial_number_len);
    
    /**
     * @brief Used to free memory managed by a Discovery object.
     * 
     * @param discovery 
     */
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

    /**
     * @brief Returns a Status string for the laser. I actually don't know
     * how much to allocate but probably no more than 256 bytes...
     * 
     * @param discovery `Discovery` object to get the status of
     * @param status `char*` buffer to store the status string. Must be pre-allocated.
     * @param status_len Returns the length of the status string that was populated.
     * @return void
     */
    API_IMPORT void discovery_get_status(Discovery discovery, char* status, size_t* status_len);

    /**
     * @brief Returns the Fault Text for the laser. I actually don't know how 
     * much to allocate but probably no more than 256 bytes...
     * 
     * @param discovery 
     * @param fault_text 
     * @param fault_text_len 
     * @return int 0 if successful, -1 if an error occurred. 
     */
    API_IMPORT void discovery_get_fault_text(Discovery discovery, char* fault_text, size_t* fault_text_len);
    API_IMPORT int discovery_clear_faults(Discovery discovery);

#ifdef COHERENT_RS_NETWORK
// Network functions to manage a Discovery over sockets.

    /**
     * @brief If unable to find a device, returns nullptr.
     * Caller is responsible for freeing the returned DiscoveryClient.
     * 
     * See `free_discovery_client` to free the returned DiscoveryClient.
     * 
     * @param port_name Port name of the device to connect to
     * @param port_name_len Length of port_name char array
     * @return `DiscoveryClient` or nullptr
     */
    API_IMPORT DiscoveryClient connect_discovery_client(const char* port_name, size_t port_name_len);

    /**
     * @brief Must be called to avoid leaks!
     * 
     * @param client DiscoveryClient to free
     */
    API_IMPORT void free_discovery_client(DiscoveryClient client);

    /**
     * @brief Set the connected `Discovery` variable path shutter to the given value.
     * Open is `true`, closed is `false`.
     * 
     * @param client `DiscoveryClient` maintaining a socket connection to a `Server`.
     * @param shutter_variable `true` for open, `false` for closed.
     * @return `int` 0 if successful, -1 if an error occurred. 
     */
    API_IMPORT int set_discovery_client_variable_shutter(DiscoveryClient client, SHUTTER_STATE shutter_variable);

    /**
     * @brief Set the connected `Discovery` fixed path shutter to the given value.
     * Open is `true`, closed is `false`.
     * 
     * @param client 
     * @param shutter_fixed 
     * @return `int` 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int set_discovery_client_fixed_shutter(DiscoveryClient client, SHUTTER_STATE shutter_fixed);

    API_IMPORT int set_discovery_client_wavelength(DiscoveryClient client, float wavelength);

    API_IMPORT int set_discovery_client_to_standby(DiscoveryClient client, bool to_standby);

    API_IMPORT int set_discovery_client_variable_alignment(DiscoveryClient client, bool alignment_on);

    API_IMPORT int set_discovery_client_fixed_alignment(DiscoveryClient client, bool alignment_on);

    API_IMPORT int set_discovery_client_gdd(DiscoveryClient client, float gdd);

    API_IMPORT int set_discovery_client_gdd_curve(DiscoveryClient client, int gdd_curve);


    API_IMPORT DiscoveryStatus discovery_client_query_status(DiscoveryClient client);
#endif // COHERENT_RS_NETWORK
 
}

#endif // COHERENT_RS_DISCOVERY_HPP