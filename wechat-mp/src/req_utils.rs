use crate::SaasContext;
use crate::{NoError, WechatEncryptError, WechatError, WechatToken};
use crate::{Wechat, WechatResult};
use lazy_static::*;
use log::info;
use maplit::hashmap;
use reqwest::get;
use reqwest::Client;
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

lazy_static! {
    static ref WECHAT_API: Url = Url::parse("https://api.weixin.qq.com/").unwrap();
}

#[derive(Deserialize, Serialize)]
pub struct EmptyApiResult {}

pub(crate) trait ToApiResult {
    fn to_api_result<T: Sized + DeserializeOwned>(&self) -> WechatResult<T>;
}

impl ToApiResult for String {
    fn to_api_result<T: Sized + DeserializeOwned>(&self) -> WechatResult<T> {
        use serde_json::Value;
        let value: Value = serde_json::from_str(&self)?;
        if let Value::Object(v) = &value {
            match (v.get("errcode"), v.get("errmsg")) {
                (Some(code), Some(msg)) if code != 0 => {
                    return Err(WechatError::EncryptError {
                        source: WechatEncryptError::ApiRequestError {
                            msg: format!("code: {}, msg: {:?}", code, msg),
                            source: Box::new(NoError),
                        },
                    });
                }
                _ => (),
            }
        }
        let item = serde_json::from_value(value)?;
        Ok(item)
    }
}

#[derive(Deserialize, Default, Debug, Clone, PartialEq)]
struct GetAccessTokenResp {
    /// 获取到的凭证
    pub access_token: String,
    /// 凭证有效时间，单位：秒
    pub expires_in: i32,
}

pub fn get_url_with_token(
    url: &str,
    token: Option<String>,
    query: Option<HashMap<String, String>>,
) -> WechatResult<Url> {
    let mut u: Url = Url::options()
        .base_url(Some(&WECHAT_API))
        .parse(url)
        .map_err(|e| WechatError::ParseError(format!("{:?}", e)))?;
    {
        let mut query_kv = u.query_pairs_mut();
        if let Some(token) = token {
            query_kv.append_pair("access_token", token.as_str());
        }
        if let Some(kv) = query {
            for (k, v) in kv {
                query_kv.append_pair(k.as_str(), v.as_str());
            }
        }
    }
    Ok(u)
}
pub fn get_url(url: &str, query: Option<HashMap<String, String>>) -> WechatResult<Url> {
    get_url_with_token(url, None, query)
}

impl Wechat {
    /// 获取token
    pub async fn get_access_token(&self, context: &SaasContext) -> WechatResult<WechatToken> {
        let config = self.saas_resolver.resolve_config(self, context).await?;

        if let Some(token) = self.token_provider.get_token(self, context).await? {
            return Ok(token);
        }
        // get lock
        let resolver = self
            .token_provider
            .lock_token_resolver(self, context)
            .await?;
        // double check
        if let Some(token) = self.token_provider.get_token(self, context).await? {
            return Ok(token);
        }
        let url = get_url(
            "cgi-bin/token",
            Some(hashmap! {
                "grant_type".into() => "client_credential".into(),
                "appid".into() => config.app_id,
                "secret".into() => config.app_secret,
            }),
        )?;
        let resp: GetAccessTokenResp = get(url).await?.text().await?.to_api_result()?;

        let token = WechatToken::new_relative(resp.access_token, resp.expires_in);

        info!("获取到新token:{:?}, {:?}", context, token);

        self.token_provider
            .set_token(self, context, Some(token.clone()))
            .await?;

        // release lock
        self.token_provider
            .unlock_token_resolver(self, context, resolver)
            .await?;

        Ok(token)
    }

    pub(crate) async fn get_url(
        &self,
        context: &SaasContext,
        url: &str,
        query: Option<HashMap<String, String>>,
    ) -> WechatResult<Url> {
        let token = self.get_access_token(context).await?;
        get_url_with_token(url, Some(token.token), query)
    }
    pub(crate) async fn api_post<T: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        context: &SaasContext,
        url: &str,
        query: Option<HashMap<String, String>>,
        body: &T,
    ) -> WechatResult<R> {
        let url = self.get_url(context, url, query).await?;
        let client = Client::new();
        let result = client.post(url.clone()).json(&body).send().await?;

        let mut result = result.text().await?;

        if result.is_empty() {
            result = "{}".into();
        }

        info!("post: {}, resp = {}", url, result);

        let result = result.to_api_result()?;

        Ok(result)
    }
    pub(crate) async fn api_get<R: DeserializeOwned>(
        &self,
        context: &SaasContext,
        url: &str,
        query: Option<HashMap<String, String>>,
    ) -> WechatResult<R> {
        let url = self.get_url(context, url, query).await?;
        let result = reqwest::get(url).await?.text().await?.to_api_result()?;
        Ok(result)
    }

    // pub(crate) async fn apt_upload<R: DeserializeOwned>(
    //     &self,
    //     context: &SaasContext,
    //     url: &str,
    //     query: Option<HashMap<String, String>>,
    // ) -> Result<R, WechatError> {
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_apiresult_serde_error() {
        let json = r#"{"errcode":40013,"errmsg":"invalid appid"}"#;
        let r: WechatResult<GetAccessTokenResp> = json.to_string().to_api_result();
        if let WechatResult::Err(_) = r {
            assert_eq!(true, true);
        } else {
            assert_eq!(true, false);
        }
    }

    #[test]
    fn test_apiresult_serde_ok() {
        let json = r#"{"access_token":"ACCESS_TOKEN","expires_in":7200}"#;
        let r: GetAccessTokenResp = json.to_string().to_api_result().unwrap();
        assert_eq!(
            GetAccessTokenResp {
                access_token: "ACCESS_TOKEN".into(),
                expires_in: 7200,
            },
            r
        );
    }

    #[test]
    fn test_unit() {
        let json = r#"{"errcode":0,"errmsg":"ok"}"#;
        let r: EmptyApiResult = serde_json::from_str(&json).unwrap();
    }
}
