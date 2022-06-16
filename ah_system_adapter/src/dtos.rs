use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceQueryForm {
    #[serde(flatten)]
    pub service_requirements: ServiceRequirements,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ping_providers: Option<bool>,
}

#[derive(Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceQueryList {
    pub service_query_data: Vec<ArrowheadService<EntryTag>>,
    pub unfiltered_hits: u32,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SecurityType {
    NotSecure,
    Certificate,
    Token,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArrowheadService<T> {
    #[serde(flatten)]
    pub entry_tag: T,
    pub service_definition: ServiceDefinitionEntry,
    #[serde(alias = "provider")]
    pub provider_system: ArrowheadSystem<T>,
    pub service_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_of_validity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure: Option<SecurityType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    pub interfaces: Vec<InterfaceEntry>,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EntryTag {
    pub id: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum ServiceDefinitionEntry {
    Value(String),
    #[serde(rename_all = "camelCase")]
    Entry {
        #[serde(flatten)]
        entry_tag: EntryTag,
        service_definition: String,
    },
}
impl ServiceDefinitionEntry {
    pub fn get_service_definition(&self) -> &str {
        match self {
            Self::Value(string) => string,
            Self::Entry {
                service_definition, ..
            } => service_definition,
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArrowheadSystem<T> {
    #[serde(flatten)]
    pub entry_tag: T,
    pub system_name: String,
    pub address: String,
    pub port: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_info: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum InterfaceEntry {
    Value(String),
    #[serde(rename_all = "camelCase")]
    Entry {
        #[serde(flatten)]
        entry_tag: EntryTag,
        interface_name: String,
    },
}
impl InterfaceEntry {
    pub fn get_interface_name(&self) -> &str {
        match self {
            Self::Value(string) => string,
            Self::Entry { interface_name, .. } => interface_name,
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub struct NoEntryTag {}

#[derive(Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequirements {
    pub service_definition_requirement: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface_requirements: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_requirements: Option<Vec<SecurityType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_requirements: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_requirement: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_version_requirement: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version_requirement: Option<u32>,
}

#[derive(Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequestForm {
    pub requester_system: ArrowheadSystem<NoEntryTag>,
    pub requested_service: ServiceRequirements,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_providers: Option<Vec<ArrowheadProvider>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orchestration_flags: Option<HashMap<OrchestrationFlagKey, bool>>,
}

#[derive(Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArrowheadProvider {
    pub provider_cloud: ArrowheadCloud,
    pub provider_system: ArrowheadSystem<NoEntryTag>,
}

#[derive(Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArrowheadCloud {
    pub operator: String,
    pub name: String,
}

#[derive(Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OrchestrationFlagKey {
    Machmaking,
    MetadataSearch,
    OnlyPreferred,
    PingProviders,
    OverrideStore,
    EnableInterCloud,
    TriggerInterCloud,
}

#[derive(Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationResponse {
    pub response: Vec<Orchestration>,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Orchestration {
    pub provider: ArrowheadSystem<EntryTag>,
    pub service: ServiceDefinitionEntry,
    pub service_uri: String,
    pub secure: SecurityType,
    pub metadata: HashMap<String, String>,
    pub interfaces: Vec<InterfaceEntry>,
    pub version: u32,
    pub authorization_tokens: Option<HashMap<String, String>>,
    pub warnings: Vec<OrchestrationWarning>,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrchestrationWarning {
    FromOtherCloud,
    TtlExpired,
    TtlExpiring,
    TtlUnknown,
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ArrowheadServerException {
    pub error_message: String,
    pub error_code: u32,
    pub exception_type: String,
    pub origin: String,
}

#[derive(Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegisterServiceInput {
    pub service_definition: ServiceDefinitionEntry,
    pub service_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_of_validity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure: Option<SecurityType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    pub interfaces: Vec<InterfaceEntry>,
}
impl RegisterServiceInput {
    pub fn to_arrowhead_service(
        self,
        provider_system: ArrowheadSystem<NoEntryTag>,
    ) -> ArrowheadService<NoEntryTag> {
        ArrowheadService {
            provider_system,
            entry_tag: NoEntryTag {},
            service_definition: self.service_definition,
            service_uri: self.service_uri,
            end_of_validity: self.end_of_validity,
            secure: self.secure,
            metadata: self.metadata,
            version: self.version,
            interfaces: self.interfaces,
        }
    }
}

#[derive(Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestOrchestrationInput {
    pub requested_service: ServiceRequirements,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_providers: Option<Vec<ArrowheadProvider>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orchestration_flags: Option<HashMap<OrchestrationFlagKey, bool>>,
}
impl RequestOrchestrationInput {
    pub fn to_service_request_form(
        self,
        requester_system: ArrowheadSystem<NoEntryTag>,
    ) -> ServiceRequestForm {
        ServiceRequestForm {
            requester_system,
            requested_service: self.requested_service,
            preferred_providers: self.preferred_providers,
            orchestration_flags: self.orchestration_flags,
        }
    }
}
