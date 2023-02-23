use crate::backend::blog::{get_one_blog, Post, PostLabel};
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogDisplay {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub labels: Vec<PostLabel>,
    pub state: String,              // "open",
    pub created_at: OffsetDateTime, //"2017-06-05T02:27:43Z",
    pub updated_at: OffsetDateTime, //String,//"2018-05-23T16:30:12Z",
    pub updated_from_now: String,
    pub outdated_info: String,
    pub body_html: String, // "## 如
}

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    let _ = GetSingleBlog::register();
}

#[server(GetSingleBlog, "/api")]
pub async fn get_single_blog(id: u64) -> Result<BlogDisplay, ServerFnError> {
    let post = get_one_blog(id)
        .await
        .map(|p| {
            let outdated_info = outdated(&p);
            BlogDisplay {
                id: p.id,
                number: p.number,
                title: p.title,
                labels: p.labels,
                state: p.state,
                created_at: p.created_at,
                updated_at: p.updated_at,
                outdated_info,
                updated_from_now: from_now(p.updated_at).unwrap_or(p.updated_at.to_string()),
                body_html: p.body_html,
            }
        })
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    debug!("post: {:#?}", post);
    Ok(post)
}

#[allow(non_snake_case)]
#[component]
pub fn SingleBlog(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);
    let post = create_resource(
        cx,
        move || {
            params.with(|p| {
                p.get("id")
                    .cloned()
                    .map(|i| i.parse::<u64>().unwrap_or_default())
                    .unwrap_or_default()
            })
        },
        get_single_blog,
    );
    let post_view = move || {
        post.with(cx, |post| {
            let post = post.clone().unwrap();
            view! {
                cx,
                 <Blog post=post/>
            }
        })
    };

    view! {
            cx,
            <main class="page-content" aria-label="Content">
        <div class="wrapper">
        <Suspense fallback=move || view! { cx, <p>"Loading posts..."</p> }>
           {post_view}
        </Suspense>
        </div>
    </main>
        }
}

#[allow(non_snake_case)]
#[component]
pub fn Blog(cx: Scope, #[prop()] post: BlogDisplay) -> impl IntoView {
    let format = time::format_description::parse("[year]/[month]/[day]").unwrap();

    let outdated_view = match post.outdated_info.as_str() {
        "outdated" => view! {
            cx,
            <div class="alert alert-danger">
            "警告：本文已被标记为过期存档，文中所描述的信息已发生改变，请不要使用。"
        </div>
        },
        "modify_expired" => view! {
            cx,
            <div class="alert alert-warning">
            "提醒：本文最后更新于" {post.updated_at.format(&format)}
            "，文中所描述的信息可能已发生改变，请谨慎使用。"
        </div>
        },
        "post_expired" => view! {
            cx,
            <div class="alert alert-warning">
            "提醒：本文发布于" {post.created_at.format(&format)}
            "，文中所描述的信息可能已发生改变，请谨慎使用。"
        </div>
        },
        _ => view! {
            cx,
            <div/>
        },
    };

    view! {
        cx,
        <article
                class="post h-entry"
                itemScope
                itemType="https://schema.org/BlogPosting"
            >
            <header class="post-header">
                <h1 class="post-title p-name" itemProp="name headline">{post.title}</h1>
                <p class="post-meta">
                    "最后更新于"
                    <time
                        class="dt-published"
                        dateTime=datetime(post.updated_at).unwrap()
                        itemProp="dateModified"
                    >
                        {post.updated_from_now}
                    </time>
                    "•"
                    <span itemProp="author" itemScope itemType="http://schema.org/Person">
                    <span class="p-author h-card" itemProp="name">
                      "HJin"
                    </span>
                  </span>
                </p>
            </header>
            <div class="post-content e-content markdown-body" id="write" itemProp="articleBody">
                {outdated_view}
                <div inner_html=post.body_html></div>
            </div>
        </article>
    }
}

use std::ops::Sub;
use time::macros::format_description;
use time::OffsetDateTime;

// This filter does not have extra arguments
#[cfg(feature = "ssr")]
pub fn from_now(s: OffsetDateTime) -> anyhow::Result<String> {
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
    return Ok(format!(
        "{} 年{}",
        d.whole_seconds().abs() / 365 / 24 / 60 / 60,
        append
    ));
}
#[cfg(feature = "ssr")]
fn outdated(post: &Post) -> String {
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

pub fn datetime(s: OffsetDateTime) -> anyhow::Result<String> {
    let format = format_description!("[year]-[month]-[day]T[hour padding:none]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]");
    Ok(s.format(&format)?)
}

#[cfg(test)]
#[test]
fn test_datetime() {
    let s = OffsetDateTime::now_utc();
    dbg!(datetime(s).unwrap());
    dbg!("{}", from_now(s).unwrap());
}
