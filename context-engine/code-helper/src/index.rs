use derive_more::{
    Deref,
    DerefMut,
};
use std::{
    collections::HashMap,
    fmt::Debug,
    path::PathBuf,
};

#[derive(Debug)]
pub struct Module {
    pub id: ModuleId,
    pub path: PathBuf,
    pub modules: HashMap<ModuleId, Module>,
}
pub type ModuleId = String;
pub type CrateId = String;

#[derive(Debug, Deref, DerefMut)]
pub struct Crate {
    module: Module,
}

pub fn index_crate(path: PathBuf) {
    println!("{:#?}", path);
}
