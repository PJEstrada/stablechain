use chain_access::LocalKeySigner;
use chain_access::signer::SignerBackend;
use chain_access::signer::SignerBackendType;
use chain_access::signer::privy_signer::PrivySigner;
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
    };
    Ok(signer)
}

// unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use chain_access::signer::SignerBackendType;
    use downcast::Any;

    #[test]
    fn test_build_signer() {
        temp_env::with_var("TEST_LOCAL_KEY_123", Some("0xdeadbeef..."), || {
            let config = SignerConfig {
                signer_backend: SignerBackendType::LocalKey,
                key_env: Some("TEST_LOCAL_KEY_123".to_string()),
                wallet_id: None,
            };
            let signer = build_signer(&config).unwrap();
            let is_local = signer.as_any().downcast_ref::<LocalKeySigner>().is_some();
            assert!(is_local);
        });

        // case for privy
        let config = SignerConfig {
            signer_backend: SignerBackendType::Privy,
            key_env: None,
            wallet_id: Some("test_wallet_id".to_string()),
        };
        let _privy_signer = build_signer(&config);
    }
}
