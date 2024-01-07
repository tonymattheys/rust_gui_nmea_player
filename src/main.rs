use reqwest::StatusCode;

#[tokio::main]
async fn main() {
    let lat = "lat=49.6308";
    let lon = "lon=-124.0240";
    let apikey = "apiKey=1cee7a64052347b3b86cc1627b441718";
    let request_url = format!(
        "https://api.geoapify.com/v1/geocode/reverse?{}&{}&{}",
        lat, lon, apikey
    );

    println!("{}", request_url);

    let response = reqwest::get(request_url).await.unwrap();

    match response.status() {
        StatusCode::OK => {
            // Grab raw JSON from the http response
            let parsed = json::parse(&response.text().await.unwrap()).unwrap();
            for (k, v) in parsed.entries() {
                match k {
                    "type" => {
                        println!("Type is {}", v)
                    }
                    "features" => {
                        println!("County is '{}'", v[0]["properties"]["county"]);
                        println!("State is '{}'", v[0]["properties"]["state"]);
                        println!("City is '{}'", v[0]["properties"]["city"]);
                    }
                    "query" => {
                        println!("The query was {}", v)
                    }
                    _ => {
                        println!("something else")
                    }
                }
            }
        }
        _ => {
            panic!("Uh oh! Something unexpected happened.");
        }
    };
}
