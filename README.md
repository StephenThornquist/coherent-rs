# Coherent-RS

Rust-based tools for communicating with `Coherent` brand lasers
used in two-photon microscopy.

**Currently implemented:**
- DiscoveryNX

**To do**
- Ultra II
- Vision S
- DiscoveryNX TPC

# Use

## Connecting to a laser 

You can find a laser either by port name, or by searching for any available
device of the intended class. Any laser that implements the `Laser` trait
can be constructed with `open` and specifying a port or by using `find_first`.
```rust

use coherent_rs::{open, Discovery}

let discovery = open::<Discovery>("NotAPortWithADiscovery")
assert!(discovery.is_err());

let discovery = Discovery::find_first();
assert!(discovery.is_ok());

let mut discovery = discovery.unwrap();
```

You can also attempt to specify one by serial number with a fallback to
`find_first` using the `new` method:

```rust
use coherent_rs::Discovery;

// Open a specific port, but it doesn't exist
let discovery = Discovery::new(Some("NotAPort"), None);
assert!(discovery.is_err());

// Open a specific port that exists
let discovery = Discovery::new(Some("COM5"), None).unwrap();

// Open the first available laser
let discovery = Discovery::new(None, None).unwrap();

// Open a specific laser by serial number
let discovery = Discovery::new(None, Some("123456")).unwrap();

// Open a specific laser by serial number on a specific port
let discovery = Discovery::new(Some("COM5"), Some("123456")).unwrap();
```

## Setting the laser

Lasers can be interacted with in two ways: the `Command` framework, which
sets parameters of the laser, and the `Query` framework, which reads parameters
of the laser. Both are purely wrappers for the serial interface -- no variables are
stored in the `Rust` structs themselves except for the `echo` and `prompt` state, which
are necessary to know how to parse the serial stream.

Each type of laser has its own `Command`s and `Query`s, and I have to admit the system I settled
on is a bit baroque. `Query`s are implemented as a `Trait` so that the `query` function can
be a generic, and thus the compiler can infer the return types of each call. `Command`s are
a laser-specific `enum`, specified in the struct's definition as `CommandEnum`. All `Command`
arguments are made using variants of the `CommandEnum`.

For convenience, for some types of `Query` or `Command` interactions, I have implemented more
traditional methods that are explicitly defined, e.g. `set_wavelength(wv : f32)`.

I also strongly recommend providing time for the serial communication -- if commands are issued very
quickly, sometimes the laser will reply with "Command not executed", which produces a
`CoherentError::CommandNotExecutedError`. When this happens, I recommend just trying to call it again.

It's much more clear when you see this written out.

The generic style looks as follows:

```rust
use coherent_rs::{Discovery, DiscoveryNXQueries, DiscoveryNXCommands, DiscoveryLaser};

let discovery = Discovery::find_first().unwrap();

// Returns a `Result<String, CoherentError>`
let serial_number = discovery.query(DiscoveryNXQueries::Serial{});

println!{"Serial number is {:?}", serial_number.unwrap()};

let fixed_wavelength_power = discovery.query(
    DiscoveryNXQueries::Power{laser : laser::DiscoveryLaser::FixedWavelength}
);
assert!(fixed_wavelength_power.is_ok());

println!{"Fixed wavelength beam power : {:?}", fixed_wavelength_power.unwrap()}

// Now use a Command to open and close the shutter.
discovery.send_command(
    DiscoveryNXCommands::Shutter{
        laser : DiscoveryLaser::FixedWavelength, state : laser::ShutterState::Open
    }
).unwrap();

// The laser ignores subsequent shutter commands for a few hundred milliseconds...
std::thread::sleep(std::time::Duration::from_millis(300));

discovery.send_command(
    DiscoveryNXCommands::Shutter
        {laser : DiscoveryLaser::FixedWavelength,
        state:laser::ShutterState::Closed}
    
).unwrap();
```

The convenience function version of this, which requires a little bit extra manual implementation
for every new laser / function, is:

```rust
use coherent_rs::laser::{Discovery, DiscoveryNXQueries, DiscoveryNXCommands, DiscoveryLaser};

let discovery = Discovery::find_first().unwrap();

println!{"Serial : {:?}", discovery.get_serial().unwrap()};

let fixed_wavelength_power = discovery.get_power(laser::DiscoveryLaser::FixedWavelength);

println!{"Fixed wavelength beam power : {:?}", fixed_wavelength_power.unwrap()}

discovery.set_shutter(laser::DiscoveryLaser::FixedWavelength,
    laser::ShutterState::Open).unwrap();

std::thread::sleep(std::time::Duration::from_millis(300));

discovery.set_shutter(laser::DiscoveryLaser::FixedWavelength,
        laser::ShutterState::Closed).unwrap();
```

