use rand::Rng;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

#[allow(unused_imports)] 	// Debug may or may not be used depending on what I'm
use log::debug; 			// doing so an unused import for it is just fine

#[derive(Debug)]
pub struct BroadcasterParameters {
    pub lat: f64,
    pub lon: f64,
    pub ifc: String,
    pub udp: u64,
    pub line: String,
}

pub fn send_file_lines(filepath: String, bp: BroadcasterParameters, tx: Sender<BroadcasterParameters>) {
	println!("File path : {:?}", filepath);
	println!("BroadcastParameters : {:?}", bp);
	println!("TX : {:?}", tx);

    let mut file_lines: String = String::new();
   
    let mut file = match File::open(filepath) {
        Ok(file) => file,
        Err(e) => panic!("couldn't open {}", e),
    };

    match file.read_to_string(&mut file_lines) {
        Ok(file_lines) => file_lines,
        Err(e) => panic!("couldn't read {}", e),
    };

    for l in file_lines.split_terminator("\r\n") {
        let f = BroadcasterParameters {
            lat: rand::thread_rng().gen_range(49.0..49.2),
            lon: rand::thread_rng().gen_range(-123.4..-123.2),
            ifc: "en0".to_string(),
            udp: 10110,
            line: l.to_string(),
        };
        match tx.send(f) {
        	Ok(f) => { 
        		println!("Sent {:?}", f)
        	},
        	Err(f) => {
        		println!("Error sending {:?}", f)
        	},
        };
        thread::sleep(Duration::from_millis(1000));
    }
    println!("OK, all done, time to leave.");
}
