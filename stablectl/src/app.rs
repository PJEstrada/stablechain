use chain_access::adapters::connect_reader;
use chain_access::domain::chain_id::ChainId;
use chain_access::ports::ChainReader;

pub struct App {
    pub reader: Box<dyn ChainReader>,
}

impl App {
    pub async fn init(chain_id: ChainId) -> anyhow::Result<Self> {
        let reader = connect_reader(chain_id)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;


        Ok(Self { reader })
    }
}
