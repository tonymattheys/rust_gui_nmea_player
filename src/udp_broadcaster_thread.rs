use std::fs::File;
use std::io::{self, Read};
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::sleep;

use pnet::datalink;

#[derive(Debug)]
pub struct Shared {
    pub pth: String,
    pub ifc: String,
    pub udp: u16,
    pub lat: f64,
    pub lon: f64,
    pub cog: f64,
    pub sog: f64,
}

pub fn read_file_lines(shared_memory: Arc<Mutex<Shared>>) {
    let mut file_lines: String = String::new();
    let p = shared_memory.lock().unwrap().pth.to_owned();
    let mut file = match File::open(p) {
        Ok(file) => file,
        Err(e) => panic!("couldn't open {}", e),
    };
    match file.read_to_string(&mut file_lines) {
        Ok(f) => f,
        Err(e) => panic!("couldn't read {}", e),
    };

    // Get the network interface with the name that was specified as the first parameter
    let interface = datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == shared_memory.lock().unwrap().ifc.trim())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Interface '".to_owned()
                    + &shared_memory.lock().unwrap().ifc.to_owned()
                    + "' not found",
            )
        })
        .unwrap();
    // Grab the broadcast address of the first IP address assigned to the specified interface
    let ip_addr = interface.ips[0].broadcast();
    let destination = SocketAddr::new(ip_addr, shared_memory.lock().unwrap().udp);
    // Open a UDP socket for the interface
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Socket bind broke.");
    // allow broadcasting on this socket...
    socket
        .set_broadcast(true)
        .expect("Setting broadcast failed.");

    for line in file_lines.split_terminator("\r\n") {
        let fields: Vec<&str> = line.split(',').collect();
        // $GPGGA,020659.21,4937.8509,N,12401.4384,W,2,9,0.83,,M,,M*44
        if fields[0].starts_with("$") && fields[0].len() >= 6 && fields[0][3..6].eq("GGA") {
            // Get latitude from GPS statement
            let x: f64 = FromStr::from_str(&fields[2]).unwrap_or(0.0);
            let lat_deg: f64 = (x / 100.0).floor();
            let lat_min: f64 = (x / 100.0).fract() * 100.0;
            let n_s: &str = fields[3];
            let mut lat_d = lat_deg + (lat_min / 60.0);
            if n_s.contains("S") {
                lat_d = lat_d * -1.0
            }
            shared_memory.lock().unwrap().lat = lat_d;
            // Get longitude from GPS statements
            let x: f64 = FromStr::from_str(&fields[4]).unwrap_or(0.0);
            let lon_deg: f64 = (x / 100.0).floor();
            let lon_min: f64 = (x / 100.0).fract() * 100.0;
            let e_w: &str = fields[5];
            let mut lon_d = lon_deg + (lon_min / 60.0);
            if e_w.contains("W") {
                lon_d = lon_d * -1.0
            }
            shared_memory.lock().unwrap().lon = lon_d;
        }
        // $IIVTG,359.5,T,,M,0.1,N,0.1,K,D*15
        if fields[0].starts_with("$") && fields[0].len() >= 6 && fields[0][3..6].eq("VTG") {
            shared_memory.lock().unwrap().cog = FromStr::from_str(&fields[1]).unwrap_or(0.0);
            shared_memory.lock().unwrap().sog = FromStr::from_str(&fields[5]).unwrap_or(0.0);
        }

        // Introduce a delay to sort of account for transmission speed
        // Assumes 38,400 baud because of AIS
        let dly: f64 = line.len() as f64 / (38400.0 / 8.0) * 1000.0;
        sleep(std::time::Duration::from_millis(dly.floor() as u64));

        socket
            .send_to(line.as_bytes(), &destination)
            .expect("Error sending on socket.");
    }
}
