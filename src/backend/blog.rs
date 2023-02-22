use anyhow::{anyhow, Result};
use elasticsearch::{Elasticsearch, SearchParts};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub async fn get_by_number(id: &u64, index: &str, es_client: &Elasticsearch) -> Result<Post> {
    let r = es_client
        .search(SearchParts::Index(&[&index]))
        .body(json!({
            "query": {
                "match": {
                    "number": id
                }
            }
        }))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    match r["hits"]["hits"].as_array().unwrap().get(0) {
        Some(v) => Ok(serde_json::from_value::<Post>(v["_source"].clone()).unwrap()),
        None => Err(anyhow!("没有找到")),
    }
}
pub async fn get_latest_with_filter(
    index: &str,
    es_client: &Elasticsearch,
    filter: Option<&str>,
) -> Result<Vec<Post>> {
    let body = match filter {
        Some(f) => json!({
            "size": 30,
            "query": {
                "multi_match": {
                "query": f,
                "fields" : [ "body_text", "title" ],
                "type": "phrase"
                }
            },
            "sort": [
                {
                    "updated_at": {
                        "order": "desc"
                    }
                }
            ]
        }),
        None => json!({
            "size": 30,
            "sort": [
                {
                    "updated_at": {
                        "order": "desc"
                    }
                }
            ]
        }),
    };
    let r = es_client
        .search(SearchParts::Index(&[&index]))
        .body(body)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    let mut posts = Vec::new();
    for v in r["hits"]["hits"].as_array().unwrap() {
        posts.push(serde_json::from_value::<Post>(v["_source"].clone()).unwrap());
    }
    Ok(posts)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostLabel {
    pub name: String,        //"Publish",
    pub description: String, // "可以被展现的文章"
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub labels: Vec<PostLabel>,
    pub state: String, // "open",
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime, //"2017-06-05T02:27:43Z",
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: time::OffsetDateTime, //String,//"2018-05-23T16:30:12Z",
    pub body_html: String, // "## 如
}
#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_get_by_number() {
        let transport =
            elasticsearch::http::transport::Transport::single_node("http://127.0.0.1:9200")
                .unwrap();
        let es_client = elasticsearch::Elasticsearch::new(transport);
        println!(
            "{:?}",
            super::get_by_number(&35, "blog", &es_client).await.unwrap()
        );
    }
    #[tokio::test]
    async fn test_get_latest_with_filter() {
        let transport =
            elasticsearch::http::transport::Transport::single_node("http://127.0.0.1:9200")
                .unwrap();
        let es_client = elasticsearch::Elasticsearch::new(transport);
        println!(
            "{:?}",
            super::get_latest_with_filter("blog", &es_client, Some("目标"))
                .await
                .unwrap()
        )
    }
}
