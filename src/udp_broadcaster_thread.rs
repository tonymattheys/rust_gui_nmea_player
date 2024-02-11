use chrono::{NaiveDate, Utc};
use pnet::datalink;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read};
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::sleep;

#[derive(Serialize, Deserialize, Debug)]
pub struct Shared {
    pub utc: String,
    pub pth: String,
    pub ifc: String,
    pub udp: u16,
    pub lat: f64,
    pub lon: f64,
    pub cog: f64,
    pub sog: f64,
    pub awa: f64,
    pub aws: f64,
    pub dpt: f64,
}
// Implement some sane defaults for the shared memory structure
impl ::std::default::Default for Shared {
    fn default() -> Self {
        Self {
            utc: "0000-00-00 00:00:00".to_string(),
            pth: "No file loaded".to_string(),
            ifc: "eth0".to_string(),
            udp: 10110,
            lat: 49.1234,
            lon: -123.4567,
            cog: 90.0,
            sog: 5.0,
            awa: 45.0,
            aws: 10.0,
            dpt: 10.0,
        }
    }
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

    // Define some variables that can store various dates/times that we need to keep
    // packet sending in synch (more or less) with real time
    let mut file_start_time = NaiveDate::from_ymd_opt(1970, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let mut locl_start_time = Utc::now().naive_utc();

    // Grab the broadcast address of the first IP address assigned to the specified interface
    let ip_addr = interface.ips[0].broadcast();
    let destination = SocketAddr::new(ip_addr, shared_memory.lock().unwrap().udp);
    // Open a UDP socket for the interface
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Socket bind broke.");
    // allow broadcasting on this socket...
    socket
        .set_broadcast(true)
        .expect("Setting broadcast failed.");

    /* Load (or create) the application configuration
    let _cfg: Shared = match confy::load("rust_gui_nmea_player", None) {
        Ok(c) => c,
        Err(e) => {
            println!("Config error: \"{}\"", e);
            Shared::default()
        }
    };*/

    for line in file_lines.split_terminator("\r\n") {
        let fields: Vec<&str> = line.split(',').collect();
        // $GPZDA,234626.99,22,02,2021,08,00*6A
        if fields[0].starts_with("$") && fields[0].len() >= 6 && fields[0][3..6].eq("ZDA") {
            let y: i32 = FromStr::from_str(fields[4]).unwrap_or(1970);
            let m: u32 = FromStr::from_str(fields[3]).unwrap_or(1);
            let d: u32 = FromStr::from_str(fields[2]).unwrap_or(1);
            let hr: u32 = FromStr::from_str(&fields[1][0..2]).unwrap_or(0);
            let mn: u32 = FromStr::from_str(&fields[1][2..4]).unwrap_or(0);
            let se: u32 = FromStr::from_str(&fields[1][4..6]).unwrap_or(0);

            // We put locl_start_time as the default for the unwrap() to help prevent panics
            let dt = NaiveDate::from_ymd_opt(y, m, d)
                .unwrap_or(locl_start_time.date())
                .and_hms_opt(hr, mn, se)
                .unwrap_or(locl_start_time);

            // Set the UTC date and time in the shared memory for dispplay in the GUI
            shared_memory.lock().unwrap().utc = dt.format("%Y-%m-%d %H:%M:%S").to_string();

            // If we have not yet initialized the start times, then do it now.
            if file_start_time
                == NaiveDate::from_ymd_opt(1970, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
            {
                file_start_time = dt;
                locl_start_time = Utc::now().naive_utc();
            }
            // Resynch the elapsed time clocks by sleeping before reading the next line
            let sleep_time = (dt - file_start_time) - (Utc::now().naive_utc() - locl_start_time);
            if sleep_time.num_milliseconds() > 0 {
                sleep(std::time::Duration::from_millis(
                    sleep_time.num_milliseconds() as u64,
                ));
            }
            // println!("I just had a nap for {} ms", sleep_time.num_milliseconds());
        }
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
        // $WIVWR,31.7,L,0.5,N,0.3,M,0.9,K*73
        if fields[0].starts_with("$") && fields[0].len() >= 6 && fields[0][3..6].eq("VWR") {
            let awa: f64 = FromStr::from_str(&fields[1]).unwrap_or(0.0);
            if fields[2].eq_ignore_ascii_case("R") {
                shared_memory.lock().unwrap().awa = awa;
            } else {
                shared_memory.lock().unwrap().awa = -awa;
            };
            shared_memory.lock().unwrap().aws = FromStr::from_str(&fields[3]).unwrap_or(0.0);
        }
        // $SDDPT,10.38,0,*6F
        if fields[0].starts_with("$") && fields[0].len() >= 6 && fields[0][3..6].eq("DPT") {
            let d: f64 = FromStr::from_str(&fields[1]).unwrap_or(0.0);
            let o: f64 = FromStr::from_str(&fields[2]).unwrap_or(0.0);
            shared_memory.lock().unwrap().dpt = d + o;
        }

        /*Save the application configuration
        match confy::store("rust_gui_nmea_player", None, shared_memory.clone()) {
            Ok(_) => {}
            Err(e) => {
                println!("Error saving config : \"{}\"", e)
            }
        };*/

        socket
            .send_to(line.as_bytes(), &destination)
            .expect("Error sending on socket.");
    }
}
