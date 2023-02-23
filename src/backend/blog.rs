#[cfg(feature = "ssr")]
use crate::backend::es::get_client;
use anyhow::{anyhow, Result};
#[cfg(feature = "ssr")]
use elasticsearch::{Elasticsearch, SearchParts};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
#[cfg(feature = "ssr")]
use std::ops::Sub;

#[cfg(feature = "ssr")]
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
#[cfg(feature = "ssr")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostLabel {
    pub name: String,        //"Publish",
    pub description: String, // "可以被展现的文章"
}
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// pub async fn redirect_to_blog(Path(id): Path<u64>) -> impl IntoResponse {
//     let es_client = get_es().await.expect("ES未初始化");
//     let post = get_by_number(&id, "blog", &es_client).await;
//     match post {
//         Ok(p) => axum::response::Redirect::permanent(format!("/blog/{}", p.number).as_str()),
//         Err(_) => axum::response::Redirect::temporary("/blog"),
//     }
// }

#[cfg(not(feature = "ssr"))]
pub async fn get_one_blog(id: u64) -> Result<Post> {
    Ok(Post {
        id,
        number: 0,
        title: "".to_string(),
        labels: vec![],
        state: "".to_string(),
        created_at: time::OffsetDateTime::now_utc(),
        updated_at: time::OffsetDateTime::now_utc(),
        body_html: "".to_string(),
    })
}

#[cfg(feature = "ssr")]
pub async fn get_one_blog(id: u64) -> Result<Post> {
    let es_client = get_client().await?;
    let p = get_by_number(&id, "blog", &es_client).await?;

    let format = time::format_description::parse("[year]/[month]/[day]").unwrap();
    let mut outdated = false;
    for o in &p.labels {
        if o.name == "Outdated" {
            outdated = true
        }
    }
    let mut outdated_info = "latest";
    if outdated {
        outdated_info = "outdated"
    } else {
        // modify_expired, post_expired
        if p.updated_at
            .sub(time::OffsetDateTime::now_utc())
            .whole_days()
            .abs()
            > 365
        {
            outdated_info = "modify_expired"
        } else if p
            .created_at
            .sub(time::OffsetDateTime::now_utc())
            .whole_days()
            .abs()
            > 500
        {
            outdated_info = "post_expired"
        }
    }
    Ok(p)
}
// pub struct PageTemplate<'a> {
//     pub post: Post,
//     pub time_format: Vec<time::format_description::FormatItem<'a>>,
//     pub outdated_info: String,
// }

#[cfg(feature = "ssr")]
pub async fn get_all_blog() -> Result<String> {
    let es_client = get_client().await?;
    let posts = get_latest_with_filter("blog", &es_client, None).await?;
    Ok(json!({
        "posts": posts,
    })
    .to_string())
}

// Any filter defined in the module `filters` is accessible in your template.
// pub mod filters {
//     use std::ops::Sub;
//     use time::macros::format_description;
//     use time::OffsetDateTime;
//     use tracing::trace;
//
//     // This filter does not have extra arguments
//     pub fn from_now<T: std::fmt::Display>(s: T) -> ::askama::Result<String> {
//         let format = format_description!("[year]-[month]-[day] [hour padding:none]:[minute]:[second].[subsecond] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]");
//         let s = s.to_string();
//         trace!("from_now: {}", s);
//         let date = OffsetDateTime::parse(&s, &format).unwrap();
//         let d = date.sub(OffsetDateTime::now_utc());
//         let append: &str = if d.is_positive() { "后" } else { "前" };
//         if d.whole_seconds().abs() < 60 {
//             return Ok("刚才".to_string());
//         }
//         if d.whole_minutes().abs() < 60 {
//             return Ok(format!("{} 分钟{}", d.whole_minutes().abs(), append));
//         }
//         if d.whole_hours().abs() < 24 {
//             return Ok(format!("{} 小时{}", d.whole_hours().abs(), append));
//         }
//         if d.whole_days().abs() < 30 {
//             return Ok(format!("{} 天{}", d.whole_days().abs(), append));
//         }
//         if d.whole_seconds().abs() / 30 / 24 / 60 / 60 < 12 {
//             return Ok(format!(
//                 "{} 个月{}",
//                 d.whole_seconds().abs() / 30 / 24 / 60 / 60,
//                 append
//             ));
//         }
//         return Ok(format!(
//             "{} 年{}",
//             d.whole_seconds().abs() / 365 / 24 / 60 / 60,
//             append
//         ));
//     }
// }

// #[cfg(test)]
// mod test {
//     use time::macros::format_description;
//     use time::{format_description, OffsetDateTime};
//
//     #[tokio::test]
//     async fn test_from_now() {
//         let format = format_description::parse(
//              "[year]-[month]-[day] [hour padding:none]:[minute]:[second].[subsecond] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]",
//          ).unwrap();
//
//         // let format = format_description!(
//         //     "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]"
//         // );
//         println!("{}", OffsetDateTime::now_utc().format(&format).unwrap());
//         let s = "2022-02-03 2:07:03.410441 +00:00:00";
//         let date = OffsetDateTime::parse(&s, &format).unwrap();
//         println!("{}", date);
//         assert_eq!("2022-02-03 2:07:03.410441 +00:00:00", date.to_string())
//     }
// }

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
