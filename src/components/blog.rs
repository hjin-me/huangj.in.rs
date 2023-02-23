use crate::backend::blog::{get_one_blog, Post};
use leptos::*;
use leptos_router::*;
use std::fmt::Display;
use tracing::debug;

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    _ = GetSingleBlog::register();
}

#[server(GetSingleBlog, "/api")]
pub async fn get_single_blog() -> Result<Post, ServerFnError> {
    let post = get_one_blog(36)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    debug!("post: {:#?}", post);
    Ok(post)
}

#[allow(non_snake_case)]
#[component]
pub fn SingleBlog(cx: Scope) -> impl IntoView {
    let post = create_resource(cx, || (), |_| async { get_single_blog().await });
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
pub fn Blog(cx: Scope, #[prop()] post: Post) -> impl IntoView {
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
                        {from_now(post.updated_at).unwrap()}
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
                <div inner_html=post.body_html></div>
            </div>
        </article>
    }
}

use std::ops::Sub;
use time::macros::format_description;
use time::OffsetDateTime;

// This filter does not have extra arguments
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
