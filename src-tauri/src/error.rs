use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Sound not found: {0}")]
    SoundNotFound(String),
    #[error("Cannot remove bundled sounds")]
    BundledSoundRemoval,
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
    #[error("Invalid filename")]
    InvalidFilename,
    #[error("Audio engine error: {0}")]
    Audio(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Home directory not found")]
    HomeDirNotFound,
}

// Tauri v2 requires command errors to be serializable.
// We serialize to the display string for frontend compatibility.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        let cases: Vec<(AppError, &str)> = vec![
            (
                AppError::SoundNotFound("rain".into()),
                "Sound not found: rain",
            ),
            (
                AppError::BundledSoundRemoval,
                "Cannot remove bundled sounds",
            ),
            (
                AppError::UnsupportedFileType(".ogg".into()),
                "Unsupported file type: .ogg",
            ),
            (AppError::InvalidFilename, "Invalid filename"),
            (
                AppError::Audio("playback failed".into()),
                "Audio engine error: playback failed",
            ),
            (AppError::HomeDirNotFound, "Home directory not found"),
        ];

        for (error, expected) in &cases {
            assert_eq!(
                format!("{}", error),
                *expected,
                "Display mismatch for {:?}",
                error
            );
        }

        // IO variant — wraps a real io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let app_err = AppError::Io(io_err);
        assert!(
            format!("{}", app_err).starts_with("IO error:"),
            "Io variant display should start with 'IO error:'"
        );

        // Serialization variant — wraps a real serde_json error
        let serde_err: Result<serde_json::Value, _> = serde_json::from_str("{invalid");
        let app_err = AppError::Serialization(serde_err.unwrap_err());
        assert!(
            format!("{}", app_err).starts_with("Serialization error:"),
            "Serialization variant display should start with 'Serialization error:'"
        );
    }

    #[test]
    fn error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "no access");
        let app_err: AppError = io_err.into();

        assert!(
            matches!(app_err, AppError::Io(_)),
            "Expected Io variant, got {:?}",
            app_err
        );
        assert!(format!("{}", app_err).contains("no access"));
    }

    #[test]
    fn error_serializes_to_string() {
        let error = AppError::SoundNotFound("thunder".into());
        let json = serde_json::to_value(&error).expect("serialize AppError");

        // The custom Serialize impl writes the Display string
        assert_eq!(
            json,
            serde_json::Value::String("Sound not found: thunder".into())
        );
    }
}
