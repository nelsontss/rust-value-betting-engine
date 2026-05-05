pub trait DomainService {}

mod cluster_service;
mod fixture_cluster;

pub use fixture_cluster::FixtureCluster;