## Network

It's slightly frustrating that there's only one USB port on the Coherent lasers,
because sometimes multiple systems interact with one laser. This crate also contains
a tool for network communication with classes implementing the `Laser` trait. It relies
on `serde` and `rmp-serde` to serialize laser commands. To build these features, you need
to use the `network` features flag, e.g.:
`cargo build --release --features network`

```rust
use coherent_rs::{Discovery, DiscoveryNXCommands,
    network::{NetworkLaserServer, BasicNetworkLaserClient}
};

let discovery = Discovery::find_first().unwrap();

let mut server = NetworkLaserServer::new(discovery, "127.0.0.1:907", Some(0.2))
    .unwrap(); // polling interval = 200 ms
server.poll();

// you can control the laser directly with the Server object if you happen
// to own it (i.e. you're not a client socket)
match server.command(
    DiscoveryNXCommands::Shutter{laser : DiscoveryLaser::VariableWavelength, state : true.into()}
){
    Ok(()) => {},
    Err(_) => {eprintln!{"Failed to call command!"};}
};

// Or you can interact view a client
let mut my_client = BasicNetworkLaserClient::<Discovery>::connect("127.0.0.1:907").unwrap();

println!("{:?}" , my_client.query_status().unwrap());

my_client.command(
    DiscoveryNXCommands::Shutter{laser : DiscoveryLaser::VariableWavelength, state : true.into()}
).unwrap();

```

There is also `primary_client` functionality -- a client can demand to become the 
primary client, and if no primary client already exists for a `Server`, it will
become the _only_ client allowed to issue commands (all can still query). 

In emergencies, you can use the `force_forget_primary_client` function, which will
just clear the `Server`s primary client. It is recommended that you not expose this
backdoor in public-facing APIs.

## FFI (C API)

This tool was developed in `Rust` to make it behave smoothly and easily across
platforms, but an expected use case is calling this code from `C` (e.g. to implement
a `ROS2` node controlling a laser, or to integrate into legacy `C`-based microscope control).
This crate exposes a limited C ABI to retrieve a pointer to and from the lasers implemented inside
and call and set specific functions. A slightly more thorough example script is in `c/example.cpp`,
but here's a simple readable version

```c
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

    std::cout << "Device found!" << std::endl;
    char *serial = new char[256];
    size_t *serial_len = new size_t;
    discovery_get_serial(discovery, serial, serial_len);
    std::cout << "Serial: "; std::cout.write(serial, *serial_len); std::cout << std::endl;

    delete[] serial; delete serial_len;

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

    free_discovery(discovery);
    return 0;
}

```

This can be executed by first building the main crate and its C wrapper:
```
cargo build --release --workspace
```
and then compiling, in this example on Windows (note that you need to use the right architecture,
so either modify your `target` in `cargo build` or use the `x64 Native` command line on Windows!)

Ubuntu:
```
g++ ./c/example.cpp -o example.a -lcoherent_rs_c -L./target/release
```

Windows:
```
cl /I ./c ./c/example.cpp /link target\release\coherent_rs_c.dll.lib
```

Then copy the `coherent_rs_c.dll` (Windows) or `coherent_rs_c.so` from `.\target\release`
to the main directory (or alternatively, add the dll location to your `PATH`) and you can run
`example.exe`!

You can also use the network tools in `C/C++`, albeit quite clunkily. Not every
function has been implemented here either...

```C++

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

    DiscoveryStatus newStatus = discovery_client_query_status(client);

    std::cout << "Status variable shutter: " << newStatus.variable_shutter << std::endl;

    std::this_thread::sleep_for(std::chrono::milliseconds(500));

    set_discovery_client_variable_shutter(client, ShutterState::CLOSED);
    // set_discovery_client_variable_shutter(client, false);

    DiscoveryStatus thirdStatus = discovery_client_query_status(client);

    std::cout << "Status variable shutter: " << thirdStatus.variable_shutter << std::endl;
    free_discovery_client(client);
    return 0;
}

```