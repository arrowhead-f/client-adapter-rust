use crate::{
    controller::{self, Controller},
    serde_table,
    settings::Settings,
};

use ah_system_adapter::{
    ArrowheadCloud, ArrowheadProvider, InterfaceEntry, NoEntryTag, RegisterServiceInput,
    RequestOrchestrationInput, ServiceDefinitionEntry, ServiceRequirements,
};

use clap::{ArgEnum, Args, Parser, Subcommand};
use regex::Regex;

use std::{collections::HashMap, error, fmt};

use Arguments::*;

#[derive(Debug)]
pub enum Error {
    CommandError(String),
    ControllerError(String),
    PrintError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandError(message) => {
                write!(f, "Error with the inputs of the command: {}", message)
            }
            Self::ControllerError(message) => {
                write!(f, "Error during executing the command: {}", message)
            }
            Self::PrintError(message) => {
                write!(f, "Error during printing the result: {}", message)
            }
        }
    }
}

impl error::Error for Error {}

impl From<controller::Error> for Error {
    fn from(err: controller::Error) -> Self {
        Self::ControllerError(format!("{}", err))
    }
}

impl From<serde_table::Error> for Error {
    fn from(err: serde_table::Error) -> Self {
        Self::PrintError(format!("{}", err))
    }
}

pub struct ConsoleApp;
impl ConsoleApp {
    pub fn run(controller: &Controller) {
        let result = Self::try_run(controller);
        if let Err(error) = result {
            eprintln!("Error: {}", error);
        }
    }

    fn try_run(controller: &Controller) -> Result<(), Error> {
        let column_width = controller.get_settings()?.column_width.unwrap_or(25);
        let args = Arguments::parse();
        match args {
            Register(RegisterCommand::Service(register_service_command)) => {
                let name = register_service_command.name.clone();
                controller.register_service(&name, register_service_command.into())?;
            }
            Request(request_command) => match request_command {
                RequestCommand::Orchestration(request_orchestration_command) => {
                    let name = request_orchestration_command.name.clone();
                    controller
                        .request_orchestration(&name, request_orchestration_command.try_into()?)?;
                }
                RequestCommand::OrchestrationId(request_orchestration_id_command) => {
                    controller.request_orchestration_id(
                        &request_orchestration_id_command.name,
                        request_orchestration_id_command.id,
                    )?;
                }
                RequestCommand::PublicKey => {
                    controller.request_publickey()?;
                }
            },
            Set(SetCommand::Settings(set_settings_command)) => {
                controller.set_settings(set_settings_command.into())?;
            }
            Show(show_command) => match show_command {
                ShowCommand::Orchestrations { name } => {
                    let orchestrations = controller.get_orchestrations(&name)?;
                    for (name, orchestration) in orchestrations.iter() {
                        serde_table::print_object(name, orchestration, column_width)?;
                    }
                }
                ShowCommand::Services { name } => {
                    let services = controller.get_services(&name)?;
                    for (name, service) in services.iter() {
                        serde_table::print_object(name, service, column_width)?;
                    }
                }
                ShowCommand::Settings => {
                    let settings = controller.get_settings()?;
                    serde_table::print_object("Settings", &settings, column_width)?;
                }
            },
            Unregister(UnregisterCommand::Service { name }) => {
                controller.unregister_service(&name)?;
            }
        }
        Ok(())
    }
}

/// Arrowhead Adapter App
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
enum Arguments {
    /// Register Arrowhead entities
    #[clap(subcommand)]
    Register(RegisterCommand),

    /// Request data from Arrowhead Systems
    #[clap(subcommand)]
    Request(RequestCommand),

    /// Set internal variables
    #[clap(subcommand)]
    Set(SetCommand),

    /// Show various data stored by the application
    #[clap(subcommand)]
    Show(ShowCommand),

    /// Unregister Arrowhead entities
    #[clap(subcommand)]
    Unregister(UnregisterCommand),
}

#[derive(Subcommand)]
enum RegisterCommand {
    /// Register a service
    Service(RegisterServiceCommand),
}

#[derive(Subcommand)]
enum RequestCommand {
    /// Request orchestration for the system
    Orchestration(RequestOrchestrationCommand),
    /// Request store orchestration by id for the system
    OrchestrationId(RequestOrchestrationIdCommand),
    /// Get the public key of the Authorization core service
    PublicKey,
}

#[derive(Subcommand)]
enum SetCommand {
    /// Set the settings
    Settings(SetSettingsCommand),
}

#[derive(Subcommand)]
enum ShowCommand {
    /// Show data of orchestrations
    Orchestrations {
        /// Filter for the previously given name of the orchestration (the orchestrations are numbered automatically)
        name: String,
    },

