pub mod blog;
pub mod es;
pub mod github_hook;
pub mod github_issues;

use anyhow::Result;
use elasticsearch::Elasticsearch;
use serde::Deserialize;
use tracing::{info, trace};
#[derive(Debug, Deserialize)]
pub struct Config {
    pub github_token: String,
    pub github_repo: String,
    pub github_owner: String,
    pub es_url: String,
}
pub async fn serv(es_client: &Elasticsearch, conf: &Config) -> Result<()> {
    es::init(conf.es_url.as_str()).expect("初始化ES失败");

    trace!("开始同步所有 issue...");
    github_issues::sync_all_issues(
        &conf.github_token,
        &conf.github_owner,
        &conf.github_repo,
        es_client,
    )
    .await?;
    info!("issue 同步完毕");
    Ok(())
}
