pub mod synbiohub;
pub mod igem_parts;
pub mod ncbi;

use crate::types::Biobrick;

pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    fn link(&self, id: &str) -> String;
    fn url(&self, id: &str) -> String;
    fn parse(&self, id: &str, text: &str) -> Option<Biobrick>;
}

pub fn get_all_providers() -> Vec<Box<dyn Provider>> {
    vec![
        Box::new(igem_parts::IgemPartsProvider),
        Box::new(synbiohub::SynBioHubProvider),
        Box::new(ncbi::NcbiProvider),
    ]
}
