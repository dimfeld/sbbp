//! Object storage configuration

#![allow(unused_imports)]

use error_stack::{Report, ResultExt};
use filigree::{
    config::parse_option,
    storage::{Storage, StorageConfig, StorageError, StoragePreset},
};
use url::Url;

pub struct AppStorage {
    pub images: Storage,
    pub uploads: Storage,
    pub config_disk: StorageConfig,
}

impl AppStorage {
    pub fn new(config: AppStorageConfig) -> Result<AppStorage, Report<StorageError>> {
        Ok(AppStorage {
            images: Storage::new(&config.images.config, config.images.bucket)
                .attach_printable("Unable to create storage for images")?
                .with_public_url(config.images.public_url),
            uploads: Storage::new(&config.uploads.config, config.uploads.bucket)
                .attach_printable("Unable to create storage for uploads")?
                .with_public_url(config.uploads.public_url),
            config_disk: config.config_disk,
        })
    }
}

pub struct AppStorageConfigEntry {
    pub config: StorageConfig,
    pub bucket: String,
    pub public_url: Option<Url>,
}

pub struct AppStorageConfig {
    pub images: AppStorageConfigEntry,
    pub uploads: AppStorageConfigEntry,
    pub config_disk: StorageConfig,
}

impl AppStorageConfig {
    /// Create the application storage configuration based on the filigree configuration files
    /// and environment variables.
    pub fn new() -> Result<AppStorageConfig, StorageError> {
        let config_disk = StorageConfig::from_env(
            StorageConfig::Local(filigree::storage::local::LocalStoreConfig {
                base_path: Some(r##"storage"##.to_string()),
            }),
            "STORAGE_PROVIDER_DISK_",
        )?;

        let mut bucket_config_images = config_disk.clone();
        bucket_config_images.merge_env("STORAGE_IMAGES_")?;

        let images_bucket =
            std::env::var("STORAGE_IMAGES_BUCKET").unwrap_or_else(|_| "sbbp-images".to_string());

        let images_public_url: Option<Url> =
            parse_option(std::env::var("STORAGE_IMAGES_PUBLIC_URL").ok()).map_err(|_| {
                StorageError::Configuration("Invalid URL in STORAGE_IMAGES_PUBLIC_URL")
            })?;

        let mut bucket_config_uploads = config_disk.clone();
        bucket_config_uploads.merge_env("STORAGE_UPLOADS_")?;

        let uploads_bucket =
            std::env::var("STORAGE_UPLOADS_BUCKET").unwrap_or_else(|_| "sbbp-uploads".to_string());

        let uploads_public_url: Option<Url> =
            parse_option(std::env::var("STORAGE_UPLOADS_PUBLIC_URL").ok()).map_err(|_| {
                StorageError::Configuration("Invalid URL in STORAGE_UPLOADS_PUBLIC_URL")
            })?;

        Ok(AppStorageConfig {
            images: AppStorageConfigEntry {
                config: bucket_config_images,
                bucket: images_bucket,
                public_url: images_public_url,
            },
            uploads: AppStorageConfigEntry {
                config: bucket_config_uploads,
                bucket: uploads_bucket,
                public_url: uploads_public_url,
            },
            config_disk,
        })
    }

    /// A test configuration that forces all storage providers to be in-memory.
    pub fn new_in_memory() -> AppStorageConfig {
        AppStorageConfig {
            images: AppStorageConfigEntry {
                config: StorageConfig::Memory,
                bucket: "sbbp-images".to_string(),
                public_url: None,
            },
            uploads: AppStorageConfigEntry {
                config: StorageConfig::Memory,
                bucket: "sbbp-uploads".to_string(),
                public_url: None,
            },
            config_disk: StorageConfig::Memory,
        }
    }
}