    /// Show data of services
    Services {
        /// Filter for the previously given name of the service
        name: String,
    },

    /// Show the settings
    Settings,
}

#[derive(Subcommand)]
enum UnregisterCommand {
    /// Register a service
    Service {
        // Name of the service to be unregistered
        name: String,
    },
}

#[derive(Args)]
struct RegisterServiceCommand {
    /// The definition (and identifier) of service to be registered
    #[clap(long)]
    name: String,

    /// List of the interfaces the service supports (pattern: <protocol>-SECURE/INSECURE-<format>, e.g.: HTTPS-SECURE-JSON)
    #[clap(long)]
    interfaces: Vec<String>,

    /// The URI which the service can be accessed on
    #[clap(long)]
    uri: String,

    /// The service is available until this UTC timestamp
    #[clap(long)]
    end_of_validity: Option<String>,

    /// Various meta information as map
    #[clap(long, use_value_delimiter = true, parse(try_from_str = parse_pair))]
    metadata: Option<Vec<(String, String)>>,

    /// The authentication type for the service to be used
    #[clap(long, arg_enum)]
    security_type: Option<SecurityTypeValue>,

    /// The version of this registry entry
    #[clap(long)]
    version: Option<u32>,
}

#[derive(Args)]
struct RequestOrchestrationCommand {
    /// The identifier name of the orchestration reponses returned (the response names are numbered from 1 automatically)
    #[clap(long)]
    name: String,

    /// The definition of service requested
    #[clap(long)]
    service_name: String,

    /// List orchestration flags which should be set true for this orchestration
    #[clap(long, use_value_delimiter = true, arg_enum)]
    flags: Option<Vec<OrchestrationFlag>>,

    /// List of the interfaces the requested service should support (pattern: <protocol>-SECURE/INSECURE-<format>, e.g.: HTTPS-SECURE-JSON)
    #[clap(long)]
    interfaces: Option<Vec<String>>,

    /// The maximal version of the requested service registry entry
    #[clap(long)]
    max_version: Option<u32>,

    /// Various meta information of the requested service as map
    #[clap(long, use_value_delimiter = true, parse(try_from_str = parse_pair))]
    metadata: Option<Vec<(String, String)>>,

    /// The minimal version of the requested service registry entry
    #[clap(long)]
    min_version: Option<u32>,

    /// Parameters of the preferred provider for the requested service, the keys for this map: 'operator-name', 'cloud-name','system-name','system-address','system-port','authentication-info'
    #[clap(long, use_value_delimiter = true, parse(try_from_str = parse_pair))]
    preferred_provider: Option<Vec<(String, String)>>,

    /// The authentication types for the requested service should use
    #[clap(long, use_value_delimiter = true, arg_enum)]
    security_types: Option<Vec<SecurityTypeValue>>,

    /// The version of the requested service registry entry
    #[clap(long)]
    version: Option<u32>,
}

#[derive(Args)]
struct RequestOrchestrationIdCommand {
    /// The identifier name of the orchestration reponses returned (the response names are numbered from 1 automatically)
    #[clap(long)]
    name: String,

    /// The store id of the orchestration
    #[clap(long)]
    id: i64,
}

#[derive(Args)]
struct SetSettingsCommand {
    /// Address of Service Registry core system
    #[clap(long)]
    service_registry_address: Option<String>,

    /// Address of Authorization core system
    #[clap(long)]
    authorization_address: Option<String>,

    /// Address of Orchestrator core system
    #[clap(long)]
    orchestrator_address: Option<String>,

    /// Name of current system
    #[clap(long)]
    system_name: Option<String>,

    /// Address of current system
    #[clap(long)]
    system_address: Option<String>,

    /// Port of current system
    #[clap(long)]
    system_port: Option<u32>,

    /// Authentication info (public key) of current system
    #[clap(long)]
    system_authentication_info: Option<String>,

    /// Column width of the table displaying content (defaults to 25 if not set)
    #[clap(long)]
    column_width: Option<usize>,
}

#[derive(ArgEnum, Clone)]
enum SecurityTypeValue {
    Certificate,
    NotSecure,
    Token,
}

#[derive(ArgEnum, Clone)]
enum OrchestrationFlag {
    Machmaking,
    MetadataSearch,
    OnlyPreferred,
    PingProviders,
    OverrideStore,
    EnableInterCloud,
    TriggerInterCloud,
}

fn parse_pair(input: &str) -> Result<(String, String), &'static str> {
    let re = Regex::new(r"^(.+?):(.+)$").unwrap();
    let captures = re
        .captures(input)
        .ok_or("Map item does not match the expected format (<key>:<value>)")?;

    let key = captures[1].to_owned();
    let value = captures[2].to_owned();
    Ok((key, value))
}

