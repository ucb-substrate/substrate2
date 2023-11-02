//! Substrate integration tests.

pub mod external;

#[cfg(test)]
#[cfg(feature = "lsf")]
pub mod bsub;
#[cfg(test)]
pub mod cache;
#[cfg(test)]
pub mod derive;
#[cfg(test)]
pub mod gds;
#[cfg(test)]
pub mod hard_macro;
#[cfg(test)]
pub mod layout;
#[cfg(test)]
pub mod netlist;
#[cfg(test)]
pub mod paths;
#[cfg(test)]
pub mod pdk;
#[cfg(test)]
pub mod schematic;
#[cfg(test)]
pub mod scir;
#[cfg(test)]
pub mod shared;
#[cfg(test)]
pub mod sim;
