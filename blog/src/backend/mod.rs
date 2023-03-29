pub mod blog;
#[cfg(feature = "ssr")]
pub mod es;
#[cfg(feature = "ssr")]
pub mod github_hook;
pub mod github_issues;

use anyhow::Result;
use serde::Deserialize;
use tracing::{info, trace};
#[derive(Debug, Deserialize)]
pub struct Config {
    pub github_token: String,
    pub github_repo: String,
    pub github_owner: String,
    pub es_url: String,
}
#[cfg(feature = "ssr")]
pub async fn serv(conf: &Config) -> Result<()> {
    es::init(conf.es_url.as_str()).expect("初始化ES失败");
    let client = es::get_client().await?;

    trace!("开始同步所有 issue...");
    github_issues::sync_all_issues(
        &conf.github_token,
        &conf.github_owner,
        &conf.github_repo,
        &client,
    )
    .await?;
    info!("issue 同步完毕");
    Ok(())
}
