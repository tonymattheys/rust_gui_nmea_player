#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use pnet::datalink::interfaces;
use egui::{Vec2, Rect, Pos2};
use std::{f64::consts::PI, thread::{self}, sync::mpsc::{self, Receiver, Sender}, path::PathBuf};

mod udp_broadcaster_thread;
use udp_broadcaster_thread::BroadcasterParameters;


#[allow(unused_imports)]	// Debug may or may not be used depending on what I'm
use log::debug;				// doing so an unused import for it is just fine


fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 920.0]),
        ..Default::default() // make everything default except what I overrode above
    };

    // Our application state:
    let mut latitude = "49.1234".to_string();
    let mut longitude = "-123.4567".to_string();
    let mut zoom = 10;
    let mut selected = 1; // more likely not to be the loopback address
	let mut udp_port = "10110".to_string();
	let mut picked_path = "".to_string();
	let mut broadcasting = false;

    eframe::run_simple_native("NMEA Player", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
		    let (tx, rx): (Sender<BroadcasterParameters>, Receiver<BroadcasterParameters>) = mpsc::channel();
			
	        ui.spacing_mut().item_spacing = Vec2 { x: 10.0, y: 10.0 };
   	        ui.heading(format!("Lat: '{latitude}', Lon: '{longitude}', Zoom : {zoom}"));
            egui_extras::install_image_loaders(ctx);
			// Get list of "interesting" network interfaces for ComboBox later
            let mut alternatives: Vec<String> = Vec::new();
            for i in interfaces() {
                for n in i.ips {
                    if n.is_ipv4() {
                    	alternatives.push(format!("{} ({})", i.name.to_owned(), n.ip()));
                    }
                }
            }
			// File selection stuff
			if ui.button("Open file…").clicked() {
				// We will only open a file if we are not already broadcasting.
				if !broadcasting {
	                let path = match rfd::FileDialog::new().pick_file() {
	                	Some(p) => {
	                		p
	                	}
	                	None => {
	                		PathBuf::new()
	                	}
	                };
	           		picked_path = path.display().to_string();
	           		let bp = BroadcasterParameters {
	           			lat: latitude.parse::<f64>().unwrap_or(0.0),
	           			lon: longitude.parse::<f64>().unwrap_or(0.0),
	           			ifc: alternatives[selected].to_owned(),
	           			udp: udp_port.parse::<u64>().unwrap_or(10110),
	           			line: "".to_owned(),
	           		};
				    let _child = thread::spawn(move || {
				        udp_broadcaster_thread::send_file_lines(path.display().to_string(), bp, tx);
				    });
				    broadcasting = true;
			    }
	        }
            ui.monospace(picked_path.to_owned());
            
			if broadcasting { // the broadcasting thread is already running so try get data
	             match rx.try_recv() {
		            Ok(f) => {
		            	println!("received {:?}", f);
		            	latitude = format!("{:.4}", f.lat);
		            	longitude = format!("{:.4}", f.lon);
		            },
		            Err(_) => {
		            	println!("Error trying to receive...")
		            },
    		    };
			}
			// Display Latitude and longitude text boxes which allow direct 
			// editing of lat/long values with realtime map update as a bonus
            ui.horizontal(|ui| {
                let lat_label = ui.label("Latitude: ");
                ui.text_edit_singleline(&mut latitude).labelled_by(lat_label.id);
                ui.separator();
                let lon_label = ui.label("Longitude: ");
                ui.text_edit_singleline(&mut longitude)
                    .labelled_by(lon_label.id);
            });

			// ComboBox to select network interface, UDP Port text edit area 
			// and slider for zoom level
            ui.horizontal(|ui| {
	            ui.spacing_mut().item_spacing = Vec2 { x: 20.0, y: 20.0 };
            	ui.style_mut().spacing.combo_width = 150.0;
	            egui::ComboBox::from_label("Interface").show_index(
	                ui,
	                &mut selected,
	                alternatives.len(),
	                |i| alternatives[i].to_owned(),
	            );
	            ui.style_mut().spacing.text_edit_width = 100.0;
                let udp_label = ui.label("UDP Port");
	            ui.text_edit_singleline(&mut udp_port)
                    .labelled_by(udp_label.id);
            	ui.style_mut().spacing.slider_width = 200.0;
            	// Slider to set zoom value (0-19)
	            ui.add(egui::Slider::new(&mut zoom, 0..=19).show_value(false).text("Zoom Level").step_by(1.0).max_decimals(0));
            });
           	ui.separator();
            
			// Find the top left corner of the window area where the map tiles will
			// be drawn. We need this to place the location marker later on            
            let topleft = ui.cursor(); 
			// Now calculate which tiles we need from the tile server based on lat/lon            
            let n = f64::powf(2.0, zoom as f64);
            let lat: f64 = latitude.parse().unwrap_or(0.0);
            let lon: f64 = longitude.parse().unwrap_or(0.0);
            let lat_rad: f64 = lat * PI / 180.0;
            let xtile = (n * ((lon + 180.0) / 360.0)).floor() as u64;
            let ytile = (n * (1.0 - ((lat_rad.tan() + (1.0 / lat_rad.cos())).ln() / PI)) / 2.0).floor() as u64;
            // Save widget spacing before changing it to zero for images
            let spaces = ui.spacing().item_spacing;
			// Set spacing between tile imaqges to zero so we don't get ugly black lines
            ui.spacing_mut().item_spacing = Vec2 { x: 0.0, y: 0.0 };
            ui.vertical(|ui| {
                for y in (ytile - 1)..=(ytile + 1) {
                    ui.horizontal(|ui| {
                        for x in (xtile - 1)..=(xtile + 1) {
                            let osmurl = format!("https://a.tile-cyclosm.openstreetmap.fr/cyclosm/{zoom}/{x}/{y}.png");
                            let img = egui::Image::new(osmurl);
                            ui.add(img.fit_to_original_size(1.0));
                        }
                    });
                }
            });
            // Now set widget spacing to whatever it was before we messed with it
            ui.spacing_mut().item_spacing = spaces;
			// Calculate where the location is by converting from lat/long to pixel
			// coordinates within the map tiles. Then draw the marker at that place
			// each map image is 256 pixels wide for a total height/width of 768
			let topleft_lat = ((PI * (1.0 - 2.0 * (ytile - 1) as f64 / n)).sinh()).atan() * 180.0 / PI;
			let topleft_lon = (xtile - 1) as f64 / n * 360.0 - 180.0;
			let bottomright_lat = ((PI * (1.0 - 2.0 * (ytile + 2) as f64 / n)).sinh()).atan() * 180.0 / PI;
			let bottomright_lon = (xtile + 2) as f64 / n * 360.0 - 180.0;
			let x = (topleft.left() as f64) + 768.0 * ((lon - topleft_lon)/(bottomright_lon - topleft_lon)).abs();
			let y = (topleft.top() as f64) + 768.0 * ((lat - topleft_lat)/(bottomright_lat - topleft_lat)).abs();
			// Place the marker so that the pointer of the 64x64 marker image is at the lat/long specified
			// The pointer is at x=32 and about y=56 (by experiment) which is close enough for this purpose
			let location: Rect = Rect{min: Pos2 { x: x as f32 - 32.0, y: y as f32 - 56.0 }, max: Pos2 { x: x as f32 + 32.0, y: y as f32 + 8.0 }};
			// egui::include_image! will statically link the image bytes into the executable            
            let marker = egui::Image::new(egui::include_image!("gps_102930.png"));
			ui.put(location, marker);
        });
    })
}