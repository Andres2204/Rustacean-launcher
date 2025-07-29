use crate::versions::version_json::{VersionJson, VersionType};
use crate::versions::manifest::VersionInfo;
use std::fmt::{Debug, Display, Formatter};
use std::io::{Error, ErrorKind};

/*
    TRAIT
*/
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

/*
    IMPLEMENTATIONS 
*/
#[derive(Debug, Clone)]
pub struct StandardVersion {
    name: String,
    url: String,
    state: VersionState,
    version_type: VersionType,
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

/*
    BUILDER
*/
#[derive(Debug)]
pub struct VersionBuilder {
    name: Option<String>,
    url: Option<String>,
    state: Option<VersionState>,
    version_type: VersionType,
}

const BUILDER_TEMPLATE: VersionBuilder = VersionBuilder {
    name: None,
    url: None,
    state: None,
    version_type: VersionType::RELEASE,
};
impl VersionBuilder {
    pub fn realease() -> Self {
        Self { .. BUILDER_TEMPLATE }
    }
    
    pub fn snapshot() -> Self {
        Self {
            version_type: VersionType::SNAPSHOT,
            .. BUILDER_TEMPLATE
        }
    }
    
    pub fn default() -> Self {
       Self::realease()
   }
}

impl VersionBuilder {
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    
    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }
    
    pub fn state(mut self, state: VersionState) -> Self {
        self.state = Some(state);
        self
    }
    
    
    pub fn build(self) -> Result<Box<dyn Version>, Error> {
        match self.version_type {
            VersionType::RELEASE => {
                if self.name.is_none() { return Err(Error::new(ErrorKind::Other, "No name given")) }
                Ok(Box::new(StandardVersion {
                    name: self.name.unwrap(),
                    url: self.url.unwrap_or(String::new()),
                    state: self.state.unwrap(),
                    version_type: VersionType::RELEASE
                }))
            }
            _ => { Err(Error::new(ErrorKind::Other, "VersionBuilder requires version type.")) }
            //VersionType::SNAPSHOT => {}
            //VersionType::OldBeta => {}
            //VersionType::OldAlpha => {}
        }

    }
}
