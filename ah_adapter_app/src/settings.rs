use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub service_registry_address: Option<String>,
    pub authorization_address: Option<String>,
    pub orchestrator_address: Option<String>,
    pub system_name: Option<String>,
    pub system_address: Option<String>,
    pub system_port: Option<u32>,
    pub system_authentication_info: Option<String>,
    pub column_width: Option<usize>,
}
impl Settings {
    pub fn merge(self, settings: Settings) -> Self {
        Self {
            service_registry_address: settings
                .service_registry_address
                .or(self.service_registry_address),
            authorization_address: settings
                .authorization_address
                .or(self.authorization_address),
            orchestrator_address: settings.orchestrator_address.or(self.orchestrator_address),
            system_name: settings.system_name.or(self.system_name),
            system_address: settings.system_address.or(self.system_address),
            system_port: settings.system_port.or(self.system_port),
            system_authentication_info: settings
                .system_authentication_info
                .or(self.system_authentication_info),
            column_width: settings.column_width.or(self.column_width),
        }
    }
}
