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
    DiscoveryNXCommands::Shutter(
        (DiscoveryLaser::FixedWavelength, laser::ShutterState::Open)
    )
).unwrap();

// The laser ignores subsequent shutter commands for a few hundred milliseconds...
std::thread::sleep(std::time::Duration::from_millis(300));

discovery.send_command(
    DiscoveryNXCommands::Shutter(
        (DiscoveryLaser::FixedWavelength, laser::ShutterState::Closed)
    )
).unwrap();
```

The convenience function version of this, which requires a little bit extra manual implementation
for every new laser / function, is:

```rust
use coherent_rs::laser::{Discovery, DiscoveryNXQueries, DiscoveryNXCommands, DiscoveryLaser};

let not_discovery = open::<Discovery>("NotAPort");
assert!(not_discovery.is_err());
println!{"Returned : {:?}", not_discovery}

let discovery = Discovery::find_first();
assert!(discovery.is_ok());
let mut discovery = discovery.unwrap();
println!("{:?}", discovery);

println!{"Serial : {:?}", discovery.get_serial().unwrap()};

let fixed_wavelength_power = discovery.get_power(laser::DiscoveryLaser::FixedWavelength);
assert!(fixed_wavelength_power.is_ok());

println!{"Fixed wavelength beam power : {:?}", fixed_wavelength_power.unwrap()}

discovery.set_shutter(laser::DiscoveryLaser::FixedWavelength,
    laser::ShutterState::Open).unwrap();

std::thread::sleep(std::time::Duration::from_millis(300));

discovery.set_shutter(laser::DiscoveryLaser::FixedWavelength,
        laser::ShutterState::Closed).unwrap();
```

## FFI (C API)

This tool was developed in `Rust` to make it behave smoothly and easily across
platforms, but an expected use case is calling this code from `C` (e.g. to implement
a `ROS2` node controlling a laser, or to integrate into legacy `C`-based microscope control).
This crate exposes a limited C ABI to retrieve a pointer to and from the lasers implemented inside
and call and set specific functions.