use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[component]
pub fn BlogApp(cx: Scope) -> impl IntoView {
    // provide_meta_context(cx);
    view! {
            cx,
        <Html lang="zh-hans"/>
        <Title
      // reactively sets document.title when `name` changes
      text="黄进的个人网站 - HuangJ.in"
      // applies the `formatter` function to the `text` value
      formatter=|text| format!("{text} is your name")
    />
        <Meta name="description" content="黄进的个人网站"/>
    <Meta name="keywords" content="HJin.me,HuangJ.in,黄进"/>
    <Stylesheet href="/pkg/hj.css"/>
            <Router>
        <Header />
    <div class="container-fluid main">
        <div class="row">
            <div class="col-md-3 col-xl-2">
                <Nav />
            </div>
            <div class="col-md-9 col-xl-5">
                 "this is container"
            </div>
        </div>
    </div>
        <Footer />
            </Router>
        }
}
#[allow(non_snake_case)]
#[component]
pub fn Header(cx: Scope) -> impl IntoView {
    view! {
            cx,
        <header class="container-fluid site-header" role="banner">
        <div class="row header-row">
            <div class="col-md-3 col-xl-2">
                <div class="header-title"><a href="/" class="site-title" rel="author"> "黄进的个人网站" </a></div>
            </div>
            <div class="col-md-9 col-xl-10 align-items-center justify-content-end d-none d-md-flex">
                <form action="/search" method="get"><input name="query" class="query-input" placeholder="搜索"/></form>
            </div>
        </div>
    </header>
    }
}
#[allow(non_snake_case)]
#[component]
pub fn Footer(cx: Scope) -> impl IntoView {
    view! {
                cx,
            <footer class="site-footer">
        <div class="miit"><a href="https://beian.miit.gov.cn/" target="_blank">"鄂ICP备15005485号-1"</a></div>
        <div class="license"> "本作品采用"<a rel="license" href="https://creativecommons.org/licenses/by-nc-sa/4.0/">
            "知识共享署名-非商业性使用-相同方式共享 4.0 国际许可协议" </a>"进行许可。"
        </div>
        <div class="license-icon"><a rel="license" href="https://creativecommons.org/licenses/by-nc-sa/4.0/"> <img
            alt="知识共享许可协议" style="border-width:0" src="https://licensebuttons.net/l/by-nc-sa/4.0/80x15.png"/> </a></div>
    </footer>
        }
}

#[allow(non_snake_case)]
#[component]
pub fn Nav(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <div class="sidebar-col d-none d-md-block">
        <div class="menu">
            <a href="/blog" class="menu-item active">"文章"</a>
            <a href="https://github.com/hjin-me" target="_blank" class="menu-item">"Github"</a>
        </div>
        </div>
    }
}
