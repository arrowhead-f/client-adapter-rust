mod dtos;
mod error;

pub use crate::dtos::{
    ArrowheadCloud, ArrowheadProvider, ArrowheadServerException, ArrowheadService, ArrowheadSystem,
    EntryTag, InterfaceEntry, NoEntryTag, Orchestration, OrchestrationFlagKey,
    OrchestrationResponse, OrchestrationWarning, RegisterServiceInput, RequestOrchestrationInput,
    SecurityType, ServiceDefinitionEntry, ServiceQueryForm, ServiceQueryList, ServiceRequestForm,
    ServiceRequirements,
};
pub use crate::error::{Error, Result};

use reqwest::blocking::Client;
use reqwest::Url;

pub struct ArrowheadSystemAdapter {
    pub service_registry_address: Url,
    pub authorization_address: Url,
    pub orchestrator_address: Url,
    pub client_system: ArrowheadSystem<NoEntryTag>,
}

impl ArrowheadSystemAdapter {
    pub fn new(
        service_registry_address: &str,
        authorization_address: &str,
        orchestrator_address: &str,
        client_system: ArrowheadSystem<NoEntryTag>,
    ) -> Result<Self> {
        Ok(ArrowheadSystemAdapter {
            service_registry_address: service_registry_address.try_into()?,
            authorization_address: authorization_address.try_into()?,
            orchestrator_address: orchestrator_address.try_into()?,
            client_system,
        })
    }
    pub fn echo_service_registry(&self) -> Result<()> {
        let client = Client::new();
        client
            .get(self.service_registry_address.join("echo")?)
            .send()?
            .error_for_status()?;
        Ok(())
    }

    pub fn query_service(&self, service_query_form: &ServiceQueryForm) -> Result<ServiceQueryList> {
        let client = Client::new();
        let response = client
            .post(self.service_registry_address.join("query")?)
            .json(&service_query_form)
            .send()?;
        if response.status().is_client_error() {
            Err(Error::ArrowheadError(response.json()?))
        } else {
            Ok(response.json()?)
        }
    }

    pub fn register_service(
        &self,
        input: RegisterServiceInput,
    ) -> Result<ArrowheadService<EntryTag>> {
        let service = input.to_arrowhead_service(self.client_system.clone());
        let client = Client::new();
        let response = client
            .post(self.service_registry_address.join("register")?)
            .json(&service)
            .send()?;
        if response.status().is_client_error() {
            Err(Error::ArrowheadError(response.json()?))
        } else {
            Ok(response.json()?)
        }
    }

    pub fn unregister_service(&self, service_definition: &str) -> Result<()> {
        let client = Client::new();
        let mut url = self.service_registry_address.join("unregister")?;
        url.query_pairs_mut().extend_pairs(&[
            ("service_definition", service_definition),
            ("system_name", &self.client_system.system_name),
            ("address", &self.client_system.address),
            ("port", &self.client_system.port.to_string()),
        ]);
        let response = client.delete(url).send()?;
        if response.status().is_client_error() {
            Err(Error::ArrowheadError(response.json()?))
        } else {
            Ok(())
        }
    }

    pub fn echo_authorization(&self) -> Result<()> {
        let client = Client::new();
        client
            .get(self.authorization_address.join("echo")?)
            .send()?
            .error_for_status()?;
        Ok(())
    }

    pub fn get_public_key(&self) -> Result<String> {
        let client = Client::new();
        let response = client
            .get(self.authorization_address.join("publickey")?)
            .send()?
            .error_for_status()?;
        Ok(response.text()?)
    }

    pub fn echo_orchestrator(&self) -> Result<()> {
        let client = Client::new();
        client
            .get(self.orchestrator_address.join("echo")?)
            .send()?
            .error_for_status()?;
        Ok(())
    }

    pub fn request_orchestration(
        &self,
        input: RequestOrchestrationInput,
    ) -> Result<OrchestrationResponse> {
        let service_request_form = input.to_service_request_form(self.client_system.clone());
        let client = Client::new();
        let response = client
            .post(self.orchestrator_address.join("orchestration")?)
            .json(&service_request_form)
            .send()?;
        if response.status().is_client_error() {
            Err(Error::ArrowheadError(response.json()?))
        } else {
            Ok(response.json()?)
        }
    }

