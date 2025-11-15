//! Network crate - MVP raw TCP transport and basic peer/codec for the project.

pub mod p2p;
pub mod protowire;
pub mod hub;

pub use p2p::Peer;
pub use hub::Hub;
