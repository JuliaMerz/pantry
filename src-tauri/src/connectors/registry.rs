use std::str::FromStr;

// connectors/registry.rs

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LLMRegistryEntryConnector {
    Ggml,
    OpenAI,
    GenericAPI
}

impl FromStr for LLMRegistryEntryConnector {
    type Err = ();

    fn from_str(input: &str) -> Result<LLMRegistryEntryConnector, Self::Err> {
        match input {
            "ggml"  => Ok(LLMRegistryEntryConnector::Ggml),
            "api" => Ok(LLMRegistryEntryConnector::GenericAPI),
            "openai" => Ok(LLMRegistryEntryConnector::OpenAI),
            _       => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize)]
pub enum LLMRegistryEntrySource {
    GitHub,
    External,
}

impl FromStr for LLMRegistryEntrySource {
    type Err = ();

    fn from_str(input: &str) -> Result<LLMRegistryEntrySource, Self::Err> {
        match input {
            "github"  => Ok(LLMRegistryEntrySource::GitHub),
            "external" => Ok(LLMRegistryEntrySource::External),
            _         => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize)]
pub enum LLMRegistryEntryInstallStep {
    Download,
}

impl FromStr for LLMRegistryEntryInstallStep {
    type Err = ();

    fn from_str(input: &str) -> Result<LLMRegistryEntryInstallStep, Self::Err> {
        match input {
            "download"  => Ok(LLMRegistryEntryInstallStep::Download),
            _         => Err(()),
        }
    }
}

//Important: update interfaces.ts whenever changing this.
#[derive(serde::Deserialize)]
pub struct LLMRegistryEntry {
    id: String,
    name: String,
    description: String,
    licence: String,
    source: LLMRegistryEntrySource, // Github or External
    sequence: Vec<(LLMRegistryEntryInstallStep, String)>,
    entry_type: String,

    create_thread: bool, // Is it an API connector?
    connector: LLMRegistryEntryConnector, // which connector to use
    config: Vec<(String, String)>, //Configs used by the connector
    parameters: Vec<(String, String)>, // Hardcoded Parameters
    user_parameters: Vec<String>, //User Parameters
}




