use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use directories::BaseDirs;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

pub struct App {
    config: Config,
    data_path: PathBuf,
}

impl App {
    pub fn new() -> Result<Self> {
        let dirs = match BaseDirs::new() {
            Some(d) => d,
            None => bail!("Can't find home directory"),
        };

        let mut config_path = dirs.config_dir().to_owned();
        config_path.push("background");
        config_path.push("config.json");

        let mut app = Self {
            config: load_json(&config_path)?,
            data_path: dirs.data_local_dir().to_owned(),
        };

        app.data_path.push("background");

        if !app.data_path.exists() {
            fs::create_dir_all(&app.data_path).with_context(|| {
                format!("Failed to create directory {}", app.data_path.display())
            })?;
        }

        app.data_path.push("data.json");

        Ok(app)
    }

    pub fn run(&self, command: &str) -> Result<()> {
        match command {
            "update" => self.update(),
            "status" => self.status(),
            _ => bail!("Unknown command '{}'", command),
        }
    }

    fn update(&self) -> Result<()> {
        let uri = format!(
            "https://api.sunrise-sunset.org/json?lat={}&lng={}&formatted=0",
            self.config.latitude, self.config.longitude
        );

        let error = "Failed to get data from API";
        let response = reqwest::blocking::get(uri)
            .context(error)?
            .text()
            .context(error)?;
        let response: Response = serde_json::from_str(&response).context(error)?;
        let data = serde_json::to_string(&response.results).context(error)?;

        let mut file = File::create(&self.data_path)?;
        write!(file, "{}", data)?;

        Ok(())
    }

    fn status(&self) -> Result<()> {
        if !self.data_path.exists() {
            self.update()?;
        }

        let data: Data = load_json(&self.data_path)?;
        let now = Utc::now();

        let status = if (data.sunrise..data.sunset).contains(&now) {
            "light"
        } else {
            "dark"
        };

        println!("{}", status);

        Ok(())
    }
}

#[derive(Deserialize)]
struct Config {
    latitude: String,
    longitude: String,
}

#[derive(Deserialize, Serialize)]
struct Data {
    sunrise: DateTime<Utc>,
    sunset: DateTime<Utc>,
}

#[derive(Deserialize)]
struct Response {
    results: Data,
}

fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let error = || format!("Failed to read JSON from {}", path.display());
    let file = File::open(path).with_context(error)?;
    let reader = BufReader::new(file);
    let data: T = serde_json::from_reader(reader).with_context(error)?;

    Ok(data)
}
