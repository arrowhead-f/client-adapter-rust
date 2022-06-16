
# ah_adapter_app
## User manual
### Overview

_ah_adapter_app_ is a command line tool, which can be used by client systems to get services registered into an Arrowhead local cloud, request orchestration to start communicating with different services and use other features of Arrowhead Framework's mandatory core systems. The app can be used as a classic command line tool.

The main functions of _ah_adapter_app_ are the followings:

  - service registration/deregistration
  - public key request for the system
  - orchestration request

The supported Arrowhead version is 4.4.0. The documentation of the Arrowhead framework is available [here](https://github.com/eclipse-arrowhead/core-java-spring).

### Basic usage

The basic syntax of the commands is the following (standard command line argument syntax):

```bash
ah <command> <subcommand> --<parameter-name1> <value1> --<parameter-name2> <value2> ...
```

For example:

```bash
ah orchestrate system --name MyOrchestration --interfaces https-secure-json --service-name my-service
```

The parameter values have one of the following types:
|Parameter type|Syntax|
|---|---|
|**Text**|```--<parameter-name> <parameter-value>```|
|**Number**|```--<parameter-name> <parameter-value>```|
|**Enumerator**|```--<parameter-name> <parameter-value>```|
|**Map**| ```--<parameter-name>=<key1>:<value1>,<key2>:<value2>... ``` or ```--<parameter-name> <key1>:<value1> --<parameter-name> <key2>:<value2>... ```|
|**List**| ```--<parameter-name>=<value1>,<value2>... ``` or ```--<parameter-name> <value1> --<parameter-name> <value2>... ```|

A few examples:

  - **Text**:	 ```--name MyFavouriteDevice```
  - **Number**:	```--system-port 8435```
  - **Map**:    ```--metadata=property1:value1,property2:value2```
  - **List**:    ```--flags=onlyPreferred,pingProviders```


### Initialization

When you use the app for the first time, you must set a few settings parameter for the Arrowhead requests to work properly (*set settings*). These are 
- the *address* of Service Registry, Orchestrator and Authorization systems ('service-registry-address', 'orchestrator-address', 'authorization-address'),
- the *name*, *address* and *port* of the client system ('system-name', 'system-address', 'system-port')

See the [Set settings](#set-settings) section.


### Command reference

|Command Name|Parameters|
|---|---|
|**set settings**|service-registry-address, orchestrator-address, authorization-address, system-name, system-address, system-port|
|**register service**|**name**, interfaces, security-type, end-of-validity, metadata, **uri**, version|
|**request orchestration**|**name**, **service-name**, interfaces, flags, max-version, metadata, min-version, preferred-provider, security-types, version|
|**request orchestration-id**|**name**, **id**|
|**request public-key**||
|**show settings**||
|**show services**|*name*|
|**show orchestrations**|*name*|
|**unregister service**|***name***|

:point_up: **Bold commands/parameters are mandatory**

:point_up: *Italic parameters' names are not to be explicity written out*


#### Set
Set internal variables.

##### Set settings
Set the settings.

|Parameter|Type|Description|
|---|---|---|
|**service-registry-address**|text|Address of Service Registry core system|
|**orchestrator-address**|text|Address of Orchestrator core system|
|**authorization-address**|text|Address of Authorization core system|
|**system-name**|text|Name of current system|
|**system-address**|text|Address of current system|
|**system-port**|number|Port of current system|


#### Register
Register Arrowhead entities.

##### Register service
Register a service.

|Parameter|Type|Description|
|---|---|---|
|**interfaces**|list|List of the interfaces the service supports (pattern: <protocol>-SECURE/INSECURE-<format>, e.g.: HTTPS-SECURE-JSON)|
|**name**|text|The definition (and identifier) of service to be registered|
|**security-type**|text|The authentication type for the service to be used|
|**end-of-validity**|text|The service is available until this UTC timestamp|
|**metadata**|map|Various meta information as map|
|**uri**|text|The URI which the service can be accessed on|
|**version**|number|The version of this registry entry|

#### Request
Request data from Arrowhead Systems.

##### Request orchestration
Request orchestration for the system.

|Parameter|Type|Description|
|---|---|---|
|**interfaces**|list|List of the interfaces the requested service should support (pattern: <protocol>-SECURE/INSECURE-<format>, e.g.: HTTPS-SECURE-JSON)|
|**name**|text|The identifier name of the orchestration reponses returned (the response names are numbered from 1 automatically)|
|**service-name**|text|The definition of service requested|
|**flags**|list|List orchestration flags which should be set true for this orchestration|
|**max-version**|number|The maximal version of the requested service registry entry|
|**metadata**|map|Various meta information of the requested service as map|
|**min-version**|number|The minimal version of the requested service registry entry|
|**preferred-provider**|map|Parameters of the preferred provider for the requested service, the keys for this map: 'operator-name', 'cloud-name','system-name','system-address','system-port','authentication-info'|
|**security-types** |list|The authentication types for the requested service should use|
|**version**|number|The version of the requested service registry entry|

##### Request orchestration-id
Request store orchestration by id for the system.

|Parameter|Type|Description|
|---|---|---|
|**name**|text|The identifier name of the orchestration reponses returned (the response names are numbered from 1 automatically)|
|**id**|number|The store id of the orchestration|

##### Request public-key
Get the public key of the Authorization core service.


#### Show
Show various data stored by the application.

##### Show settings
Show the settings.

##### Show services
Show data of services.

|Parameter|Type|Description|
|---|---|---|
|**name**|text|Filter for the previously given name of the service|

##### Show orchestrations
Show data of orchestrations.

|Parameter|Type|Description|
|---|---|---|
|**name**|text|Filter for the previously given name of the orchestration (the orchestrations are numbered automatically)|


#### Unregister
Unregister Arrowhead entities.

##### Unregister service
Use this command to unregister a service.

|Parameter|Type|Description|
|---|---|---|
|**name**|text|Name of the service to be unregistered|
