
use anyhow::Result;
use elasticsearch::Elasticsearch;
use once_cell::sync::OnceCell;
use std::sync::Arc;

static INSTANCE: OnceCell<Arc<Elasticsearch>> = OnceCell::new();

pub fn init(es_url: &str) -> Result<()> {
    let transport = elasticsearch::http::transport::Transport::single_node(es_url).unwrap();
    let client = Elasticsearch::new(transport);
    INSTANCE.set(Arc::new(client)).expect("初始化ES失败");
    Ok(())
}

pub async fn get_client() -> Result<Arc<Elasticsearch>> {
    match INSTANCE.get() {
        Some(c) => Ok(c.clone()),
        None => Err(anyhow::anyhow!("ES未初始化")),
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use super::*;

    #[tokio::test]
    async fn es() {
        init("http://127.0.0.1:9200").unwrap();
        let client = get_client().await.unwrap();
        let rand_index = format!("test_{}", time::OffsetDateTime::now_utc().unix_timestamp());
    }
}
