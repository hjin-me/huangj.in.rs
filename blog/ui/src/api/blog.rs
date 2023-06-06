#[cfg(feature = "ssr")]
use elasticsearch::Elasticsearch;
use leptos::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    let _ = GetSingleBlog::register();
    let _ = GetBlogs::register();
}

#[server(GetSingleBlog, "/api")]
pub async fn get_single_blog(cx: Scope, id: u64) -> Result<BlogDisplay, ServerFnError> {
    let es_client = use_context::<std::sync::Arc<Elasticsearch>>(cx).ok_or(
        ServerFnError::ServerError("Elasticsearch client not found".to_string()),
    )?;
    let post = biz::blog::get_one_blog(&es_client, id)
        .await
        .map(BlogDisplay::from)
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    Ok(post)
}

#[server(GetBlogs, "/api")]
pub async fn get_blogs(
    cx: Scope,
    filter: Option<String>,
) -> Result<Vec<BlogAbbrDisplay>, ServerFnError> {
    let es_client = use_context::<std::sync::Arc<Elasticsearch>>(cx).ok_or(
        ServerFnError::ServerError("Elasticsearch client not found".to_string()),
    )?;
    let posts = biz::blog::get_blogs_with_filter(&es_client, filter)
        .await
        .map(|ps| {
            ps.iter()
                .map(|p| {
                    let p = p.clone();
                    BlogAbbrDisplay::from(p)
                })
                .collect()
        })
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    Ok(posts)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogAbbrDisplay {
    pub number: u64,
    pub title: String,
    pub created_at: OffsetDateTime, //"2017-06-05T02:27:43Z",
    pub created_from_now: String,
    pub updated_at: OffsetDateTime, //String,//"2018-05-23T16:30:12Z",
    pub updated_from_now: String,
}

#[cfg(feature = "ssr")]
impl From<biz::blog::Post> for BlogAbbrDisplay {
    fn from(p: biz::blog::Post) -> Self {
        BlogAbbrDisplay {
            number: p.number,
            title: p.title,
            created_at: p.created_at,
            created_from_now: from_now(p.created_at).unwrap_or(p.created_at.to_string()),
            updated_at: p.updated_at,
            updated_from_now: from_now(p.updated_at).unwrap_or(p.updated_at.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogDisplay {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub labels: Vec<PostLabel>,
    pub state: String,              // "open",
    pub created_at: OffsetDateTime, //"2017-06-05T02:27:43Z",
    pub created_from_now: String,
    pub updated_at: OffsetDateTime, //String,//"2018-05-23T16:30:12Z",
    pub updated_from_now: String,
    pub outdated_info: String,
    pub body_html: String, // "## 如
}

#[cfg(feature = "ssr")]
impl From<biz::blog::Post> for BlogDisplay {
    fn from(p: biz::blog::Post) -> Self {
        let outdated_info = outdated(&p);
        BlogDisplay {
            id: p.id,
            number: p.number,
            title: p.title,
            labels: p
                .labels
                .iter()
                .map(|l| PostLabel {
                    name: l.name.clone(),
                    description: l.description.clone(),
                })
                .collect(),
            state: p.state,
            created_at: p.created_at,
            created_from_now: from_now(p.created_at).unwrap_or(p.created_at.to_string()),
            updated_at: p.updated_at,
            updated_from_now: from_now(p.updated_at).unwrap_or(p.updated_at.to_string()),
            outdated_info,
            body_html: p.body_html,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostLabel {
    pub name: String,        //"Publish",
    pub description: String, // "可以被展现的文章"
}

// This filter does not have extra arguments
#[cfg(feature = "ssr")]
pub fn from_now(s: OffsetDateTime) -> anyhow::Result<String> {
    use std::ops::Sub;
    let d = s.sub(OffsetDateTime::now_utc());
    let append: &str = if d.is_positive() { "后" } else { "前" };
    if d.whole_seconds().abs() < 60 {
        return Ok("刚才".to_string());
    }
    if d.whole_minutes().abs() < 60 {
        return Ok(format!("{} 分钟{}", d.whole_minutes().abs(), append));
    }
    if d.whole_hours().abs() < 24 {
        return Ok(format!("{} 小时{}", d.whole_hours().abs(), append));
    }
    if d.whole_days().abs() < 30 {
        return Ok(format!("{} 天{}", d.whole_days().abs(), append));
    }
    if d.whole_seconds().abs() / 30 / 24 / 60 / 60 < 12 {
        return Ok(format!(
            "{} 个月{}",
            d.whole_seconds().abs() / 30 / 24 / 60 / 60,
            append
        ));
    }
    Ok(format!(
        "{} 年{}",
        d.whole_seconds().abs() / 365 / 24 / 60 / 60,
        append
    ))
}

#[cfg(feature = "ssr")]
fn outdated(post: &biz::blog::Post) -> String {
    use std::ops::Sub;
    let mut outdated = false;
    for o in &post.labels {
        if o.name == "Outdated" {
            outdated = true
        }
    }
    let mut outdated_info = "latest";
    if outdated {
        outdated_info = "outdated"
    } else {
        // modify_expired, post_expired
        if post
            .updated_at
            .sub(OffsetDateTime::now_utc())
            .whole_days()
            .abs()
            > 365
        {
            outdated_info = "modify_expired"
        } else if post
            .created_at
            .sub(OffsetDateTime::now_utc())
            .whole_days()
            .abs()
            > 500
        {
            outdated_info = "post_expired"
        }
    }
    outdated_info.to_string()
}

#[cfg(all(test, feature = "ssr"))]
mod test {
    use super::*;
    use time::{format_description, OffsetDateTime};
    #[test]
    fn test_datetime() {
        let s = OffsetDateTime::now_utc();
        dbg!("{}", from_now(s).unwrap());
    }
    #[tokio::test]
    async fn test_from_now() {
        let format = format_description::parse(
            "[year]-[month]-[day] [hour padding:none]:[minute]:[second].[subsecond] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]",
        ).unwrap();

        // let format = format_description!(
        //     "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]"
        // );
        println!("{}", OffsetDateTime::now_utc().format(&format).unwrap());
        let s = "2022-02-03 2:07:03.410441 +00:00:00";
        let date = OffsetDateTime::parse(s, &format).unwrap();
        assert_eq!("2022-02-03 2:07:03.410441 +00:00:00", date.to_string())
    }
}
