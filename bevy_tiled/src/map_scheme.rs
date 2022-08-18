//! Module which houses a macro, that allows to quickly "match" against
//! a map from `tiled` crate, enabling the user to cleanly enforce some
//! structure.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LevelMatchError {
    #[error("The layer wasn't found")]
    NotFound,
    #[error("Found the layer of wrong type. Expected {expected:}, found {found:}")]
    TypeMismatched {
        expected: &'static str,
        found: &'static str,
    },
    #[error("An error occured while parsing layer {sublayer_name:}.")]
    SublayerError {
        sublayer_name: &'static str,
        source: Box<LevelMatchError>,
    },
}

#[derive(Debug, Error)]
#[error("Failed to parse layer {layer_name:}")]
pub struct MapMatchError {
    layer_name: &'static str,
    source: LevelMatchError,
}

