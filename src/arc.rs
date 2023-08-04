#[cfg(not(feature = "precompute"))]
mod cached;
#[cfg(not(feature = "precompute"))]
pub use cached::*;

#[cfg(feature = "precompute")]
mod precomputed;
#[cfg(feature = "precompute")]
pub use precomputed::*;

mod uarc;
pub use uarc::*;
