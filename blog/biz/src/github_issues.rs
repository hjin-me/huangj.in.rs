use anyhow::Result;
use elasticsearch::indices::{IndicesCreateParts, IndicesExistsParts};
use elasticsearch::{Elasticsearch, UpdateParts};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::trace;

pub async fn sync_all_issues(
    github_token: &String,
    owner: &String,
    repo: &String,
    es_client: &Elasticsearch,
) -> Result<()> {
    let request_url = format!("https://api.github.com/repos/{owner}/{repo}/issues");
    trace!("request_url: {}", request_url);
    trace!("github_token: {}", github_token);
    trace!("es_client: {:?}", es_client);
    let mut header = HeaderMap::new();
    header.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {github_token}")).unwrap(),
    );
    header.insert(
        ACCEPT,
        HeaderValue::from_str("application/vnd.github.full+json").unwrap(),
    );
    header.insert(USER_AGENT, HeaderValue::from_str("rust-client").unwrap());
    header.insert("Time-Zone", HeaderValue::from_str("Asia/Shanghai").unwrap());
    let client = reqwest::ClientBuilder::new()
        .default_headers(header)
        .timeout(Duration::new(10, 0))
        .build()
        .unwrap();
    let response = client.get(&request_url).send().await?;
    // println!("{:?}", response.bytes().await.unwrap());
    let issues: Vec<Issue> = response.json().await?;
    // println!("{:?}", &issuess);

    const INDEX_NAME: &str = "blog";
    let exist = index_exist(es_client, INDEX_NAME).await?;
    if !exist {
        create_index(es_client, INDEX_NAME).await?;
    }
    for issue in issues {
        upsert_issue(es_client, INDEX_NAME, &issue).await?
    }
    Ok(())
}
#[derive(Deserialize, Serialize, Debug)]
pub struct IssueUser {
    login: String,
    id: u64,
    node_id: String,
    avatar_url: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct IssueLabel {
    // "id": 942521019,
    // "node_id": "MDU6TGFiZWw5NDI1MjEwMTk=",
    // "url": "https://api.github.com/repos/hjin-me/blog/labels/Publish",
    name: String,        //"Publish",
    color: String,       //"0e8a16",
    description: String, // "可以被展现的文章"
}
#[derive(Deserialize, Serialize, Debug)]
pub struct IssueReactions {
    // "url": "https://api.github.com/repos/hjin-me/blog/issues/16/reactions",
    total_count: u64, // 0,
                      // "+1": 0,
                      // "-1": 0,
                      // "laugh": 0,
                      // "hooray": 0,
                      // "confused": 0,
                      // "heart": 0,
                      // "rocket": 0,
                      // "eyes": 0
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Issue {
    // "url": "https://api.github.com/repos/hjin-me/blog/issues/16",
    // "repository_url": "https://api.github.com/repos/hjin-me/blog",
    // "labels_url": "https://api.github.com/repos/hjin-me/blog/issues/16/labels{/name}",
    // "comments_url": "https://api.github.com/repos/hjin-me/blog/issues/16/comments",
    comments_url: String,
    // "events_url": "https://api.github.com/repos/hjin-me/blog/issues/16/events",
    // "html_url": "https://github.com/hjin-me/blog/issues/16",
    // "id": 233479897,
    id: u64,
    // "node_id": "MDU6SXNzdWUyMzM0Nzk4OTc=",
    node_id: String,
    // "number": 16,
    number: u64,
    // "title": "ThreeJS 使用总结 FAQ",
    title: String,

    user: IssueUser,
    labels: Vec<IssueLabel>,
    state: String, // "open",
    // "locked": false,
    // "assignee": null,
    // "assignees": [
    //
    // ],
    // "milestone": null,
    comments: u64,      // 0,
    created_at: String, //"2017-06-05T02:27:43Z",
    updated_at: String, //String,//"2018-05-23T16:30:12Z",
    // "closed_at": null,
    // "author_association": "OWNER",
    // "active_lock_reason": null,
    body_text: String, // "## 如何
    body_html: String, // "## 如
    reactions: IssueReactions, // "timeline_url": "https://api.github.com/repos/hjin-me/blog/issues/16/timeline",
                               // "performed_via_github_app": null,
                               // "state_reason": null
}

async fn index_exist(client: &Elasticsearch, index: &str) -> Result<bool, elasticsearch::Error> {
    let resp = client
        .indices()
        .exists(IndicesExistsParts::Index(&[&index]))
        .request_timeout(Duration::new(1, 0))
        .send()
        .await?; //.map(|r| r.status_code() == 200)
    if resp.status_code().is_success() {
        Ok(true)
    } else {
        let b = resp.bytes().await?;
        trace!("index_exist: {:?}", b);
        Ok(false)
    }
}

async fn create_index(client: &Elasticsearch, index: &str) -> Result<(), elasticsearch::Error> {
    let resp = client
        .indices()
        .create(IndicesCreateParts::Index(&index))
        .body(json!({
          "settings": {
            "number_of_shards": 1,
            "number_of_replicas": 0
          },
          "mappings": {
              "properties": {
                "body_html": {
                  "type": "text"
                },
                "body_text": {
                  "type": "text",
                  "analyzer": "ik_max_word",
                  "search_analyzer": "ik_max_word"
                },
                "number": {
                  "type": "integer"
                },
                "closed": {
                  "type": "boolean"
                },
                "created_at": {
                  "type": "date"
                },
                "title": {
                  "type": "text",
                  "analyzer": "ik_max_word",
                  "search_analyzer": "ik_max_word"
                },
                "updated_at": {
                  "type": "date"
                },
                "labels": {
                  "type": "nested",
                  "properties": {
                    "description": {
                      "type": "text"
                    },
                    "name": {
                      "type": "keyword"
                    }
                  }
                }
              }
            }
        }))
        .request_timeout(Duration::new(1, 0))
        .send()
        .await?;
    // if resp.status_code().is_success() {
    //     let b = resp.bytes().await?;
    //     trace!("index_exist: {:?}", b);
    // }
    match resp.error_for_status_code() {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

async fn upsert_issue(
    client: &Elasticsearch,
    index: &str,
    issue: &Issue,
) -> Result<(), elasticsearch::Error> {
    let resp = client
        .update(UpdateParts::IndexId(&index, &issue.id.to_string()))
        .body(json!({
            "doc": issue,
            "doc_as_upsert": true
        }))
        // .index(IndexParts::Index(&index))
        // .body(&issue)
        .request_timeout(Duration::new(1, 0))
        .send()
        .await?;
    match resp.error_for_status_code() {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[derive(Debug, Deserialize)]
    struct Config {
        // github_sec: String,
        github_token: String,
        es_url: String,
    }

    #[tokio::test]
    async fn test_all_issues() {
        let contents =
            fs::read_to_string("./config.toml").expect("Should have been able to read the file");
        let conf: Config = toml::from_str(contents.as_str()).unwrap();

        let github_token = conf.github_token;
        let owner = "hjin-me".to_string();
        let repo = "blog".to_string();

        let transport =
            elasticsearch::http::transport::Transport::single_node(conf.es_url.as_str()).unwrap();
        let client = Elasticsearch::new(transport);
        sync_all_issues(&github_token, &owner, &repo, &client)
            .await
            .unwrap();
        // assert_eq!(1, 2)
    }

    #[tokio::test]
    async fn es() {
        let transport =
            elasticsearch::http::transport::Transport::single_node("http://127.0.0.1:9200")
                .unwrap();
        let client = Elasticsearch::new(transport);
        let rand_index = format!("test_{}", time::OffsetDateTime::now_utc().unix_timestamp());
        println!("{:?}", index_exist(&client, &rand_index).await.unwrap());
        create_index(&client, &rand_index).await.unwrap();
        assert_eq!(true, index_exist(&client, &rand_index).await.unwrap());

        upsert_issue(
            &client,
            &rand_index,
            &Issue {
                comments_url: "https://api.github.com/repos/hjin-me/blog/issues/16/comments"
                    .to_string(),
                id: 233479897,
                node_id: "MDU6SXNzdWUyMzM0Nzk4OTc=".to_string(),
                number: 16,
                title: "ThreeJS 使用总结 FAQ".to_string(),
                user: IssueUser {
                    login: "hjin-me".to_string(),
                    id: 102523,
                    node_id: "MDQ6VXNlcjEwMjUyMw==".to_string(),
                    avatar_url: "https://avatars.githubusercontent.com/u/102523?v=4".to_string(),
                },
                labels: vec![],
                state: "".to_string(),
                comments: 0,
                created_at: "2018-05-23T16:30:10Z".to_string(),
                updated_at: "2018-05-23T16:30:10Z".to_string(),
                body_text: "this is text".to_string(),
                body_html: "<h1>this is <b>HTML</b></h1>".to_string(),
                reactions: IssueReactions { total_count: 0 },
            },
        )
        .await
        .unwrap();
    }
}
