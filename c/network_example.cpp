/*
Example code to demonstrate the FFI of the Rust network code with C++
*/

#define COHERENT_RS_NETWORK
#include "discovery.h"
#include <iostream>
#include <chrono>
#include <thread>

void print_status(DiscoveryStatus &status) {
    std::cout << "Status echo: " << status.echo << std::endl;
    std::cout << "Status laser: " << status.laser << std::endl;
    std::cout << "Status variable shutter: " << status.variable_shutter << std::endl;
    std::cout << "Status fixed shutter: " << status.fixed_shutter << std::endl;
    std::cout << "Status keyswitch: " << status.keyswitch << std::endl;
    std::cout << "Status faults: " << status.faults << std::endl;

    if (status.fault_text == nullptr) {
        std::cout << "Status fault text: nullptr" << std::endl;
    }
    else {

        std::cout << "Status fault text: ";
        std::cout.write(status.fault_text, status.fault_text_len);
        std::cout << std::endl;
    }

    std::cout << "Status tuning: " << status.tuning << std::endl;
    std::cout << "Status alignment variable: " << status.alignment_var << std::endl;
    std::cout << "Status alignment fixed: " << status.alignment_fixed << std::endl;
    if (status.status == nullptr) {
        std::cout << "Status status: nullptr" << std::endl;
    }
    else {
        std::cout << "Status status: ";
        std::cout.write(status.status, status.status_len);
        std::cout << std::endl;
    }

    std::cout << "Status wavelength: " << status.wavelength << std::endl;
    std::cout << "Status power variable: " << status.power_variable << std::endl;
    std::cout << "Status power fixed: " << status.power_fixed << std::endl;
    std::cout << "Status gdd curve: " << status.gdd_curve << std::endl;

    if (status.gdd_curve_n == nullptr) {
        std::cout << "Status gdd curve n: nullptr" << std::endl;
    }
    else {
        std::cout << "Status gdd curve n: ";
        std::cout.write(status.gdd_curve_n, status.gdd_curve_n_len);
        std::cout << std::endl;
    }

    std::cout << "Status gdd: " << status.gdd << std::endl;

}

int main() {

    std::string port("127.0.0.1:907");

    DiscoveryClient client = connect_discovery_client(port.c_str(), port.length());

    DiscoveryStatus status = discovery_client_query_status(client);

    print_status(status);

    // set_discovery_client_variable_shutter(client, true);
    set_discovery_client_variable_shutter(client, ShutterState::OPEN);

    status = discovery_client_query_status(client);

    std::cout << "Status variable shutter: " << status.variable_shutter << std::endl;

    std::this_thread::sleep_for(std::chrono::milliseconds(500));

    set_discovery_client_variable_shutter(client, ShutterState::CLOSED);
    // set_discovery_client_variable_shutter(client, false);

    status = discovery_client_query_status(client);

    std::cout << "Status variable shutter: " << status.variable_shutter << std::endl;

    free_discovery_client(client);
    return 0;
}