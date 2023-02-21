use anyhow::Result;
use elasticsearch::Elasticsearch;
use once_cell::sync::OnceCell;
use std::sync::Arc;

static INSTANCE: OnceCell<Arc<Elasticsearch>> = OnceCell::new();

pub fn init_es(es_url: &str) -> Result<()> {
    let transport = elasticsearch::http::transport::Transport::single_node(es_url).unwrap();
    let client = Elasticsearch::new(transport);
    INSTANCE.set(Arc::new(client)).expect("初始化ES失败");
    Ok(())
}

pub fn get_es() -> Result<Arc<Elasticsearch>> {
    match INSTANCE.get() {
        Some(c) => Ok(c.clone()),
        None => Err(anyhow::anyhow!("ES未初始化")),
    }
}

#[cfg(test)]
mod test {
    use crate::data::{get_es, init_es};
    use anyhow::Result;

    #[tokio::test]
    async fn es() {
        init_es("http://127.0.0.1:9200").unwrap();
        let client = get_es().unwrap();
        let rand_index = format!("test_{}", time::OffsetDateTime::now_utc().unix_timestamp());
        // println!("{:?}", index_exist(&client, &rand_index).await.unwrap());
        // create_index(&client, &rand_index).await.unwrap();
        // assert_eq!(true, index_exist(&client, &rand_index).await.unwrap());
        //
        // upsert_issue(
        //     &client,
        //     &rand_index,
        //     &Issue {
        //         comments_url: "https://api.github.com/repos/hjin-me/blog/issues/16/comments"
        //             .to_string(),
        //         id: 233479897,
        //         node_id: "MDU6SXNzdWUyMzM0Nzk4OTc=".to_string(),
        //         number: 16,
        //         title: "ThreeJS 使用总结 FAQ".to_string(),
        //         user: IssueUser {
        //             login: "hjin-me".to_string(),
        //             id: 102523,
        //             node_id: "MDQ6VXNlcjEwMjUyMw==".to_string(),
        //             avatar_url: "https://avatars.githubusercontent.com/u/102523?v=4".to_string(),
        //         },
        //         labels: vec![],
        //         state: "".to_string(),
        //         comments: 0,
        //         created_at: "2018-05-23T16:30:10Z".to_string(),
        //         updated_at: "2018-05-23T16:30:10Z".to_string(),
        //         body_text: "this is text".to_string(),
        //         body_html: "<h1>this is <b>HTML</b></h1>".to_string(),
        //         reactions: IssueReactions { total_count: 0 },
        //     },
        // )
        //     .await
        //     .unwrap();
    }
}
