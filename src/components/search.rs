use crate::components::blog::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[component]
pub fn SearchPage(cx: Scope) -> impl IntoView {
    let query = use_query_map(cx);
    let posts = create_resource(
        cx,
        move || query.with(|p| p.get("query").cloned()),
        get_blogs,
    );
    let query_key = query.get().get("query").cloned().unwrap_or_default();

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
                    <h1>"搜索: "{query_key}</h1>
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
