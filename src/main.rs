//! zarrdump ncdump but for zarr
//!
//! TODO Long Description
//!
//! # Examples
//!
//! TODO Example
use std::sync::Arc;

use clap::Parser;
use zarrs::group::{Group, GroupCreateError};
use zarrs::storage::{ReadableListableStorage, ReadableListableStorageTraits};
use zarrs_filesystem::FilesystemStore;

/// Errors that can be raised by this program
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to serialize value into JSON or from JSON: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("ChildIteratorError: Failed to iterate over child nodes in zarr dataset, perhaps their missing?: {0}")]
    ChildIteratorError(#[from] zarrs::storage::StorageError),
}

/// Dump a ZARR v2 or v3 dataset metadata as JSON
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File path to the zarr dataset
    path: String,
}

/// Mapper from zarr storage to JSON metadata about the root group of that store.
struct GroupMetadataMapper<S: ?Sized + ReadableListableStorageTraits> {
    group: Group<S>,
}

impl<S: ?Sized + ReadableListableStorageTraits> GroupMetadataMapper<S> {
    /// Create a new [`GroupMetadataMapper`] from a store. Note, this method
    /// takes ownership of the Arc, so the client should probably call `clone`
    /// if they want to keep a reference.
    pub fn try_from_store(storage: Arc<S>) -> Result<Self, GroupCreateError> {
        Ok(Self {
            group: Group::open(storage, "/")?,
        })
    }

    /// Return a JSON value of the group metadata with each child serialized as
    /// a variable.
    pub fn to_value(&self) -> Result<serde_json::Value, Error> {
        let mut root = serde_json::to_value(self.group.metadata())
            .expect("Failed to serialize group metadata into JSON");
        if !matches!(root, serde_json::Value::Object(..)) {
            panic!("Group metadata is not a valid mapping");
        }
        if let serde_json::Value::Object(root_map) = &mut root {
            let mut new_mapping = serde_json::Map::new();
            self.group.children(true)?.iter().for_each(|node| {
                let name = String::from(node.name().as_str());
                new_mapping.insert(name, serde_json::to_value(node.metadata()).unwrap());
            });
            root_map.insert(
                "variables".to_string(),
                serde_json::Value::Object(new_mapping),
            );
        }
        Ok(root)
    }
}

/// Main entry point
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments
    let args = Args::parse();

    let store: ReadableListableStorage = Arc::new(FilesystemStore::new(&args.path)?);
    let mapper = GroupMetadataMapper::try_from_store(store.clone())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&mapper.to_value()?)
            .expect("Failed to serialize JSON back into JSON...")
    );

    Ok(())
}
