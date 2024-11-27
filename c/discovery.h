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
const bool echo;
    const bool laser;
    const bool variable_shutter;
    const bool fixed_shutter;
    const bool keyswitch;
    const bool faults;
    const char *fault_text;
    const size_t fault_text_len;
    const bool tuning;
    const bool alignment_var;
    const bool alignment_fixed;
    const char *status;
    const size_t status_len;
    const float wavelength;
    const float power_variable;
    const float power_fixed;
    const int gdd_curve;
    const char *gdd_curve_n;
    const size_t gdd_curve_n_len;
    const float gdd;
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
     * @brief Set the wavelength of the variable-wavelength laser. Returns
     * 0 if successful, -1 if the wavelength is out of bounds.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @param wavelength Desired wavelength in nm
     * 
     * @return `int` 0 if successful, -1 if the wavelength is out of bounds.
     */
    API_IMPORT int discovery_set_wavelength(Discovery discovery, float wavelength);

    /**
     * @brief Get the wavelength of the variable-wavelength laser.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @return `float` Wavelength in nm
     */
    API_IMPORT float discovery_get_wavelength(Discovery discovery);

    API_IMPORT float discovery_get_power_variable(Discovery discovery);
    API_IMPORT float discovery_get_power_fixed(Discovery discovery);

    /**
     * @brief Set the GDD of the laser. Returns 0 if successful, -1 if the GDD is out of bounds.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @param gdd Desired GDD in fs^2
     * 
     * @return `int` 0 if successful, -1 if the GDD is out of bounds.
     */
    API_IMPORT int discovery_set_gdd(Discovery discovery, float gdd);

    /**
     * @brief Get the GDD of the laser.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @return `float` GDD in fs^2
     */
    API_IMPORT float discovery_get_gdd(Discovery discovery);

    /**
     * @brief Set the alignment mode of the variable-wavelength laser. Returns 0 if successful, -1 if an error occurred.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @param alignment_variable `true` for alignment mode on, `false` for off.
     * 
     * @return `int` 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int discovery_set_alignment_variable(Discovery discovery, bool alignment_variable);

    /**
     * @brief Get the alignment mode of the variable-wavelength laser.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @return `bool` `true` if alignment mode is on, `false` if off.
     */
    API_IMPORT bool discovery_get_alignment_variable(Discovery discovery);

    /**
     * @brief Set the alignment mode of the fixed-wavelength laser.
     * Returns 0 if successful, -1 if an error occurred.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @param alignment_fixed `true` for alignment mode on, `false` for off.
     * 
     * @return `int` 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int discovery_set_alignment_fixed(Discovery discovery, bool alignment_fixed);

    /**
     * @brief Get the alignment mode of the fixed-wavelength laser.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @return `bool` `true` if alignment mode is on, `false` if off.
     */
    API_IMPORT bool discovery_get_alignment_fixed(Discovery discovery);

    /**
     * @brief Get the serial number of the device. Returns nullptr if an error occurred.
     * The serial number is stored in the provided buffer, and the length of the serial number is stored in serial_len.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @param serial Buffer to store the serial number
     * @param serial_len Stores the length of the serial number
     * 
     * @return void
     */
    API_IMPORT void discovery_get_serial(Discovery discovery, char* serial, size_t* serial_len);

    /**
     * @brief Sets the shutter for the variable path. Open is `true`, closed is `false`.
     * 
     * @param discovery 
     * @param shutter_variable 
     * @return API_IMPORT 
     */
    API_IMPORT int discovery_set_shutter_variable(Discovery discovery, SHUTTER_STATE shutter_variable);

    /**
     * @brief Gets the shutter state for the variable path.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * 
     * @return `SHUTTER_STATE` `true` if open, `false` if closed.
     */
    API_IMPORT SHUTTER_STATE discovery_get_shutter_variable(Discovery discovery);

    /**
     * @brief  Sets the shutter for the fixed path. Open is `true`, closed is `false`.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @param shutter_fixed `true` for open, `false` for closed.
     * @return `int` 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int discovery_set_shutter_fixed(Discovery discovery, SHUTTER_STATE shutter_fixed);

    /**
     * @brief Gets the shutter state for the fixed path.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @return `SHUTTER_STATE` `true` if open, `false` if closed.
     */
    API_IMPORT SHUTTER_STATE discovery_get_shutter_fixed(Discovery discovery);

    /**
     * @brief Sets the laser to standby mode. Standby is `true`, active is `false`.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @param laser_standby `true` for standby, `false` for active.
     * @return `int` 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int discovery_set_laser_to_standby(Discovery discovery, bool laser_standby);

    /**
     * @brief Gets the standby mode of the laser.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @return `bool` `true` if in standby mode, `false` if active.
     */
    API_IMPORT bool discovery_get_laser_standby(Discovery discovery);

    /**
     * @brief Gets the keyswitch state of the laser.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @return `bool` `true` if the keyswitch is on, `false` if off.
     */
    API_IMPORT bool discovery_get_keyswitch(Discovery discovery);

    /**
     * @brief Gets the tuning state of the laser.
     * 
     * @param discovery Raw pointer to a `Discovery` object
     * @return `bool` `true` if tuning, `false` if ready
     */
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
     * @return `int` 0 if successful, -1 if an error occurred, -2 if the caller is not the primary client
     */
    API_IMPORT int set_discovery_client_variable_shutter(DiscoveryClient client, SHUTTER_STATE shutter_variable);

    /**
     * @brief Set the connected `Discovery` fixed path shutter to the given value.
     * Open is `true`, closed is `false`.
     * 
     * @param client `DiscoveryClient` maintaining a socket connection to a `Server`.
     * @param shutter_fixed `true` for open, `false` for closed.
     * @return `int` 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int set_discovery_client_fixed_shutter(DiscoveryClient client, SHUTTER_STATE shutter_fixed);

    /**
     * @brief Set the connected `Discovery` laser's variable output's wavelength
     * @param client `DiscoveryClient` maintaining a socket connection to a `Server`.
     * @param to_standby 
     * @return `int` 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int set_discovery_client_wavelength(DiscoveryClient client, float wavelength);

    /**
     * @brief Set the connected `Discovery` laser to standby mode.
     * Standby is `true`, active is `false`.
     * 
     * @param client `DiscoveryClient` maintaining a socket connection to a `Server`.
     * @param to_standby `true` for standby, `false` for active.
     * @return `int` 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int set_discovery_client_to_standby(DiscoveryClient client, bool to_standby);

    /**
     * @brief Set the connected `Discovery` variable path to alignment mode
     * 
     * @param client `DiscoveryClient` maintaining a socket connection to a `Server`.
     * @param alignment_on `true` for alignment mode on, `false` for off.
     * @return `int` 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int set_discovery_client_variable_alignment(DiscoveryClient client, bool alignment_on);

    /**
     * @brief Set the connected `Discovery` fixed path to alignment mode
     * 
     * @param client `DiscoveryClient` maintaining a socket connection to a `Server`.
     * @param alignment_on `true` for alignment mode on, `false` for off.
     * @return `int` 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int set_discovery_client_fixed_alignment(DiscoveryClient client, bool alignment_on);

    /**
     * @brief Set the connected `Discovery` laser's GDD
     * 
     * @param client `DiscoveryClient` maintaining a socket connection to a `Server`.
     * @param gdd Desired GDD in fs^2
     * @return `int` 0 if successful, -1 if an error occurred.
     */
    API_IMPORT int set_discovery_client_gdd(DiscoveryClient client, float gdd);

    /**
     * @brief Get the connected `Discovery` laser's GDD
     * 
     * @param client `DiscoveryClient` maintaining a socket connection to a `Server`.
     * @return `float` GDD in fs^2
     */
    API_IMPORT int set_discovery_client_gdd_curve(DiscoveryClient client, int gdd_curve);

    /**
     * @brief Queries the status of the connected `Discovery` laser and returns
     * a `DiscoveryStatus` struct containing all of the various parameters of the
     * laser.
     * 
     * @param client `DiscoveryClient` maintaining a socket connection to a `Server`.
     * @return `DiscoveryStatus` struct containing the status of the laser.
     */
    API_IMPORT DiscoveryStatus discovery_client_query_status(DiscoveryClient client);

    /**
     * @brief Demands to become the primary client of the connected server.
     * 
     * @param client 
     * @return `int` 0 if successful, -1 if there was already a primary client.
     */
    API_IMPORT int demand_primary_client(DiscoveryClient client);

    /**
     * @brief Releases the primary client status of the connected server.
     * 
     * @param client 
     * @return `int` 0 if successful, -1 if an error occured.
     */
    API_IMPORT int release_primary_client(DiscoveryClient client);

#endif // COHERENT_RS_NETWORK
 
}

#endif // COHERENT_RS_DISCOVERY_HPP