    pub fn request_orchestration_by_id(&self, id: i64) -> Result<OrchestrationResponse> {
        let client = Client::new();
        let response = client
            .get(
                self.orchestrator_address
                    .join(&format!("orchestration/{}", id))?,
            )
            .send()?;
        if response.status().is_client_error() {
            Err(Error::ArrowheadError(response.json()?))
        } else {
            Ok(response.json()?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn echo_service_registry() {
        let mock = mockito::mock("GET", "/echo").create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            &mockito::server_url(),
            "http://dontcare",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.echo_service_registry();

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn echo_service_registry_http_error() {
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://invalid#",
            "http://dontcare",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.echo_service_registry();

        assert!(matches!(result, Err(Error::HttpError(_))));
    }

    #[test]
    fn query_service() {
        let mock = mockito::mock("POST", "/query")
            .match_header("content-type", "application/json")
            .match_body(Matcher::Json(json!({
                "serviceDefinitionRequirement": "string",
                "interfaceRequirements": [
                  "string"
                ],
                "securityRequirements": [
                  "NOT_SECURE"
                ],
                "metadataRequirements": {
                  "additionalProp1": "string",
                  "additionalProp2": "string",
                  "additionalProp3": "string"
                },
                "versionRequirement": 0,
                "maxVersionRequirement": 0,
                "minVersionRequirement": 0,
                "pingProviders": true
            })))
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "serviceQueryData": [
                      {
                        "id": 0,
                        "serviceDefinition": {
                          "id": 0,
                          "serviceDefinition": "string",
                          "createdAt": "string",
                          "updatedAt": "string"
                        },
                        "provider": {
                          "id": 0,
                          "systemName": "string",
                          "address": "string",
                          "port": 0,
                          "authenticationInfo": "string",
                          "createdAt": "string",
                          "updatedAt": "string"
                        },
                        "serviceUri": "string",
                        "endOfValidity": "string",
                        "secure": "NOT_SECURE",
                        "metadata": {
                          "additionalProp1": "string",
                          "additionalProp2": "string",
                          "additionalProp3": "string"
                        },
                        "version": 0,
                        "interfaces": [
                          {
                            "id": 0,
                            "interfaceName": "string",
                            "createdAt": "string",
                            "updatedAt": "string"
                          }
                        ],
                        "createdAt": "string",
                        "updatedAt": "string"
                       }
                    ],
                    "unfilteredHits": 0
                })
                .to_string(),
            )
            .create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            &mockito::server_url(),
            "http://dontcare",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.query_service(&ServiceQueryForm {
            service_requirements: ServiceRequirements {
                service_definition_requirement: "string".to_owned(),
                interface_requirements: Some(vec!["string".to_owned()]),
                security_requirements: Some(vec![SecurityType::NotSecure]),
                metadata_requirements: Some(HashMap::from([
                    ("additionalProp1".to_owned(), "string".to_owned()),
                    ("additionalProp2".to_owned(), "string".to_owned()),
                    ("additionalProp3".to_owned(), "string".to_owned()),
                ])),
                version_requirement: Some(0),
                max_version_requirement: Some(0),
                min_version_requirement: Some(0),
            },
            ping_providers: Some(true),
        });
        let expected_service_query_list = ServiceQueryList {
            service_query_data: vec![ArrowheadService {
                entry_tag: EntryTag {
                    id: 0,
                    created_at: "string".to_owned(),
                    updated_at: "string".to_owned(),
                },
                service_definition: ServiceDefinitionEntry::Entry {
                    entry_tag: EntryTag {
                        id: 0,
                        created_at: "string".to_owned(),
                        updated_at: "string".to_owned(),
                    },
                    service_definition: "string".to_owned(),
                },
                provider_system: ArrowheadSystem {
                    entry_tag: EntryTag {
                        id: 0,
                        created_at: "string".to_owned(),
                        updated_at: "string".to_owned(),
                    },
                    system_name: "string".to_owned(),
                    address: "string".to_owned(),
                    port: 0,
                    authentication_info: Some("string".to_owned()),
                },
                service_uri: "string".to_owned(),
                end_of_validity: Some("string".to_owned()),
                secure: Some(SecurityType::NotSecure),
                metadata: Some(HashMap::from([
                    ("additionalProp1".to_owned(), "string".to_owned()),
                    ("additionalProp2".to_owned(), "string".to_owned()),
                    ("additionalProp3".to_owned(), "string".to_owned()),
                ])),
                version: Some(0),
                interfaces: vec![InterfaceEntry::Entry {
                    entry_tag: EntryTag {
                        id: 0,
                        created_at: "string".to_owned(),
                        updated_at: "string".to_owned(),
                    },
                    interface_name: "string".to_owned(),
                }],
            }],
            unfiltered_hits: 0,
        };
        assert!(
            matches!(result, Ok(service_query_list) if service_query_list == expected_service_query_list)
        );
        mock.assert();
    }

    #[test]
    fn query_service_http_error() {
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://invalid#",
            "http://dontcare",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.query_service(&ServiceQueryForm {
            service_requirements: ServiceRequirements {
                service_definition_requirement: "string".to_owned(),
                interface_requirements: Some(vec!["string".to_owned()]),
                security_requirements: Some(vec![SecurityType::NotSecure]),
                metadata_requirements: Some(HashMap::from([
                    ("additionalProp1".to_owned(), "string".to_owned()),
                    ("additionalProp2".to_owned(), "string".to_owned()),
                    ("additionalProp3".to_owned(), "string".to_owned()),
                ])),
                version_requirement: Some(0),
                max_version_requirement: Some(0),
                min_version_requirement: Some(0),
            },
            ping_providers: Some(true),
        });

        assert!(matches!(result, Err(Error::HttpError(_))));
    }

    #[test]
    fn query_service_arrowhead_error() {
        let mock = mockito::mock("POST", "/query")
            .match_header("content-type", "application/json")
            .match_body(Matcher::Json(json!({
                "serviceDefinitionRequirement": "string",
                "interfaceRequirements": [
                  "string"
                ],
                "securityRequirements": [
                  "NOT_SECURE"
                ],
                "metadataRequirements": {
                  "additionalProp1": "string",
                  "additionalProp2": "string",
                  "additionalProp3": "string"
                },
                "versionRequirement": 0,
                "maxVersionRequirement": 0,
                "minVersionRequirement": 0,
                "pingProviders": true
            })))
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                  "errorMessage": "string",
                  "errorCode": 0,
                  "exceptionType": "string",
                  "origin": "string"
                })
                .to_string(),
            )
            .create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            &mockito::server_url(),
            "http://dontcare",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.query_service(&ServiceQueryForm {
            service_requirements: ServiceRequirements {
                service_definition_requirement: "string".to_owned(),
                interface_requirements: Some(vec!["string".to_owned()]),
                security_requirements: Some(vec![SecurityType::NotSecure]),
                metadata_requirements: Some(HashMap::from([
                    ("additionalProp1".to_owned(), "string".to_owned()),
                    ("additionalProp2".to_owned(), "string".to_owned()),
                    ("additionalProp3".to_owned(), "string".to_owned()),
                ])),
                version_requirement: Some(0),
                max_version_requirement: Some(0),
                min_version_requirement: Some(0),
            },
            ping_providers: Some(true),
        });
        let expected_arrowhead_server_exception = ArrowheadServerException {
            error_message: "string".to_owned(),
            error_code: 0,
            exception_type: "string".to_owned(),
            origin: "string".to_owned(),
        };
        assert!(
            matches!(result, Err(Error::ArrowheadError(arrowhead_server_exception)) if arrowhead_server_exception == expected_arrowhead_server_exception)
        );
        mock.assert();
    }

    #[test]
    fn register_service() {
        let mock = mockito::mock("POST", "/register")
            .match_header("content-type", "application/json")
            .match_body(Matcher::Json(json!({
              "serviceDefinition": "string",
              "providerSystem": {
                "systemName": "string",
                "address": "string",
                "port": 0,
                "authenticationInfo": "string"
              },
              "serviceUri": "string",
              "endOfValidity": "string",
              "secure": "NOT_SECURE",
              "metadata": {
                "additionalProp1": "string",
                "additionalProp2": "string",
                "additionalProp3": "string"
              },
              "version": 0,
              "interfaces": [
                "string"
              ]
            })))
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                  "id": 0,
                  "serviceDefinition": {
                    "id": 0,
                    "serviceDefinition": "string",
                    "createdAt": "string",
                    "updatedAt": "string"
                  },
                  "provider": {
                    "id": 0,
                    "systemName": "string",
                    "address": "string",
                    "port": 0,
                    "authenticationInfo": "string",
                    "createdAt": "string",
                    "updatedAt": "string"
                  },
                  "serviceUri": "string",
                  "endOfValidity": "string",
                  "secure": "NOT_SECURE",
                  "metadata": {
                    "additionalProp1": "string",
                    "additionalProp2": "string",
                    "additionalProp3": "string"
                  },
                  "version": 0,
                  "interfaces": [
                    {
                      "id": 0,
                      "interfaceName": "string",
                      "createdAt": "string",
                      "updatedAt": "string"
                 }
                 ],
                 "createdAt": "string",
                 "updatedAt": "string"
                })
                .to_string(),
            )
            .create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            &mockito::server_url(),
            "http://dontcare",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.register_service(RegisterServiceInput {
            service_definition: ServiceDefinitionEntry::Value("string".to_owned()),
            service_uri: "string".to_owned(),
            end_of_validity: Some("string".to_owned()),
            secure: Some(SecurityType::NotSecure),
            metadata: Some(HashMap::from([
                ("additionalProp1".to_owned(), "string".to_owned()),
                ("additionalProp2".to_owned(), "string".to_owned()),
                ("additionalProp3".to_owned(), "string".to_owned()),
            ])),
            version: Some(0),
            interfaces: vec![InterfaceEntry::Value("string".to_owned())],
        });
        let expected_service_entry = ArrowheadService {
            entry_tag: EntryTag {
                id: 0,
                created_at: "string".to_owned(),
                updated_at: "string".to_owned(),
            },
            service_definition: ServiceDefinitionEntry::Entry {
                entry_tag: EntryTag {
                    id: 0,
                    created_at: "string".to_owned(),
                    updated_at: "string".to_owned(),
                },
                service_definition: "string".to_owned(),
            },
            provider_system: ArrowheadSystem {
                entry_tag: EntryTag {
                    id: 0,
                    created_at: "string".to_owned(),
                    updated_at: "string".to_owned(),
                },
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
            service_uri: "string".to_owned(),
            end_of_validity: Some("string".to_owned()),
            secure: Some(SecurityType::NotSecure),
            metadata: Some(HashMap::from([
                ("additionalProp1".to_owned(), "string".to_owned()),
                ("additionalProp2".to_owned(), "string".to_owned()),
                ("additionalProp3".to_owned(), "string".to_owned()),
            ])),
            version: Some(0),
            interfaces: vec![InterfaceEntry::Entry {
                entry_tag: EntryTag {
                    id: 0,
                    created_at: "string".to_owned(),
                    updated_at: "string".to_owned(),
                },
                interface_name: "string".to_owned(),
            }],
        };
        assert!(matches!(result, Ok(service_entry) if service_entry == expected_service_entry));
        mock.assert();
    }

    #[test]
    fn register_service_http_error() {
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://invalid#",
            "http://dontcare",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.register_service(RegisterServiceInput {
            service_definition: ServiceDefinitionEntry::Value("string".to_owned()),
            service_uri: "string".to_owned(),
            end_of_validity: Some("string".to_owned()),
            secure: Some(SecurityType::NotSecure),
            metadata: Some(HashMap::from([
                ("additionalProp1".to_owned(), "string".to_owned()),
                ("additionalProp2".to_owned(), "string".to_owned()),
                ("additionalProp3".to_owned(), "string".to_owned()),
            ])),
            version: Some(0),
            interfaces: vec![InterfaceEntry::Value("string".to_owned())],
        });

        assert!(matches!(result, Err(Error::HttpError(_))));
    }

    #[test]
    fn register_service_arrowhead_error() {
        let mock = mockito::mock("POST", "/register")
            .match_header("content-type", "application/json")
            .match_body(Matcher::Json(json!({
              "serviceDefinition": "string",
              "providerSystem": {
                "systemName": "string",
                "address": "string",
                "port": 0,
                "authenticationInfo": "string"
              },
              "serviceUri": "string",
              "endOfValidity": "string",
              "secure": "NOT_SECURE",
              "metadata": {
                "additionalProp1": "string",
                "additionalProp2": "string",
                "additionalProp3": "string"
              },
              "version": 0,
              "interfaces": [
                "string"
              ]
            })))
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                  "errorMessage": "string",
                  "errorCode": 0,
                  "exceptionType": "string",
                  "origin": "string"
                })
                .to_string(),
            )
            .create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            &mockito::server_url(),
            "http://dontcare",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.register_service(RegisterServiceInput {
            service_definition: ServiceDefinitionEntry::Value("string".to_owned()),
            service_uri: "string".to_owned(),
            end_of_validity: Some("string".to_owned()),
            secure: Some(SecurityType::NotSecure),
            metadata: Some(HashMap::from([
                ("additionalProp1".to_owned(), "string".to_owned()),
                ("additionalProp2".to_owned(), "string".to_owned()),
                ("additionalProp3".to_owned(), "string".to_owned()),
            ])),
            version: Some(0),
            interfaces: vec![InterfaceEntry::Value("string".to_owned())],
        });
        let expected_arrowhead_server_exception = ArrowheadServerException {
            error_message: "string".to_owned(),
            error_code: 0,
            exception_type: "string".to_owned(),
            origin: "string".to_owned(),
        };
        assert!(
            matches!(result, Err(Error::ArrowheadError(arrowhead_server_exception)) if arrowhead_server_exception == expected_arrowhead_server_exception)
        );
        mock.assert();
    }

    #[test]
    fn unregister_service() {
        let mock = mockito::mock("DELETE", "/unregister")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("service_definition".into(), "string".into()),
                Matcher::UrlEncoded("system_name".into(), "string".into()),
                Matcher::UrlEncoded("address".into(), "string".into()),
                Matcher::UrlEncoded("port".into(), "0".into()),
            ]))
            .create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            &mockito::server_url(),
            "http://dontcare",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.unregister_service("string");

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn unregister_service_http_error() {
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://invalid#",
            "http://dontcare",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.unregister_service("string");

        assert!(matches!(result, Err(Error::HttpError(_))));
    }

    #[test]
    fn unregister_service_arrowhead_error() {
        let mock = mockito::mock("DELETE", "/unregister")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("service_definition".into(), "string".into()),
                Matcher::UrlEncoded("system_name".into(), "string".into()),
                Matcher::UrlEncoded("address".into(), "string".into()),
                Matcher::UrlEncoded("port".into(), "0".into()),
            ]))
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                  "errorMessage": "string",
                  "errorCode": 0,
                  "exceptionType": "string",
                  "origin": "string"
                })
                .to_string(),
            )
            .create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            &mockito::server_url(),
            "http://dontcare",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.unregister_service("string");
        let expected_arrowhead_server_exception = ArrowheadServerException {
            error_message: "string".to_owned(),
            error_code: 0,
            exception_type: "string".to_owned(),
            origin: "string".to_owned(),
        };
        assert!(
            matches!(result, Err(Error::ArrowheadError(arrowhead_server_exception)) if arrowhead_server_exception == expected_arrowhead_server_exception)
        );
        mock.assert();
    }

    #[test]
    fn echo_authorization() {
        let mock = mockito::mock("GET", "/echo").create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://dontcare",
            &mockito::server_url(),
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.echo_authorization();

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn echo_authorization_http_error() {
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://dontcare",
            "http://invalid#",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.echo_authorization();

        assert!(matches!(result, Err(Error::HttpError(_))));
    }

    #[test]
    fn get_public_key() {
        let mock = mockito::mock("GET", "/publickey")
            .with_body("testkey")
            .create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://dontcare",
            &mockito::server_url(),
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.get_public_key();
        let expected_public_key = "testkey";

        assert!(matches!(result, Ok(public_key) if public_key == expected_public_key));
        mock.assert();
    }

    #[test]
    fn get_public_key_http_error() {
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://dontcare",
            "http://invalid#",
            "http://dontcare",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.get_public_key();

        assert!(matches!(result, Err(Error::HttpError(_))));
    }

    #[test]
    fn echo_orchestrator() {
        let mock = mockito::mock("GET", "/echo").create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://dontcare",
            "http://dontcare",
            &mockito::server_url(),
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.echo_orchestrator();

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn echo_orchestrator_http_error() {
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://dontcare",
            "http://dontcare",
            "http://invalid#",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.echo_orchestrator();

        assert!(matches!(result, Err(Error::HttpError(_))));
    }

    #[test]
    fn request_orchestration() {
        let mock = mockito::mock("POST", "/orchestration")
            .match_header("content-type", "application/json")
            .match_body(Matcher::Json(json!({
              "requesterSystem": {
                "systemName": "string",
                "address": "string",
                "port": 0,
                "authenticationInfo": "string"
              },
              "requestedService": {
                "serviceDefinitionRequirement": "string",
                "interfaceRequirements": [
                  "string"
                ],
                "securityRequirements": [
                  "NOT_SECURE", "CERTIFICATE", "TOKEN"
                ],
                "metadataRequirements": {
                  "additionalProp1": "string",
                  "additionalProp2": "string",
                  "additionalProp3": "string"
                },
                "versionRequirement": 0,
                "maxVersionRequirement": 0,
               "minVersionRequirement": 0
              },
              "preferredProviders": [
                {
                  "providerCloud": {
                    "operator": "string",
                    "name": "string"
                  },
                  "providerSystem": {
                    "systemName": "string",
                    "address": "string",
                    "port": 0
                  }
                }
              ],
              "orchestrationFlags": {
                  "machmaking": true,
                  "metadataSearch": true,
                  "onlyPreferred": true,
                  "pingProviders": true,
                  "overrideStore": true,
                  "enableInterCloud": true,
                  "triggerInterCloud": true
              }
            })))
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                  "response": [
                    {
                      "provider": {
                        "id": 0,
                        "systemName": "string",
                        "address": "string",
                        "port": 0,
                        "authenticationInfo": "string",
                        "createdAt": "string",
                        "updatedAt": "string"
                      },
                      "service": {
                        "id": 0,
                        "serviceDefinition": "string",
                        "createdAt": "string",
                        "updatedAt": "string"
                      },
                      "serviceUri": "string",
                      "secure": "TOKEN",
                      "metadata": {
                        "additionalProp1": "string",
                        "additionalProp2": "string",
                        "additionalProp3": "string"
                      },
                      "interfaces": [
                        {
                          "id": 0,
                          "createdAt": "string",
                          "interfaceName": "string",
                          "updatedAt": "string"
                        }
                      ],
                      "version": 0,
                      "authorizationTokens": {
                        "interfaceName1": "token1",
                        "interfaceName2": "token2"
                      },
                      "warnings": [
                        "FROM_OTHER_CLOUD", "TTL_UNKNOWN"
                      ]
                    }
                  ]
                })
                .to_string(),
            )
            .create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://dontcare",
            "http://dontcare",
            &mockito::server_url(),
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.request_orchestration(RequestOrchestrationInput {
            requested_service: ServiceRequirements {
                service_definition_requirement: "string".to_owned(),
                interface_requirements: Some(vec!["string".to_owned()]),
                security_requirements: Some(vec![
                    SecurityType::NotSecure,
                    SecurityType::Certificate,
                    SecurityType::Token,
                ]),
                metadata_requirements: Some(HashMap::from([
                    ("additionalProp1".to_owned(), "string".to_owned()),
                    ("additionalProp2".to_owned(), "string".to_owned()),
                    ("additionalProp3".to_owned(), "string".to_owned()),
                ])),
                version_requirement: Some(0),
                max_version_requirement: Some(0),
                min_version_requirement: Some(0),
            },
            preferred_providers: Some(vec![ArrowheadProvider {
                provider_cloud: ArrowheadCloud {
                    operator: "string".to_owned(),
                    name: "string".to_owned(),
                },
                provider_system: ArrowheadSystem {
                    entry_tag: NoEntryTag {},
                    system_name: "string".to_owned(),
                    address: "string".to_owned(),
                    port: 0,
                    authentication_info: None,
                },
            }]),
            orchestration_flags: Some(HashMap::from([
                (OrchestrationFlagKey::Machmaking, true),
                (OrchestrationFlagKey::MetadataSearch, true),
                (OrchestrationFlagKey::OnlyPreferred, true),
                (OrchestrationFlagKey::PingProviders, true),
                (OrchestrationFlagKey::OverrideStore, true),
                (OrchestrationFlagKey::EnableInterCloud, true),
                (OrchestrationFlagKey::TriggerInterCloud, true),
            ])),
        });
        let expected_orchestration_response = OrchestrationResponse {
            response: vec![Orchestration {
                provider: ArrowheadSystem {
                    entry_tag: EntryTag {
                        id: 0,
                        created_at: "string".to_owned(),
                        updated_at: "string".to_owned(),
                    },
                    system_name: "string".to_owned(),
                    address: "string".to_owned(),
                    port: 0,
                    authentication_info: Some("string".to_owned()),
                },
                service: ServiceDefinitionEntry::Entry {
                    entry_tag: EntryTag {
                        id: 0,
                        created_at: "string".to_owned(),
                        updated_at: "string".to_owned(),
                    },
                    service_definition: "string".to_owned(),
                },
                service_uri: "string".to_owned(),
                secure: SecurityType::Token,
                metadata: HashMap::from([
                    ("additionalProp1".to_owned(), "string".to_owned()),
                    ("additionalProp2".to_owned(), "string".to_owned()),
                    ("additionalProp3".to_owned(), "string".to_owned()),
                ]),
                interfaces: vec![InterfaceEntry::Entry {
                    entry_tag: EntryTag {
                        id: 0,
                        created_at: "string".to_owned(),
                        updated_at: "string".to_owned(),
                    },
                    interface_name: "string".to_owned(),
                }],
                version: 0,
                authorization_tokens: Some(HashMap::from([
                    ("interfaceName1".to_owned(), "token1".to_owned()),
                    ("interfaceName2".to_owned(), "token2".to_owned()),
                ])),
                warnings: vec![
                    OrchestrationWarning::FromOtherCloud,
                    OrchestrationWarning::TtlUnknown,
                ],
            }],
        };
        assert!(
            matches!(result, Ok(orchestration_response) if orchestration_response == expected_orchestration_response)
        );
        mock.assert();
    }

    #[test]
    fn request_orchestration_http_error() {
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://dontcare",
            "http://dontcare",
            "http://invalid#",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.request_orchestration(RequestOrchestrationInput {
            requested_service: ServiceRequirements {
                service_definition_requirement: "string".to_owned(),
                interface_requirements: Some(vec!["string".to_owned()]),
                security_requirements: Some(vec![
                    SecurityType::NotSecure,
                    SecurityType::Certificate,
                    SecurityType::Token,
                ]),
                metadata_requirements: Some(HashMap::from([
                    ("additionalProp1".to_owned(), "string".to_owned()),
                    ("additionalProp2".to_owned(), "string".to_owned()),
                    ("additionalProp3".to_owned(), "string".to_owned()),
                ])),
                version_requirement: Some(0),
                max_version_requirement: Some(0),
                min_version_requirement: Some(0),
            },
            preferred_providers: Some(vec![ArrowheadProvider {
                provider_cloud: ArrowheadCloud {
                    operator: "string".to_owned(),
                    name: "string".to_owned(),
                },
                provider_system: ArrowheadSystem {
                    entry_tag: NoEntryTag {},
                    system_name: "string".to_owned(),
                    address: "string".to_owned(),
                    port: 0,
                    authentication_info: None,
                },
            }]),
            orchestration_flags: Some(HashMap::from([
                (OrchestrationFlagKey::Machmaking, true),
                (OrchestrationFlagKey::MetadataSearch, true),
                (OrchestrationFlagKey::OnlyPreferred, true),
                (OrchestrationFlagKey::PingProviders, true),
                (OrchestrationFlagKey::OverrideStore, true),
                (OrchestrationFlagKey::EnableInterCloud, true),
                (OrchestrationFlagKey::TriggerInterCloud, true),
            ])),
        });

        assert!(matches!(result, Err(Error::HttpError(_))));
    }

    #[test]
    fn request_orchestration_arrowhead_error() {
        let mock = mockito::mock("POST", "/orchestration")
            .match_header("content-type", "application/json")
            .match_body(Matcher::Json(json!({
              "requesterSystem": {
                "systemName": "string",
                "address": "string",
                "port": 0,
                "authenticationInfo": "string"
              },
              "requestedService": {
                "serviceDefinitionRequirement": "string",
                "interfaceRequirements": [
                  "string"
                ],
                "securityRequirements": [
                  "NOT_SECURE", "CERTIFICATE", "TOKEN"
                ],
                "metadataRequirements": {
                  "additionalProp1": "string",
                  "additionalProp2": "string",
                  "additionalProp3": "string"
                },
                "versionRequirement": 0,
                "maxVersionRequirement": 0,
               "minVersionRequirement": 0
              },
              "preferredProviders": [
                {
                  "providerCloud": {
                    "operator": "string",
                    "name": "string"
                  },
                  "providerSystem": {
                    "systemName": "string",
                    "address": "string",
                    "port": 0
                  }
                }
              ],
              "orchestrationFlags": {
                  "machmaking": true,
                  "metadataSearch": true,
                  "onlyPreferred": true,
                  "pingProviders": true,
                  "overrideStore": true,
                  "enableInterCloud": true,
                  "triggerInterCloud": true
              }
            })))
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                  "errorMessage": "string",
                  "errorCode": 0,
                  "exceptionType": "string",
                  "origin": "string"
                })
                .to_string(),
            )
            .create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://dontcare",
            "http://dontcare",
            &mockito::server_url(),
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.request_orchestration(RequestOrchestrationInput {
            requested_service: ServiceRequirements {
                service_definition_requirement: "string".to_owned(),
                interface_requirements: Some(vec!["string".to_owned()]),
                security_requirements: Some(vec![
                    SecurityType::NotSecure,
                    SecurityType::Certificate,
                    SecurityType::Token,
                ]),
                metadata_requirements: Some(HashMap::from([
                    ("additionalProp1".to_owned(), "string".to_owned()),
                    ("additionalProp2".to_owned(), "string".to_owned()),
                    ("additionalProp3".to_owned(), "string".to_owned()),
                ])),
                version_requirement: Some(0),
                max_version_requirement: Some(0),
                min_version_requirement: Some(0),
            },
            preferred_providers: Some(vec![ArrowheadProvider {
                provider_cloud: ArrowheadCloud {
                    operator: "string".to_owned(),
                    name: "string".to_owned(),
                },
                provider_system: ArrowheadSystem {
                    entry_tag: NoEntryTag {},
                    system_name: "string".to_owned(),
                    address: "string".to_owned(),
                    port: 0,
                    authentication_info: None,
                },
            }]),
            orchestration_flags: Some(HashMap::from([
                (OrchestrationFlagKey::Machmaking, true),
                (OrchestrationFlagKey::MetadataSearch, true),
                (OrchestrationFlagKey::OnlyPreferred, true),
                (OrchestrationFlagKey::PingProviders, true),
                (OrchestrationFlagKey::OverrideStore, true),
                (OrchestrationFlagKey::EnableInterCloud, true),
                (OrchestrationFlagKey::TriggerInterCloud, true),
            ])),
        });
        let expected_arrowhead_server_exception = ArrowheadServerException {
            error_message: "string".to_owned(),
            error_code: 0,
            exception_type: "string".to_owned(),
            origin: "string".to_owned(),
        };
        assert!(
            matches!(result, Err(Error::ArrowheadError(arrowhead_server_exception)) if arrowhead_server_exception == expected_arrowhead_server_exception)
        );
        mock.assert();
    }

    #[test]
    fn request_orchestration_by_id() {
        let mock = mockito::mock("GET", "/orchestration/0")
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                  "response": [
                    {
                      "provider": {
                        "id": 0,
                        "systemName": "string",
                        "address": "string",
                        "port": 0,
                        "authenticationInfo": "string",
                        "createdAt": "string",
                        "updatedAt": "string"
                      },
                      "service": {
                        "id": 0,
                        "serviceDefinition": "string",
                        "createdAt": "string",
                        "updatedAt": "string"
                      },
                      "serviceUri": "string",
                      "secure": "TOKEN",
                      "metadata": {
                        "additionalProp1": "string",
                        "additionalProp2": "string",
                        "additionalProp3": "string"
                      },
                      "interfaces": [
                        {
                          "id": 0,
                          "createdAt": "string",
                          "interfaceName": "string",
                          "updatedAt": "string"
                        }
                      ],
                      "version": 0,
                      "authorizationTokens": {
                        "interfaceName1": "token1",
                        "interfaceName2": "token2"
                      },
                      "warnings": [
                        "FROM_OTHER_CLOUD", "TTL_UNKNOWN"
                      ]
                    }
                  ]
                })
                .to_string(),
            )
            .create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://dontcare",
            "http://dontcare",
            &mockito::server_url(),
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.request_orchestration_by_id(0);
        let expected_orchestration_response = OrchestrationResponse {
            response: vec![Orchestration {
                provider: ArrowheadSystem {
                    entry_tag: EntryTag {
                        id: 0,
                        created_at: "string".to_owned(),
                        updated_at: "string".to_owned(),
                    },
                    system_name: "string".to_owned(),
                    address: "string".to_owned(),
                    port: 0,
                    authentication_info: Some("string".to_owned()),
                },
                service: ServiceDefinitionEntry::Entry {
                    entry_tag: EntryTag {
                        id: 0,
                        created_at: "string".to_owned(),
                        updated_at: "string".to_owned(),
                    },
                    service_definition: "string".to_owned(),
                },
                service_uri: "string".to_owned(),
                secure: SecurityType::Token,
                metadata: HashMap::from([
                    ("additionalProp1".to_owned(), "string".to_owned()),
                    ("additionalProp2".to_owned(), "string".to_owned()),
                    ("additionalProp3".to_owned(), "string".to_owned()),
                ]),
                interfaces: vec![InterfaceEntry::Entry {
                    entry_tag: EntryTag {
                        id: 0,
                        created_at: "string".to_owned(),
                        updated_at: "string".to_owned(),
                    },
                    interface_name: "string".to_owned(),
                }],
                version: 0,
                authorization_tokens: Some(HashMap::from([
                    ("interfaceName1".to_owned(), "token1".to_owned()),
                    ("interfaceName2".to_owned(), "token2".to_owned()),
                ])),
                warnings: vec![
                    OrchestrationWarning::FromOtherCloud,
                    OrchestrationWarning::TtlUnknown,
                ],
            }],
        };
        assert!(
            matches!(result, Ok(orchestration_response) if orchestration_response == expected_orchestration_response)
        );
        mock.assert();
    }

    #[test]
    fn request_orchestration_by_id_http_error() {
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://dontcare",
            "http://dontcare",
            "http://invalid#",
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.request_orchestration_by_id(0);

        assert!(matches!(result, Err(Error::HttpError(_))));
    }

    #[test]
    fn request_orchestration_by_id_arrowhead_error() {
        let mock = mockito::mock("GET", "/orchestration/0")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                  "errorMessage": "string",
                  "errorCode": 0,
                  "exceptionType": "string",
                  "origin": "string"
                })
                .to_string(),
            )
            .create();
        let ah_adapter = ArrowheadSystemAdapter::new(
            "http://dontcare",
            "http://dontcare",
            &mockito::server_url(),
            ArrowheadSystem {
                entry_tag: NoEntryTag {},
                system_name: "string".to_owned(),
                address: "string".to_owned(),
                port: 0,
                authentication_info: Some("string".to_owned()),
            },
        )
        .unwrap();
        let result = ah_adapter.request_orchestration_by_id(0);
        let expected_arrowhead_server_exception = ArrowheadServerException {
            error_message: "string".to_owned(),
            error_code: 0,
            exception_type: "string".to_owned(),
            origin: "string".to_owned(),
        };
        assert!(
            matches!(result, Err(Error::ArrowheadError(arrowhead_server_exception)) if arrowhead_server_exception == expected_arrowhead_server_exception)
        );
        mock.assert();
    }
}
