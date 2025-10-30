use crate::utils;
use crate::utils::async_runtime::ASYNC_RUNTIME;
use async_compression::tokio::bufread::GzipDecoder;
use bullet_stream::global::print;
use futures::StreamExt;
use futures::stream::TryStreamExt;
use sha2::{Digest, Sha256};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs::File as AsyncFile;
use tokio::io::{AsyncWriteExt, BufReader as AsyncBufReader, copy as async_copy, sink};
use tokio_util::io::{InspectReader, StreamReader};

pub(crate) fn download_sync<'a, T>(downloader: T) -> Result<(), DownloaderError>
where
    T: Downloader<'a>,
{
    ASYNC_RUNTIME.block_on(async { download(downloader).await })
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn download<'a, T>(downloader: T) -> Result<(), DownloaderError>
where
    T: Downloader<'a>,
{
    let response = utils::http::get(downloader.source_url())
        .connect_timeout(downloader.connect_timeout())
        .read_timeout(downloader.read_timeout())
        .max_retries(downloader.max_retries())
        .call()
        .await
        .map_err(|e| DownloaderError::Request {
            url: downloader.source_url().to_string(),
            source: e,
        })?;

    let timer = print::sub_start_timer("Downloading");

    let checksum_type = downloader.checksum_validator();

    #[allow(clippy::manual_map)]
    let mut hasher = match checksum_type {
        Some(ChecksumValidator::Sha256(_)) => Some(Sha256::new()),
        None => None,
    };

    // ensure the writer is dropped in this scope so the InspectReader is dropped and is no longer borrowing the hash digest
    {
        let mut inspectable_reader = AsyncBufReader::new(
            // the inspect reader lets us pipe this decompressed output to both the output file and the hash digest
            InspectReader::new(
                StreamReader::new(response.bytes_stream().map_err(io::Error::other)),
                |bytes| {
                    if let Some(hasher) = &mut hasher {
                        hasher.update(bytes);
                    }
                },
            ),
        );

        let create_write_error = |e| DownloaderError::Write {
            url: downloader.source_url().to_string(),
            destination: downloader.destination().to_path_buf(),
            source: e,
        };

        match downloader.extractor() {
            Some(Extractor::Gzip(options)) => {
                let mut reader = GzipDecoder::new(inspectable_reader);
                // Enable/Disable support for multistream gz files. In this mode, the reader expects the input to
                // be a sequence of individually gzipped data streams, each with its own header and trailer,
                // ending at EOF. This is standard behavior for gzip readers.
                reader.multiple_members(options.multiple_members);
                let mut tar_archive = tokio_tar::Archive::new(reader);
                let mut entries = tar_archive.entries().map_err(create_write_error)?;
                while let Some(entry) = entries.next().await {
                    let mut entry = entry.map_err(create_write_error)?;
                    let path = entry.path().map_err(create_write_error)?;

                    // Get the path components
                    let path_components: Vec<_> = path.components().collect();

                    // Skip if we don't have enough components after stripping
                    if path_components.len() <= options.strip_components {
                        async_copy(&mut entry, &mut sink())
                            .await
                            .map_err(create_write_error)?;
                        continue;
                    }

                    // Skip if the path contains '..' or is absolute
                    if path_components.iter().any(|c| {
                        matches!(
                            c,
                            std::path::Component::ParentDir | std::path::Component::RootDir
                        )
                    }) {
                        async_copy(&mut entry, &mut sink())
                            .await
                            .map_err(create_write_error)?;
                        continue;
                    }

                    // Build the stripped path
                    let stripped_path: PathBuf = path_components
                        .into_iter()
                        .skip(options.strip_components)
                        .collect();

                    // Skip empty paths
                    if stripped_path.components().count() == 0 {
                        async_copy(&mut entry, &mut sink())
                            .await
                            .map_err(create_write_error)?;
                        continue;
                    }

                    // Skip excluded paths
                    let exclude = &options.exclude;
                    if exclude(&stripped_path) {
                        async_copy(&mut entry, &mut sink())
                            .await
                            .map_err(create_write_error)?;
                        continue;
                    }

                    let entry_type = entry.header().entry_type();
                    let dest_path = downloader.destination().join(&stripped_path);

                    // Create parent directories for regular files and symlinks
                    if (entry_type.is_file()
                        || entry_type.is_symlink()
                        || entry_type.is_hard_link())
                        && let Some(parent) = dest_path.parent()
                    {
                        tokio::fs::create_dir_all(parent)
                            .await
                            .map_err(create_write_error)?;
                    }

                    entry.unpack(dest_path).await.map_err(create_write_error)?;
                }
            }
            None => {
                let mut writer = AsyncFile::create(downloader.destination())
                    .await
                    .map_err(create_write_error)?;
                async_copy(&mut inspectable_reader, &mut writer)
                    .await
                    .map_err(create_write_error)?;
                writer.flush().await.map_err(create_write_error)?;
            }
        }
    }

    timer.done();

    if let Some(hasher) = hasher.take() {
        match checksum_type {
            Some(ChecksumValidator::Sha256(expected_value)) => {
                let actual_checksum = hasher.finalize();
                if expected_value != actual_checksum.to_vec() {
                    Err(DownloaderError::ChecksumMismatch {
                        url: downloader.source_url().to_string(),
                        expected_checksum: hex::encode(expected_value),
                        actual_checksum: format!("{actual_checksum:x}"),
                    })?;
                }
            }
            None => {}
        }
    }

    Ok(())
}

pub(crate) trait Downloader<'a> {
    fn source_url(&self) -> &str;

    fn destination(&self) -> &Path;

    fn checksum_validator(&self) -> Option<ChecksumValidator<'a>> {
        None
    }

    fn extractor(&self) -> Option<Extractor> {
        None
    }

    fn connect_timeout(&self) -> Duration {
        crate::utils::http::DEFAULT_CONNECT_TIMEOUT
    }

    fn read_timeout(&self) -> Duration {
        crate::utils::http::DEFAULT_READ_TIMEOUT
    }

    fn max_retries(&self) -> u32 {
        crate::utils::http::DEFAULT_RETRIES
    }
}

#[derive(Debug)]
pub(crate) enum Extractor {
    Gzip(GzipOptions),
}

pub(crate) struct GzipOptions {
    pub(crate) multiple_members: bool,
    pub(crate) strip_components: usize,
    pub(crate) exclude: Box<dyn Fn(&Path) -> bool>,
}

impl std::fmt::Debug for GzipOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GzipOptions")
            .field("multiple_members", &self.multiple_members)
            .field("strip_components", &self.strip_components)
            .field("exclude", &"<closure>")
            .finish()
    }
}

impl Default for GzipOptions {
    fn default() -> Self {
        Self {
            multiple_members: true,
            strip_components: 0,
            exclude: Box::new(|_| false),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ChecksumValidator<'a> {
    Sha256(&'a [u8]),
}

#[derive(Debug)]
pub(crate) enum DownloaderError {
    Request {
        url: String,
        source: utils::http::Error,
    },
    Write {
        url: String,
        destination: PathBuf,
        source: io::Error,
    },
    ChecksumMismatch {
        url: String,
        expected_checksum: String,
        actual_checksum: String,
    },
}
