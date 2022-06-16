use crate::settings::Settings;

use ah_system_adapter::{ArrowheadService, EntryTag, Orchestration};

use regex::Regex;
use serde::{de::DeserializeOwned, Serialize};

use std::error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;
use std::result;

pub type Result<T> = result::Result<T, Error>;

const ROOT_FOLDER: &str = "data";
const SERVICES_FOLDER: &str = "services";
const ORCHESTRATIONS_FOLDER: &str = "orchestrations";
const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug)]
pub enum Error {
    IOError(String),
    RegexError(String),
    SerializationError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IOError(message) => write!(f, "IO error: {}", message),
            Self::RegexError(message) => write!(f, "Regex error: {}", message),
            Self::SerializationError(message) => {
                write!(f, "Serialization error: {}", message)
            }
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IOError(format!("{}", err))
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Self::RegexError(format!("{}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(format!("{}", err))
    }
}

pub struct DataStore {
    services_path: PathBuf,
    orchestrations_path: PathBuf,
    settings_file_path: PathBuf,
}

impl DataStore {
    pub fn new() -> Result<Self> {
        let services_path: PathBuf = [ROOT_FOLDER, SERVICES_FOLDER].iter().collect();
        let orchestrations_path: PathBuf = [ROOT_FOLDER, ORCHESTRATIONS_FOLDER].iter().collect();
        let settings_file_path: PathBuf = [ROOT_FOLDER, SETTINGS_FILE].iter().collect();

        fs::create_dir_all(&services_path)?;
        fs::create_dir_all(&orchestrations_path)?;

        Ok(Self {
            services_path,
            orchestrations_path,
            settings_file_path,
        })
    }

    pub fn set_settings(&self, settings: Settings) -> Result<()> {
        let new_settings = self.get_settings()?.merge(settings);
        Self::save_json(&new_settings, &self.settings_file_path)
    }

    pub fn get_settings(&self) -> Result<Settings> {
        if self.settings_file_path.exists() {
            Self::read_json(&self.settings_file_path)
        } else {
            Ok(Settings::default())
        }
    }

    pub fn save_service(
        &self,
        name: &str,
        service_to_save: &ArrowheadService<EntryTag>,
    ) -> Result<()> {
        let path = self.services_path.join(format!("{}.json", name));
        Self::save_json(service_to_save, &path)
    }

    pub fn save_orchestration(
        &self,
        name: &str,
        orchestartion_to_save: &Orchestration,
    ) -> Result<()> {
        let path = self.orchestrations_path.join(format!("{}.json", name));
        Self::save_json(orchestartion_to_save, &path)
    }

    pub fn delete_service(&self, service_name: &str) -> Result<()> {
        Ok(fs::remove_file(
            &self.services_path.join(format!("{}.json", service_name)),
        )?)
    }

    pub fn get_services(
        &self,
        name_regex: &str,
    ) -> Result<Vec<(String, ArrowheadService<EntryTag>)>> {
        Self::get_filtered_items(&self.services_path, name_regex)
    }

    pub fn get_orchestrations(&self, name_regex: &str) -> Result<Vec<(String, Orchestration)>> {
        Self::get_filtered_items(&self.orchestrations_path, name_regex)
    }

    fn save_json<T: Serialize>(object: &T, path: &PathBuf) -> Result<()> {
        let writer = BufWriter::new(File::create(path)?);
        Ok(serde_json::to_writer_pretty(writer, object)?)
    }

    fn read_json<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
        let reader = BufReader::new(File::open(path)?);
        Ok(serde_json::from_reader(reader)?)
    }

    fn get_filtered_items<T: DeserializeOwned>(
        path: &PathBuf,
        name_regex: &str,
    ) -> Result<Vec<(String, T)>> {
        let re = Regex::new(name_regex)?;
        fs::read_dir(path)?
            .filter_map(|r| r.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter_map(|p| {
                p.file_stem()
                    .map(|o| o.to_str().unwrap_or(""))
                    .filter(|f| !f.is_empty() && re.is_match(f))
                    .map(|f| Self::read_json(&p).map(|t| (f.to_owned(), t)))
            })
            .collect()
    }
}
