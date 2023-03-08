use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type LogResult<T> = Result<T, String>;

pub trait LogOperations {
    fn ls(&self, path: &str) -> LogResult<Vec<FileIndex>>;
    fn download(&self, path: &str, to: &str) -> LogResult<()>;
    fn new(configs: Configs) -> Self;
}

pub trait LogGetter {
    fn list_folders(&self, device: &String) -> LogResult<Vec<String>>;
    fn download_folder(&self, device: &String, folder: &String) -> LogResult<String>;
    fn new(configs: Configs) -> Self;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigsCollection {
    pub dev: Configs,
    pub stage: Configs,
    pub prod: Configs,
}

impl ConfigsCollection {
    pub fn get_config(&self, env : &Environment) -> &Configs {
        match *env {
            Environment::Prod => &self.prod,
            Environment::Stage => &self.stage,
            Environment::Dev => &self.dev,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configs {
    pub base_path: String,
    pub profile: String,
    pub output_path: String,
}

#[derive(Clone, Debug)]
pub struct File {
    pub name: String,
    pub size: u64,
    pub date: DateTime<Utc>,
}
#[derive(Clone, Debug)]
pub struct Prefix {
    pub name: String,
}
#[derive(Clone, Debug)]
pub enum FileIndex {
    File(File),
    Prefix(Prefix),
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Environment {
    Prod,
    Stage,
    Dev,
}

impl std::str::FromStr for Environment {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "production" | "prod" | "p" => Ok(Environment::Prod),
            "stage" | "s" => Ok(Environment::Stage),
            "development" | "dev" | "d" => Ok(Environment::Dev),
            _ => Err(Self::Err::new(
                std::io::ErrorKind::NotFound,
                "Unknown environment!",
            )),
        }
    }
}