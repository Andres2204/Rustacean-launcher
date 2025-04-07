use crate::core::downloader::download_structs::{VersionJson, VersionType};
use crate::core::versions::manifest::VersionInfo;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use crate::core::versions::verifier::VersionVerifier;
// TODO: trait -> normalversion, modpackversion, forgeversion (mod client)

pub trait Version: Send + Sync {
    fn name(&self) -> String;
    fn set_name(&mut self, name: String);
    fn state(&self) -> VersionState;
    fn set_state(&mut self, state: VersionState);
    fn version_type(&self) -> VersionType;
    fn set_version_type(&mut self, version_type: VersionType);
    fn json_url(&self) -> String;
    fn box_clone(&self) -> Box<dyn Version>;
    fn from_local(version_json: VersionJson) -> Box<dyn Version> where Self: Sized;
}
impl Display for dyn Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, Type: {:?}",
            self.name(),
            self.version_type(),
        )
    }
}
impl Debug for dyn Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, State: {:?}, Type: {:?}, URL: {}",
            self.name(),
            self.state(),
            self.version_type(),
            self.json_url()
        )
    }
}
impl Clone for Box<dyn Version> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
    
}

#[derive(Debug, Copy, Clone)]
pub enum VersionState {
    INSTALLED(bool),
    DOWNLOADING,
    VERIFYING,
}

#[derive(Debug, Clone)]
pub struct StandardVersion {
    name: String,
    version_type: VersionType,
    url: String,
    state: VersionState,
}
impl StandardVersion {
    pub fn new(name: &str, version_type: VersionType, url: &str, state: VersionState) -> Self {
        Self {
            name: name.to_string(),
            version_type,
            url: url.to_string(),
            state,
        }
    }
}
impl From<&VersionInfo> for StandardVersion {
    fn from(value: &VersionInfo) -> Self {
        Self {
            // TODO: avoid clone
            name: value.id.clone(),
            version_type: value.version_type.clone(),
            url: value.url.clone(),
            state: VersionState::INSTALLED(false),
        }
    }
}
impl Version for StandardVersion {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn set_name(&mut self, name: String) {
        self.name = name
    }
    fn state(&self) -> VersionState {
        self.state.clone()
    }
    fn set_state(&mut self, state: VersionState) {
        self.state = state;
    }
    fn version_type(&self) -> VersionType {
        self.version_type.clone()
    }
    fn set_version_type(&mut self, version_type: VersionType) {
        self.version_type = version_type;
    }
    fn json_url(&self) -> String {
        self.url.clone()
    }
    fn box_clone(&self) -> Box<dyn Version> {
        Box::new(self.clone())
    }
    fn from_local(version_json: VersionJson) -> Box<dyn Version> {
        let version: Box<dyn Version> = Box::new(Self {
            name: version_json.id(),
            version_type: version_json.get_type(), 
            url: "".to_string(), // TODO: find the url with the json or save it in a file
            state: VersionState::INSTALLED(true),
        }); // TODO: Verify instalation
        version
    }
}