impl Into<RegisterServiceInput> for RegisterServiceCommand {
    fn into(self) -> RegisterServiceInput {
        RegisterServiceInput {
            service_definition: ServiceDefinitionEntry::Value(self.name),
            service_uri: self.uri,
            end_of_validity: self.end_of_validity,
            secure: self.security_type.map(SecurityTypeValue::into),
            metadata: self.metadata.map(|m| m.into_iter().collect()),
            version: self.version,
            interfaces: self
                .interfaces
                .into_iter()
                .map(InterfaceEntry::Value)
                .collect(),
        }
    }
}

impl TryInto<RequestOrchestrationInput> for RequestOrchestrationCommand {
    type Error = Error;

    fn try_into(self) -> Result<RequestOrchestrationInput, Self::Error> {
        let requested_service = ServiceRequirements {
            service_definition_requirement: self.service_name,
            interface_requirements: self.interfaces,
            security_requirements: self
                .security_types
                .map(|s| s.into_iter().map(SecurityTypeValue::into).collect()),
            metadata_requirements: None,
            version_requirement: self.version,
            max_version_requirement: self.max_version,
            min_version_requirement: self.min_version,
        };

        let preferred_providers = match self.preferred_provider {
            Some(pp) => {
                let pp: HashMap<String, String> = pp.into_iter().collect();
                let err = |missing| {
                    Error::CommandError(format!(
                        "The field '{}' of argument 'preferred-provider' is missing",
                        missing
                    ))
                };

                Some(vec![ArrowheadProvider {
                    provider_cloud: ArrowheadCloud {
                        operator: pp
                            .get("operator-name")
                            .ok_or_else(|| err("operator-name"))?
                            .clone(),
                        name: pp
                            .get("cloud-name")
                            .ok_or_else(|| err("cloud-name"))?
                            .clone(),
                    },
                    provider_system: ah_system_adapter::ArrowheadSystem {
                        entry_tag: NoEntryTag {},
                        system_name: pp
                            .get("system-name")
                            .ok_or_else(|| err("system-name"))?
                            .clone(),
                        address: pp
                            .get("system-address")
                            .ok_or_else(|| err("system-address"))?
                            .clone(),
                        port: pp
                            .get("system-port")
                            .ok_or_else(|| err("system-port"))?
                            .parse()
                            .map_err(|e| Error::CommandError(format!("Error during parsing field 'system-port' of argument 'preferred-provider': {}", e)))?,
                        authentication_info: pp.get("authentication-info").map(String::clone),
                    },
                }])
            }
            None => None,
        };

        Ok(RequestOrchestrationInput {
            requested_service,
            preferred_providers,
            orchestration_flags: self
                .flags
                .map(|f| f.into_iter().map(|f| (f.into(), true)).collect()),
        })
    }
}

impl Into<Settings> for SetSettingsCommand {
    fn into(self) -> Settings {
        Settings {
            service_registry_address: self.service_registry_address,
            authorization_address: self.authorization_address,
            orchestrator_address: self.orchestrator_address,
            system_name: self.system_name,
            system_address: self.system_address,
            system_port: self.system_port,
            system_authentication_info: self.system_authentication_info,
            column_width: self.column_width,
        }
    }
}

impl Into<ah_system_adapter::SecurityType> for SecurityTypeValue {
    fn into(self) -> ah_system_adapter::SecurityType {
        match self {
            Self::Certificate => ah_system_adapter::SecurityType::Certificate,
            Self::NotSecure => ah_system_adapter::SecurityType::NotSecure,
            Self::Token => ah_system_adapter::SecurityType::Token,
        }
    }
}
impl Into<ah_system_adapter::OrchestrationFlagKey> for OrchestrationFlag {
    fn into(self) -> ah_system_adapter::OrchestrationFlagKey {
        match self {
            Self::Machmaking => ah_system_adapter::OrchestrationFlagKey::Machmaking,
            Self::MetadataSearch => ah_system_adapter::OrchestrationFlagKey::MetadataSearch,
            Self::OnlyPreferred => ah_system_adapter::OrchestrationFlagKey::OnlyPreferred,
            Self::PingProviders => ah_system_adapter::OrchestrationFlagKey::PingProviders,
            Self::OverrideStore => ah_system_adapter::OrchestrationFlagKey::OverrideStore,
            Self::EnableInterCloud => ah_system_adapter::OrchestrationFlagKey::EnableInterCloud,
            Self::TriggerInterCloud => ah_system_adapter::OrchestrationFlagKey::TriggerInterCloud,
        }
    }
}
