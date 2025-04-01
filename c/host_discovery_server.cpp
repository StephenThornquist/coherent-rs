
#define COHERENT_RS_NETWORK
#include "discovery.h"
#include <iostream>
#include <chrono>
#include <thread>
#include <Windows.h>

int main() {

    std::string port("127.0.0.1:907");

    Discovery laser = discovery_find_first();

    void* server = host_discovery_server(laser, port.c_str(), port.length());
    poll_server(server);
    
    Sleep(20000);

    stop_polling(server);
    free_server(server);

    return 0;
}