use flate2::read::GzDecoder;
use std::fs;
use std::io;
use std::path::Path;
use tar::Archive;

pub fn download(uri: impl AsRef<str>, destination: impl AsRef<Path>) -> Result<(), DownloadError> {
    let mut response_reader = ureq::get(uri.as_ref())
        .call()
        .map_err(|err| DownloadError::RequestError(Box::new(err)))?
        .into_reader();

    let mut destination_file = fs::File::create(destination.as_ref())
        .map_err(DownloadError::CouldNotCreateDestinationFile)?;

    io::copy(&mut response_reader, &mut destination_file)
        .map_err(DownloadError::CouldNotWriteDestinationFile)?;

    Ok(())
}

#[derive(Debug)]
pub enum DownloadError {
    // Boxed to prevent `large_enum_variant` errors since `ureq::Error` is massive.
    RequestError(Box<ureq::Error>),
    CouldNotCreateDestinationFile(std::io::Error),
    CouldNotWriteDestinationFile(std::io::Error),
}

pub fn untar(path: impl AsRef<Path>, destination: impl AsRef<Path>) -> Result<(), UntarError> {
    let file = fs::File::open(path.as_ref()).map_err(UntarError::CouldNotOpenFile)?;

    Archive::new(GzDecoder::new(file))
        .unpack(destination.as_ref())
        .map_err(UntarError::CouldNotUnpack)
}

#[derive(Debug)]
pub enum UntarError {
    CouldNotOpenFile(std::io::Error),
    CouldNotUnpack(std::io::Error),
}
