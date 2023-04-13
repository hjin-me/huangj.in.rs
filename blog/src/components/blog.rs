use crate::api::blog::{get_blogs, get_single_blog, BlogAbbrDisplay, BlogDisplay};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use time::macros::format_description;
use time::OffsetDateTime;

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
        move |id| get_single_blog(cx, id),
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

pub fn datetime(s: OffsetDateTime) -> anyhow::Result<String> {
    let format = format_description!("[year]-[month]-[day]T[hour padding:none]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]");
    Ok(s.format(&format)?)
}
// Blogs

#[allow(non_snake_case)]
#[component]
pub fn BlogList(cx: Scope) -> impl IntoView {
    let posts = create_resource(cx, || (), move |_| get_blogs(cx, None));
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
