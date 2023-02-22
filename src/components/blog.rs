use crate::backend::blog::{get_by_number, get_latest_with_filter};
use crate::backend::es::get_client;
use anyhow::Result;
use axum::extract::Path;
use axum::response::IntoResponse;
use serde_json::json;
use std::ops::Sub;

// pub async fn redirect_to_blog(Path(id): Path<u64>) -> impl IntoResponse {
//     let es_client = get_es().await.expect("ES未初始化");
//     let post = get_by_number(&id, "blog", &es_client).await;
//     match post {
//         Ok(p) => axum::response::Redirect::permanent(format!("/blog/{}", p.number).as_str()),
//         Err(_) => axum::response::Redirect::temporary("/blog"),
//     }
// }

pub async fn get_one_blog(Path(id): Path<u64>) -> Result<String> {
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
    Ok(json!({
        "post": p,
        "outdated_info": outdated_info.to_string(),
    })
    .to_string())
}
// pub struct PageTemplate<'a> {
//     pub post: Post,
//     pub time_format: Vec<time::format_description::FormatItem<'a>>,
//     pub outdated_info: String,
// }

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
