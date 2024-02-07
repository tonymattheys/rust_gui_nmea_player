use reverse_geocoder::ReverseGeocoder;

// This function uses the native reverse geocoding mechanism which should be
// wicked fast. There is another function below which assumes you have started 
// a local reverse geocoding server to resolve the queries
pub fn now(lat: f64, lon: f64) -> String {
    let geocoder = ReverseGeocoder::new();
    let coords = (lat, lon);
    let r = geocoder.search(coords).record;
    format!("{}, {}, {}, {}", r.name, r.admin1, r.admin2, r.cc)
}
