//! Bootstrap - Base directory resolution and environment loading.

use std::path::PathBuf;

/// Get the base directory for bkg-p2p data.
///
/// Priority:
/// 1. `BKG_P2P_HOME` environment variable
/// 2. `~/.bkg-p2p` on Unix systems
/// 3. `%APPDATA%\bkg-p2p` on Windows
pub fn base_dir() -> PathBuf {
    if let Ok(home) = std::env::var("BKG_P2P_HOME") {
        return PathBuf::from(home);
    }

    dirs::home_dir()
        .map(|h| h.join(".bkg-p2p"))
        .unwrap_or_else(|| PathBuf::from(".bkg-p2p"))
}

/// Get the directory for WASM tools.
pub fn tools_dir() -> PathBuf {
    base_dir().join("tools")
}

/// Get the directory for agent specs.
pub fn agents_dir() -> PathBuf {
    base_dir().join("agents")
}

/// Get the directory for data (database, etc.).
pub fn data_dir() -> PathBuf {
    base_dir().join("data")
}

/// Get the directory for models (LLM weights).
pub fn models_dir() -> PathBuf {
    base_dir().join("models")
}

/// Get the path to the identity key file.
pub fn identity_path() -> PathBuf {
    base_dir().join("identity.key")
}

/// Get the path to the database file.
pub fn database_path() -> PathBuf {
    data_dir().join("bkg-p2p.redb")
}

/// Get the path to the config file.
pub fn config_path() -> PathBuf {
    base_dir().join("config.toml")
}

/// Load environment variables from `.bkg-p2p/.env` if present.
pub fn load_env() {
    let env_path = base_dir().join(".env");
    if env_path.exists() {
        if let Err(e) = dotenvy::from_path(&env_path) {
            tracing::warn!("Failed to load .env from {:?}: {}", env_path, e);
        }
    }
}

/// Ensure all required directories exist.
pub fn ensure_dirs() -> std::io::Result<()> {
    std::fs::create_dir_all(base_dir())?;
    std::fs::create_dir_all(tools_dir())?;
    std::fs::create_dir_all(agents_dir())?;
    std::fs::create_dir_all(data_dir())?;
    std::fs::create_dir_all(models_dir())?;
    std::fs::create_dir_all(base_dir().join("skills"))?;
    Ok(())
}

/// Migrate from old `~/.bkg-peer` directory to `~/.bkg-p2p`.
///
/// This function is idempotent - safe to call multiple times.
/// It does NOT delete the old directory; it only copies data forward.
pub fn migrate_from_legacy() -> std::io::Result<()> {
    let legacy_dir = dirs::home_dir()
        .map(|h| h.join(".bkg-peer"))
        .unwrap_or_else(|| PathBuf::from(".bkg-peer"));

    let new_dir = base_dir();

    // Skip if new directory already has content
    if new_dir.exists() && new_dir.join("config.toml").exists() {
        return Ok(());
    }

    // Skip if legacy directory doesn't exist
    if !legacy_dir.exists() {
        return Ok(());
    }

    tracing::info!(
        "Migrating data from {} to {}",
        legacy_dir.display(),
        new_dir.display()
    );

    std::fs::create_dir_all(&new_dir)?;

    // Copy directory contents recursively
    for entry in std::fs::read_dir(&legacy_dir)? {
        let entry = entry?;
        let src = entry.path();
        let dst = new_dir.join(entry.file_name());

        if src.is_dir() {
            std::fs::create_dir_all(&dst)?;
            copy_dir_recursive(&src, &dst)?;
        } else {
            std::fs::copy(&src, &dst)?;
        }
    }

    tracing::info!(
        "Migration complete. Old data preserved at {}",
        legacy_dir.display()
    );
    Ok(())
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path)?;
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_dir_exists() {
        let dir = base_dir();
        assert!(dir.ends_with(".bkg-p2p"));
    }

    #[test]
    fn test_subdirs() {
        assert!(tools_dir().ends_with("tools"));
        assert!(agents_dir().ends_with("agents"));
        assert!(data_dir().ends_with("data"));
        assert!(models_dir().ends_with("models"));
    }
}
