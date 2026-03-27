use crate::cmd::session::load_user_jwt;
use chain_access::LocalKeySigner;
use chain_access::signer::SignerBackend;
use chain_access::signer::SignerBackendType;
use chain_access::signer::privy_signer::PrivySigner;
use chain_access::signer::privy_user_signer::PrivyUserSigner;
use privy_rs::PrivyClient;

pub struct SignerConfig {
    pub signer_backend: SignerBackendType,
    pub key_env: Option<String>,
    pub wallet_id: Option<String>,
}
pub fn build_signer(config: &SignerConfig) -> Result<Box<dyn SignerBackend>, anyhow::Error> {
    let signer: Box<dyn SignerBackend> = match config.signer_backend {
        SignerBackendType::LocalKey => {
            let key_env = config
                .key_env
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("--key-env is required for --signer local-key"))?;
            Box::new(LocalKeySigner::from_env(key_env).map_err(|e| anyhow::anyhow!("{e}"))?)
        }
        SignerBackendType::Privy => {
            let wallet_id = config
                .wallet_id
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("--wallet-id is required for --signer privy"))?;

            // Read the test environment variables
            let app_id = std::env::var("PRIVY_TEST_APP_ID")
                .map_err(|_| anyhow::anyhow!("missing env var: PRIVY_TEST_APP_ID"))?;
            let app_secret = std::env::var("PRIVY_TEST_APP_SECRET")
                .map_err(|_| anyhow::anyhow!("missing env var: PRIVY_TEST_APP_SECRET"))?;

            // Create client with explicit credentials
            let client = PrivyClient::new(app_id, app_secret)?;
            Box::new(PrivySigner::new(client, wallet_id.clone()))
        }
        SignerBackendType::PrivyUser => {
            let wallet_id = config.wallet_id.as_ref().ok_or_else(|| {
                anyhow::anyhow!("--wallet-id is required for --signer privy-user")
            })?;

            let app_id = std::env::var("PRIVY_TEST_APP_ID")
                .map_err(|_| anyhow::anyhow!("missing env var: PRIVY_TEST_APP_ID"))?;
            let app_secret = std::env::var("PRIVY_TEST_APP_SECRET")
                .map_err(|_| anyhow::anyhow!("missing env var: PRIVY_TEST_APP_SECRET"))?;
            let jwt = load_user_jwt()?;

            let client = PrivyClient::new(app_id, app_secret)?;
            Box::new(PrivyUserSigner::new(client, wallet_id.clone(), jwt))
        }
    };
    Ok(signer)
}

// unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use chain_access::signer::SignerBackendType;

    #[test]
    fn test_build_signer() {
        temp_env::with_var(
            "TEST_LOCAL_KEY_123",
            Some("0x1111111111111111111111111111111111111111111111111111111111111111"),
            || {
                let config = SignerConfig {
                    signer_backend: SignerBackendType::LocalKey,
                    key_env: Some("TEST_LOCAL_KEY_123".to_string()),
                    wallet_id: None,
                };
                let signer = build_signer(&config);
                assert!(signer.is_ok());
            },
        );
    }

    #[test]
    fn test_build_signer_privy_missing_env_fails() {
        temp_env::with_var("PRIVY_TEST_APP_ID", None::<String>, || {
            temp_env::with_var("PRIVY_TEST_APP_SECRET", None::<String>, || {
                let config = SignerConfig {
                    signer_backend: SignerBackendType::Privy,
                    key_env: None,
                    wallet_id: Some("test_wallet_id".to_string()),
                };
                let signer = build_signer(&config);
                assert!(signer.is_err());
            });
        });
    }

    #[test]
    fn test_build_signer_privy_user_missing_session_fails() {
        temp_env::with_var("PRIVY_TEST_APP_ID", Some("app_id"), || {
            temp_env::with_var("PRIVY_TEST_APP_SECRET", Some("app_secret"), || {
                temp_env::with_var("HOME", Some("/__stablectl_nonexistent_home__"), || {
                    let config = SignerConfig {
                        signer_backend: SignerBackendType::PrivyUser,
                        key_env: None,
                        wallet_id: Some("test_wallet_id".to_string()),
                    };
                    let signer = build_signer(&config);
                    assert!(signer.is_err());
                });
            });
        });
    }
}
