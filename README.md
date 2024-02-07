# NMEA file player written in Rust

Rust program which will play the contents of a NMEA text file over the network.

# How to run
- cd wherever_you_downloaded_the_program
- cargo run [--release]

# How to build an executable
- cd wherever_you_downloaded_the_program
- cargo build --release
will build an executable called "nmea_player" under ./target/release

# Description
This program will read a file specified by the user and perform various operations
using the contents of the file as input. The most common way to use this program is
to read in a NMEA0183 file and re-send the NMEA sentences out onto the network using
UDP broadcast on port 10110. This will appear to be a Comar system to Navionics and
other navigation systems that listen for UDP broadcasts on the network.
