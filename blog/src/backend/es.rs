use anyhow::Result;
use elasticsearch::Elasticsearch;

pub fn init(es_url: &str) -> Result<Elasticsearch> {
    let transport = elasticsearch::http::transport::Transport::single_node(es_url)?;
    let client = Elasticsearch::new(transport);
    Ok(client)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn es() {
        let client = init("http://127.0.0.1:9200").unwrap();
        let rand_index = format!("test_{}", time::OffsetDateTime::now_utc().unix_timestamp());
    }
}
