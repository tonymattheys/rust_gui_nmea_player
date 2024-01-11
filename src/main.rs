#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use egui::{Vec2, Rect, Pos2};
use std::f64;
use std::f64::consts::PI;
use log::debug;

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

    eframe::run_simple_native("Hello World Rust", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tony's Application");

            egui_extras::install_image_loaders(ctx);

            ui.horizontal(|ui| {
                let lat_label = ui.label("Latitude: ");
                ui.text_edit_singleline(&mut latitude)
                    .labelled_by(lat_label.id);
                ui.separator();
                let lon_label = ui.label("Longitude: ");
                ui.text_edit_singleline(&mut longitude)
                    .labelled_by(lon_label.id);
            });

            ui.vertical(|ui| {
            	ui.style_mut().spacing.slider_width = 400.0;
	            ui.add(egui::Slider::new(&mut zoom, 0..=19).show_value(false).text("Zoom Level").step_by(1.0).max_decimals(0));
            	ui.separator();
    	        ui.label(format!("Latitude: '{latitude}', Longitude: '{longitude}', Zoom Level: {zoom}"));
            	ui.separator();
            });
            
            if ui.button("Click to fetch map tiles").clicked() {
            }

			// Find the top left corner of the window area where the map tiles will
			// be drawn. We need this to place the location marker later on            
            let topleft = ui.cursor(); 
            
            let n = f64::powf(2.0, zoom as f64);
            let lat: f64 = latitude.parse().unwrap_or(0.0);
            let lon: f64 = longitude.parse().unwrap_or(0.0);
            let lat_rad: f64 = lat * PI / 180.0;
            let xtile = (n * ((lon + 180.0) / 360.0)).floor() as u64;
            let ytile = (n * (1.0 - ((lat_rad.tan() + (1.0 / lat_rad.cos())).ln() / PI)) / 2.0).floor() as u64;
            // Save widget spacing before changing it to zero for images
            let spaces = ui.spacing().item_spacing;
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
			// coordinates within the map tiles
			// each map image is 256 pixels wide for a total height/width of 768
			let tl_lat = ((PI * (1.0 - 2.0 * (ytile - 1) as f64 / n)).sinh()).atan() * 180.0 / PI;
			let tl_lon = (xtile - 1) as f64 / n * 360.0 - 180.0;
			let br_lat = ((PI * (1.0 - 2.0 * (ytile + 2) as f64 / n)).sinh()).atan() * 180.0 / PI;
			let br_lon = (xtile + 2) as f64 / n * 360.0 - 180.0;
			let x = (topleft.left() as f64) + 768.0 * ((lon - tl_lon)/(br_lon - tl_lon)).abs();
			let y = (topleft.top() as f64) + 768.0 * ((lat - tl_lat)/(br_lat - tl_lat)).abs();
			// Place the marker so that the centre of the 64x64 marker image is at the lat/long specified
			// The marker image is 64 pixels wide, hence the adding and subtracting of 32 (64/2) in the line below.
			let location: Rect = Rect{min: Pos2 { x: x as f32 - 32.0, y: y as f32 - 32.0 }, max: Pos2 { x: x as f32 + 32.0, y: y as f32 + 32.0 }};
            let marker = egui::Image::new("https://cdn.icon-icons.com/icons2/1494/PNG/64/gps_102930.png");
			ui.put(location, marker);
            
        });
    })
}
