use crate::backend::blog::{get_one_blog, Post};
use leptos::*;
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
    // list of todos is loaded from the server in reaction to changes
    let post: Resource<(), Result<Post, ServerFnError>> =
        create_resource(cx, move || (), move |_| get_single_blog());

    view! {
            cx,
            <main class="page-content" aria-label="Content">
        <div class="wrapper">
            <article
                class="post h-entry"
                itemScope
                itemType="https://schema.org/BlogPosting"
            >
            {move || {post.read(cx).map(move |p| match p {
                        Ok(p) => {
                            view! {
                                cx,
                                <div>"some"</div>
                            }
                        },
                        Err(e) => {
                            dbg!(e);
                            view! {
                                cx,
                                <div>"error..."</div>
                            }
                        }}).unwrap_or_else(move ||  view! {
                                cx,
                                <div>"none..."</div>
                            })} }
                <header class="post-header">
                    <h1 class="post-title p-name" itemProp="name headline">
                        "{{ post.title }}"
                    </h1>
                    <p class="post-meta">
                        "最后更新于"
                        <time
                            class="dt-published"
                            dateTime="{{post.updated_at}}"
                            itemProp="dateModified"
                        >
                            "{{ post.updated_at|from_now }}"
                        </time>
                        "•"
                        <span itemProp="author" itemScope itemType="http://schema.org/Person">
                <span class="p-author h-card" itemProp="name">
                  "HJin"
                </span>
              </span>
                    </p>
            </header>

            </article>
        </div>
    </main>
        }
}
