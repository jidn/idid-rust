use chrono::{DateTime, FixedOffset};
use std::env;
use std::fs;
use std::io::{Error, ErrorKind, Write};
use std::path::PathBuf;

/// Get the path to a TSV file.
///
/// Confirm the path exists and is a file.
///
/// # Arguments
/// * `tsv` - An optional `std::path::PathBuf` representing a reference date.
///     The are three possible sources of this path in increasing order of
///     likelyhood and preference.  
///     1. Given std::path::PathBuf.
///     2. The environment variable $ididTSV has the path.
///     3. The environment variable $XDG_DATA_HOME/idid/idid.tsv  
///        If the file doesn't exist, it will attempt to create it.
pub fn get_tsv_path(tsv: &Option<std::path::PathBuf>) -> Result<PathBuf, Error> {
    match tsv {
        Some(path) => is_existing_file(&path, "--tsv "),
        _ => {
            let idid_tsv = "ididTSV";
            // Existing "ididTSV" environment variable contains absolute path
            if let Ok(value) = env::var(idid_tsv) {
                let prefix = format!("${} ", idid_tsv);
                return is_existing_file(&PathBuf::from(value), &prefix);
            }
            let env_xdg = "XDG_DATA_HOME";
            match env::var(env_xdg) {
                Err(e) => Err(Error::new(
                    ErrorKind::NotFound,
                    format!("${} does not exist: {}", env_xdg, e),
                )),
                Ok(value) => {
                    let prefix = format!("${} ", env_xdg);
                    let mut path = PathBuf::from(value);
                    path.push("idid");
                    path.push("idid.tsv");

                    // Check for file $XDG_DATA_HOME/idid/idid.tsv
                    match is_existing_file(&path, &prefix) {
                        Ok(path) => Ok(path),
                        Err(_e) => {
                            // Create the file if possible
                            match fs::File::create(&path) {
                                Ok(_fc) => Ok(path),
                                Err(e) => Err(e),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Check the path exists and it is a file.
fn is_existing_file(path: &std::path::PathBuf, prefix: &str) -> Result<PathBuf, Error> {
    let file_path = path.to_string_lossy().to_owned();
    if path.exists() {
        if path.is_file() {
            // Return the path if it exists and is a file
            Ok(path.clone())
        } else {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!("{}not a file: {}", prefix, file_path),
            ))
        }
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("{}does not exist: {}", prefix, file_path),
        ))
    }
}

pub fn write_to_tsv(path: &str, timestamp: &DateTime<FixedOffset>, text_to_append: &str) {
    // Open the file in append mode or create it if it doesn't exist
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .expect("Failed to open file");

    file.write_all(
        timestamp
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
            .as_bytes(),
    )
    .expect("Failed to write timestamp to file");
    file.write_all(b"\t").expect("Failed to write tab");
    file.write_all(text_to_append.as_bytes())
        .expect("Failed to write to file");
    file.write_all(b"\n").expect("Failed to write line-feed");
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::get_tsv_path;
    use std::env;
    use std::fs;
    use std::io::Error;
    use std::path::PathBuf;
    use tempfile::Builder;

    // These tests mess with environmen variables so should run syncro
    // cargo test -- --ignored --test-threads=1
    // cargo test -- --include-ignored --test-threads=1
    #[test]
    #[ignore]
    fn test_get_tsv_path_ididtsv() -> Result<(), Error> {
        let env_vars = ["ididTSV", "XDG_DATA_HOME"];
        // Save the current values of the environment variables
        let saved_values: Vec<Option<String>> = env_vars
            .iter()
            .map(|&var| save_environment_variable(var))
            .collect();

        // Create a tmp file
        let temp_file = Builder::new().suffix(".txt").tempfile()?;
        let file_path = temp_file.path().to_owned();

        // Set the environment variable for testing purposes
        env::set_var(env_vars[0], &file_path);
        env::remove_var(env_vars[1]);

        let result = get_tsv_path(None::<PathBuf>);

        // Restore the original values of the environment variables or delete
        // them if they didn't exist before
        for (var, saved_value) in env_vars.iter().zip(saved_values) {
            restore_environment_variable(var, saved_value);
        }

        // Remove the file
        drop(temp_file);
        let _ = std::fs::remove_file(&file_path);

        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_get_tsv_path_xdg() -> Result<(), Error> {
        let env_vars = ["ididTSV", "XDG_DATA_HOME"];
        // Save the current values of the environment variables
        let saved_values: Vec<Option<String>> = env_vars
            .iter()
            .map(|&var| save_environment_variable(var))
            .collect();

        // Get the temporary directory path
        let temp_dir = env::temp_dir();

        // Create the "idid" directory inside the temporary directory
        let idid_dir = temp_dir.join("idid");
        fs::create_dir_all(&idid_dir)?;

        // Create an empty file "idid.txt" inside the "idid" directory
        let file_path = idid_dir.join("idid.tsv");
        let temp_file = fs::File::create(&file_path)?;
        drop(temp_file);

        // Set the environment variable for testing purposes
        env::set_var(env_vars[1], &temp_dir);
        env::remove_var(env_vars[0]);

        let result = get_tsv_path(None::<PathBuf>);

        // Restore the original values of the environment variables or delete
        // them if they didn't exist before
        for (var, saved_value) in env_vars.iter().zip(saved_values) {
            restore_environment_variable(var, saved_value);
        }

        // Remove the file
        std::fs::remove_file(&file_path)?;

        assert!(result.is_ok());
        Ok(())
    }

    // Function to save the current value of an environment variable
    fn save_environment_variable(name: &str) -> Option<String> {
        match env::var(name) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }
    // Function to restore the original value of an environment variable
    // or delete if it didn't exist before
    fn restore_environment_variable(name: &str, saved_value: Option<String>) {
        match saved_value {
            Some(value) => env::set_var(name, &value),
            None => env::remove_var(name),
        }
    }
}
