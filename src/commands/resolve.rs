use faccess::PathExt;
use std::path::PathBuf;

use crate::commands::error::CommandsError;

pub(crate) fn find_exec_file(cmd: &str) -> Result<Option<PathBuf>, CommandsError> {
    let Some(env_path) = std::env::var_os("PATH") else {
        return Err(CommandsError::PathNotSet)?;
    };
    for mut path in std::env::split_paths(&env_path) {
        if let Ok(exists) = path.try_exists() {
            if !exists {
                continue;
            }
            path.push(cmd);
            if path.executable() {
                return Ok(Some(path));
            }
        }
    }
    Ok(None)
}
