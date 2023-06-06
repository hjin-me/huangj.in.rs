use super::errors::WechatEncryptError;
use crate::core::utils::iso_date_format;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WechatToken {
    pub token: String,
    #[serde(with = "iso_date_format")]
    pub expire_at: DateTime<Utc>,
}

impl WechatToken {
    pub fn new_relative(token: String, expire_in_seconds: i32) -> Self {
        use chrono::Duration;
        WechatToken {
            token,
            expire_at: chrono::Utc::now() + Duration::seconds(expire_in_seconds as i64),
        }
    }

    pub fn remain_ttl(&self) -> i64 {
        use chrono::Duration;
        let ttl: Duration = self.expire_at - chrono::Utc::now();
        let ttl = ttl.num_seconds();
        if ttl < 0 {
            0
        } else {
            ttl
        }
    }
}

/// 微信基本配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WechatConfig {
    /// 对接的token
    pub echo_token: String,
    /// 加密的密钥
    pub key: Option<Vec<u8>>,
    /// app id
    pub app_id: String,
    /// app secret
    pub app_secret: String,
    /// 1、在微信公众号请求用户网页授权之前，开发者需要先到公众平台官网中的
    /// “开发 - 接口权限 - 网页服务 - 网页帐号 - 网页授权获取用户基本信息”的配置选项中，修改授权回调域名。
    /// 请注意，这里填写的是域名（是一个字符串），而不是URL，因此请勿加 http:// 等协议头；
    ///
    /// 授权回调域名配置规范为全域名，比如需要网页授权的域名为：www.qq.com，
    /// 配置以后此域名下面的页面http://www.qq.com/music.html 、 http://www.qq.com/login.html 都可以进行OAuth2.0鉴权。
    /// 但http://pay.qq.com 、 http://music.qq.com 、 http://qq.com 无法进行OAuth2.0鉴权
    pub oauth_redirect_url: String,
}

impl WechatConfig {
    pub fn decode_aes_key(key: &String) -> Result<Option<Vec<u8>>, WechatEncryptError> {
        crate::message::crypt::decode_aes_key(key)
    }
    pub fn new(
        key: Option<Vec<u8>>,
        app_id: String,
        app_secret: String,
        echo_token: String,
        oauth_redirect_url: String,
    ) -> Self {
        WechatConfig {
            echo_token,
            key,
            app_id,
            app_secret,
            oauth_redirect_url,
        }
    }
}

impl Default for WechatConfig {
    fn default() -> Self {
        Self {
            echo_token: "".into(),
            key: None,
            app_id: "".into(),
            app_secret: "".into(),
            oauth_redirect_url: "".into(),
        }
    }
}

#[derive(Copy, Debug, Clone)]
pub struct SaasContext {
    pub id: u64,
}

impl SaasContext {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}
