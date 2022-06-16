use crate::datastore::{self, DataStore};
use crate::settings::Settings;

use ah_system_adapter::{
    ArrowheadService, ArrowheadSystem, ArrowheadSystemAdapter, EntryTag, NoEntryTag, Orchestration,
    RegisterServiceInput, RequestOrchestrationInput,
};

use std::error;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ArrowheadAdapterError(String),
    DataStoreError(String),
    SettingsError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArrowheadAdapterError(message) => write!(
                f,
                "Error during communicating with Arrowhead core systems: {}",
                message
            ),
            Self::DataStoreError(message) => {
                write!(f, "Error during handling saved data: {}", message)
            }
            Self::SettingsError(message) => {
                write!(f, "Error with user settings: {}", message)
            }
        }
    }
}

impl error::Error for Error {}

impl From<ah_system_adapter::Error> for Error {
    fn from(err: ah_system_adapter::Error) -> Self {
        Self::ArrowheadAdapterError(format!("{}", err))
    }
}

impl From<datastore::Error> for Error {
    fn from(err: datastore::Error) -> Self {
        Self::DataStoreError(format!("{}", err))
    }
}

pub struct Controller {
    pub data_store: DataStore,
}

impl Controller {
    pub fn register_service(
        &self,
        name: &str,
        service_to_register: RegisterServiceInput,
    ) -> Result<()> {
        let system_adapter = self.get_system_adapter()?;

        let registered_service = system_adapter.register_service(service_to_register)?;

        self.data_store.save_service(name, &registered_service)?;

        Ok(())
    }

    pub fn unregister_service(&self, name: &str) -> Result<()> {
        let system_adapter = self.get_system_adapter()?;

        let service_definition_entry = &self.data_store.get_services(name)?[0].1.service_definition;
        system_adapter.unregister_service(service_definition_entry.get_service_definition())?;

        self.data_store.delete_service(name)?;

        Ok(())
    }

    pub fn request_publickey(&self) -> Result<String> {
        let system_adapter = self.get_system_adapter()?;
        Ok(system_adapter.get_public_key()?)
    }

    pub fn request_orchestration(
        &self,
        name: &str,
        input: RequestOrchestrationInput,
    ) -> Result<()> {
        let system_adapter = self.get_system_adapter()?;

        let orchestrations = system_adapter.request_orchestration(input)?.response;

        for (i, orchestration) in orchestrations.iter().enumerate() {
            self.data_store
                .save_orchestration(&format!("{}{}", name, i), orchestration)?;
        }

        Ok(())
    }

    pub fn request_orchestration_id(&self, name: &str, id: i64) -> Result<()> {
        let system_adapter = self.get_system_adapter()?;

        let orchestrations = system_adapter.request_orchestration_by_id(id)?.response;

        for (i, orchestration) in orchestrations.iter().enumerate() {
            self.data_store
                .save_orchestration(&format!("{}{}", name, i), orchestration)?;
        }

        Ok(())
    }

    pub fn set_settings(&self, settings: Settings) -> Result<()> {
        Ok(self.data_store.set_settings(settings)?)
    }

    pub fn get_settings(&self) -> Result<Settings> {
        Ok(self.data_store.get_settings()?)
    }

    pub fn get_services(
        &self,
        name_regex: &str,
    ) -> Result<Vec<(String, ArrowheadService<EntryTag>)>> {
        Ok(self.data_store.get_services(name_regex)?)
    }

    pub fn get_orchestrations(&self, name_regex: &str) -> Result<Vec<(String, Orchestration)>> {
        Ok(self.data_store.get_orchestrations(name_regex)?)
    }

    fn get_system_adapter(&self) -> Result<ArrowheadSystemAdapter> {
        let settings = self.get_settings()?;
        let err_not_set =
            |missing| Error::SettingsError(format!("The value '{}' is not set", missing));
        Ok(ArrowheadSystemAdapter::new(
            &settings
                .service_registry_address
                .ok_or_else(|| err_not_set("serviceRegistryAddress"))?,
            &settings
                .authorization_address
                .ok_or_else(|| err_not_set("authorizationAddress"))?,
            &settings
                .orchestrator_address
                .ok_or_else(|| err_not_set("orchestratorAddress"))?,
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: settings
                    .system_name
                    .ok_or_else(|| err_not_set("systemName"))?,
                address: settings
                    .system_address
                    .ok_or_else(|| err_not_set("systemAddress"))?,
                port: settings
                    .system_port
                    .ok_or_else(|| err_not_set("systemPort"))?,
                authentication_info: settings.system_authentication_info,
            },
        )?)
    }
}
