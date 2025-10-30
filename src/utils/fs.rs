use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub(crate) fn symlink_executable(
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
) -> Result<(), std::io::Error> {
    // Get current permissions
    let metadata = std::fs::metadata(&original)?;
    let mut permissions = metadata.permissions();

    // Set the user, group, and other executable bits (0o111)
    let mode = permissions.mode() | 0o111;
    permissions.set_mode(mode);

    // Ensure the Yarn binary is executable
    std::fs::set_permissions(&original, permissions)?;

    std::os::unix::fs::symlink(original, link)
}
