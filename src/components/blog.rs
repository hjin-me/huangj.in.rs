use crate::backend::blog::{get_blogs_with_filter, get_one_blog, Post, PostLabel};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use std::ops::Sub;
use time::macros::format_description;
use time::OffsetDateTime;
use tracing::debug;

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
impl From<Post> for BlogDisplay {
    fn from(p: Post) -> Self {
        let outdated_info = outdated(&p);
        BlogDisplay {
            id: p.id,
            number: p.number,
            title: p.title,
            labels: p.labels,
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

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    let _ = GetSingleBlog::register();
    let _ = GetBlogs::register();
}

#[server(GetSingleBlog, "/api")]
pub async fn get_single_blog(id: u64) -> Result<BlogDisplay, ServerFnError> {
    let post = get_one_blog(id)
        .await
        .map(BlogDisplay::from)
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
            let title = post.title.clone();
            view! {
                cx,
                 <Title text=title />
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
            "提醒：本文最后更新于 " {post.updated_at.format(&format)}
            " ，文中所描述的信息可能已发生改变，请谨慎使用。"
        </div>
        },
        "post_expired" => view! {
            cx,
            <div class="alert alert-warning">
            "提醒：本文发布于 " {post.created_at.format(&format)}
            " ，文中所描述的信息可能已发生改变，请谨慎使用。"
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
                    "最后更新于 "
                    <time
                        class="dt-published"
                        dateTime=datetime(post.updated_at).unwrap()
                        itemProp="dateModified"
                    >
                        {post.updated_from_now}
                    </time>
                    " • "
                    <span itemProp="author" itemScope itemType="https://schema.org/Person">
                    <span class="p-author h-card" itemProp="name">
                      " HJin "
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
    Ok(format!(
        "{} 年{}",
        d.whole_seconds().abs() / 365 / 24 / 60 / 60,
        append
    ))
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
// Blogs

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
impl From<Post> for BlogAbbrDisplay {
    fn from(p: Post) -> Self {
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

#[server(GetBlogs, "/api")]
pub async fn get_blogs(filter: Option<String>) -> Result<Vec<BlogAbbrDisplay>, ServerFnError> {
    let posts = get_blogs_with_filter(filter)
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

#[allow(non_snake_case)]
#[component]
pub fn BlogList(cx: Scope) -> impl IntoView {
    let posts = create_resource(cx, || (), |_| async { get_blogs(None).await });
    let posts_view = move || {
        posts.with(cx, |posts| {
            let posts = posts.clone().unwrap();
            posts
                .iter()
                .map(move |post| {
                    view! {
                        cx,
                        <BlogAbbr post=post.clone()/>
                    }
                })
                .collect::<Vec<_>>()
        })
    };

    view! {
        cx,
        <main class="page-content" aria-label="Content">
            <div class="wrapper">
                <div class="home">
                    <Title text="首页" />
                    <Suspense fallback=move || view! { cx, <p>"Loading..."</p> }>
                        <ul class="post-list">
                            {posts_view}
                        </ul>
                    </Suspense>
                </div>
            </div>
        </main>
    }
}

#[allow(non_snake_case)]
#[component]
pub fn BlogAbbr(cx: Scope, #[prop()] post: BlogAbbrDisplay) -> impl IntoView {
    view! {
        cx,
        <li>
            <span class="post-meta">{ post.created_from_now }</span>
            <h3><a href=format!("/blog/{}", post.number) class="post-link">{ post.title }</a></h3>
        </li>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod test {
    use super::*;
    use time::{format_description, OffsetDateTime};
    #[test]
    fn test_datetime() {
        let s = OffsetDateTime::now_utc();
        dbg!(datetime(s).unwrap());
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
