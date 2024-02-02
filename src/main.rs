// TODO:
// - Error handle 404s on the API level

use config_file::FromConfigFile;
use serde::Deserialize;
use reqwest::{self, blocking};

#[derive(Debug, Deserialize)]
struct Config {
    key: String,
    zip: i32,
}

#[derive(Debug, Deserialize)]
struct City {
    lat: f64,
    lon: f64,
    zip: String,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    temp: f64,
    description: String,
    name: String,
}

fn convert_k_to_f(t: f64) -> f64 {
    (t - 273.15) * 9.0/5.0 + 32.0
}
fn get_city(zip: i32, key: &str) -> Result<City, Box<dyn std::error::Error>> {
    let loc_url = format!("http://api.openweathermap.org/geo/1.0/zip?zip={}&limit=5&appid={}", zip, key);
    let res = blocking::get(loc_url)?.text()?;
    let root: City = serde_json::from_str(&res)?;
    Ok(root)
}

fn get_current_weather(city: City, key: &str) -> Result<CurrentWeather, Box<dyn std::error::Error>> {

    // helper structs for the JSON data
    #[derive(Debug, Deserialize)]
    struct ApiData {
        weather: Vec<WeatherData>,
        main: Main,
        name: String
    }
    #[derive(Debug, Deserialize)]
    struct WeatherData{
        description: String,
    }
    #[derive(Debug, Deserialize)]
    struct Main {
        temp: f64,
    }

    let weather_url = format!("https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}", city.lat, city.lon, key);
    let res = blocking::get(weather_url)?.text()?;
    let root: ApiData = serde_json::from_str(&res)?;
    Ok(
        CurrentWeather {
            name: root.name,
            temp: convert_k_to_f(root.main.temp),
            description: root.weather.into_iter().map(|f|f.description).collect(),
        }
    )
}

fn main() {
    // importing configuration
    let config = Config::from_config_file("./location.toml");
    let config = match config {
        Ok(config) => config,
        Err(e) => {
            panic!("There was a problem parsing your configuration file: {}",e);
        }
    };

    let grad = match get_city(config.zip, &config.key) {
        Ok(grad) => grad,
        Err(e) => {
            eprintln!("There was an issue getting city details: {}", e);
            std::process::exit(1);
        }
    };

    let curr_weather = match get_current_weather(grad, &config.key) {
        Ok(cw) => cw,
        Err(e) => {
            eprintln!("There was an issue getting weather details: {}",e);
            std::process::exit(1);
        }
    };

    println!("The current weather in {} is {}, with a temperature of {:.0} F", curr_weather.name, curr_weather.description, curr_weather.temp);

}
