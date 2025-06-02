// Graph Management Context - Core Domain
// Responsible for graph structure and topology

pub mod domain;
pub mod events;
pub mod plugin;
pub mod repositories;
pub mod services;
pub mod storage;

#[cfg(debug_assertions)]
pub mod verify_storage;

#[cfg(test)]
mod tests;
