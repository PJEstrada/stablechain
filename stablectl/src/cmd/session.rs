use std::fs;
use std::path::PathBuf;

const SESSION_DIR: &str = ".stablectl";
const SESSION_FILE: &str = "session.jwt";

fn home_dir() -> anyhow::Result<PathBuf> {
    let home =
        std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
    Ok(PathBuf::from(home))
}

pub fn session_file_path() -> anyhow::Result<PathBuf> {
    Ok(home_dir()?.join(SESSION_DIR).join(SESSION_FILE))
}

pub fn save_user_jwt(jwt: &str) -> anyhow::Result<PathBuf> {
    let path = session_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, jwt.trim())?;
    Ok(path)
}

pub fn load_user_jwt() -> anyhow::Result<String> {
    let path = session_file_path()?;
    let jwt = fs::read_to_string(&path).map_err(|_| {
        anyhow::anyhow!("missing session: run `stablectl signer login --jwt <TOKEN>` first")
    })?;

    let trimmed = jwt.trim();
    if trimmed.is_empty() {
        anyhow::bail!("invalid session: empty JWT in {}", path.display());
    }

    Ok(trimmed.to_string())
}

pub fn clear_user_jwt() -> anyhow::Result<bool> {
    let path = session_file_path()?;
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_file(path)?;
    Ok(true)
}
