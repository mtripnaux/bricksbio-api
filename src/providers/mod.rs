use async_trait::async_trait;
use crate::types::Biobrick;

#[async_trait]
pub trait ProviderEnumTrait: Send + Sync {
    fn name(&self) -> &'static str;
    fn link(&self, id: &str) -> String;
    fn url(&self, id: &str) -> String;
    async fn parse(&self, id: &str, text: &str) -> Option<Biobrick>;
}
pub mod synbiohub;
pub mod igem_parts;
pub mod ncbi;
pub mod igem_registry;
pub mod ensembl;
pub mod addgene;

use crate::providers::synbiohub::SynBioHubProvider;
use crate::providers::igem_parts::IgemPartsProvider;
use crate::providers::ncbi::NcbiProvider;
use crate::providers::igem_registry::IgemApiProvider;
use crate::providers::ensembl::EnsemblProvider;
use crate::providers::addgene::AddGeneProvider;

pub enum ProviderEnum {
    SynBioHub(SynBioHubProvider),
    IgemParts(IgemPartsProvider),
    Ncbi(NcbiProvider),
    IgemApi(IgemApiProvider),
    Ensembl(EnsemblProvider),
    AddGene(AddGeneProvider),
}

impl ProviderEnum {
    pub fn name(&self) -> &'static str {
        match self {
            ProviderEnum::SynBioHub(p) => p.name(),
            ProviderEnum::IgemParts(p) => p.name(),
            ProviderEnum::Ncbi(p) => p.name(),
            ProviderEnum::IgemApi(p) => p.name(),
            ProviderEnum::Ensembl(p) => p.name(),
            ProviderEnum::AddGene(p) => p.name(),
        }
    }
    pub fn link(&self, id: &str) -> String {
        match self {
            ProviderEnum::SynBioHub(p) => p.link(id),
            ProviderEnum::IgemParts(p) => p.link(id),
            ProviderEnum::Ncbi(p) => p.link(id),
            ProviderEnum::IgemApi(p) => p.link(id),
            ProviderEnum::Ensembl(p) => p.link(id),
            ProviderEnum::AddGene(p) => p.link(id),
        }
    }
    pub fn url(&self, id: &str) -> String {
        match self {
            ProviderEnum::SynBioHub(p) => p.url(id),
            ProviderEnum::IgemParts(p) => p.url(id),
            ProviderEnum::Ncbi(p) => p.url(id),
            ProviderEnum::IgemApi(p) => p.url(id),
            ProviderEnum::Ensembl(p) => p.url(id),
            ProviderEnum::AddGene(p) => p.url(id),
        }
    }
    pub async fn parse(&self, id: &str, text: &str) -> Option<Biobrick> {
        use crate::providers::ProviderEnumTrait;
        match self {
            ProviderEnum::SynBioHub(p) => p.parse(id, text).await,
            ProviderEnum::IgemParts(p) => p.parse(id, text).await,
            ProviderEnum::Ncbi(p) => p.parse(id, text).await,
            ProviderEnum::IgemApi(p) => p.parse(id, text).await,
            ProviderEnum::Ensembl(p) => p.parse(id, text).await,
            ProviderEnum::AddGene(p) => p.parse(id, text).await,
        }
    }
}

pub fn get_all_providers() -> Vec<ProviderEnum> {
    vec![
        ProviderEnum::IgemApi(IgemApiProvider),
        ProviderEnum::SynBioHub(SynBioHubProvider),
        ProviderEnum::IgemParts(IgemPartsProvider),
        ProviderEnum::Ncbi(NcbiProvider),
        ProviderEnum::Ensembl(EnsemblProvider),
        ProviderEnum::AddGene(AddGeneProvider),
    ]
}
