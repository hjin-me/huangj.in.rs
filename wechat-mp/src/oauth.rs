use crate::{SaasContext, Wechat, WechatResult};
use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};

/// 网页授权的两种scope
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum OauthScope {
    /// 以snsapi_base为scope发起的网页授权，是用来获取进入页面的用户的openid的，
    /// 并且是静默授权并自动跳转到回调页的。用户感知的就是直接进入了回调页（往往是业务页面）
    #[serde(rename = "snsapi_base")]
    Base = 1,
    /// 以snsapi_userinfo为scope发起的网页授权，是用来获取用户的基本信息的。
    /// 但这种授权需要用户手动同意，并且由于用户同意过，所以无须关注，就可在授权后获取该用户的基本信息
    #[serde(rename = "snsapi_userinfo")]
    UserInfo = 2,
}

impl ToString for OauthScope {
    fn to_string(&self) -> String {
        match self {
            OauthScope::Base => "snsapi_base".into(),
            OauthScope::UserInfo => "snsapi_userinfo".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct OauthUserInfo {
    /// 用户的唯一标识
    pub openid: String,
    /// 用户昵称
    pub nickname: String,
    /// 用户的性别，值为1时是男性，值为2时是女性，值为0时是未知
    pub sex: crate::menu::Gender,
    /// 用户个人资料填写的省份
    pub province: String,
    /// 普通用户个人资料填写的城市
    pub city: String,
    /// 国家，如中国为CN
    pub country: String,
    /// 用户头像， /// 最后一个数值代表正方形头像大小（有0、46、64、96、132数值可选，0代表640*640正方形头像），
    /// 用户没有头像时该项为空。若用户更换头像，原有头像URL将失效。
    pub headimgurl: String,
    /// 用户特权信息，json 数组，如微信沃卡用户为（chinaunicom）
    pub privilege: Vec<String>,
    /// 只有在用户将公众号绑定到微信开放平台帐号后，才会出现该字段。
    pub unionid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct OauthAccessToken {
    /// 网页授权接口调用凭证,注意：此access_token与基础支持的access_token不同
    pub access_token: String,
    /// access_token接口调用凭证超时时间，单位（秒）
    pub expires_in: i64,
    /// 用户刷新access_token
    pub refresh_token: String,
    /// 用户唯一标识
    pub openid: String,
    /// 用户授权的作用域，使用逗号（,）分隔
    pub scope: OauthScope,
}

/// 网页授权
#[async_trait]
pub trait Oauth {
    /// 获取授权跳转的url, 随后通过url交互可以获取code,
    async fn get_authorize_url(
        &self,
        context: &SaasContext,
        // 应用授权作用域，
        scope: &OauthScope,
        // 重定向后会带上state参数，开发者可以填写a-zA-Z0-9的参数值，最多128字节
        state: &String,
    ) -> WechatResult<String>;

    /// 通过code换取网页授权access_token
    /// code说明 ： code作为换取access_token的票据，每次用户授权带上的code将不一样，code只能使用一次，5分钟未被使用自动过期。
    ///
    /// 由于公众号的secret和获取到的access_token安全级别都非常高，必须只保存在服务器，不允许传给客户端。
    /// 后续刷新access_token、通过access_token获取用户信息等步骤，也必须从服务器发起。
    async fn get_authorize_access_token(
        &self,
        context: &SaasContext,
        code: &String,
    ) -> WechatResult<OauthAccessToken>;

    /// 刷新access_token（如果需要）
    /// 由于access_token拥有较短的有效期，当access_token超时后，可以使用refresh_token进行刷新，
    /// refresh_token有效期为30天，当refresh_token失效之后，需要用户重新授权。
    async fn refresh_authorize_access_token(
        &self,
        context: &SaasContext,
        refresh_token: &String,
    ) -> WechatResult<OauthAccessToken>;

    /// 检验授权凭证（access_token）是否有效
    async fn verify_authorize_access_token(
        &self,
        context: &SaasContext,
        openid: &String,
        access_token: &String,
    ) -> WechatResult<()>;

    /// 拉取用户信息(需scope为 snsapi_userinfo)
    ///
    /// 用户管理类接口中的“获取用户基本信息接口”，是在用户和公众号产生消息交互或关注后事件推送后，
    /// 才能根据用户OpenID来获取用户基本信息。这个接口，包括其他微信接口，
    /// 都是需要该用户（即openid）关注了公众号后，才能调用成功的。
    async fn get_user_info_by_access_token(
        &self,
        context: &SaasContext,
        openid: &String,
        access_token: &String,
    ) -> WechatResult<OauthUserInfo>;
}

#[async_trait]
impl Oauth for Wechat {
    async fn get_authorize_url(
        &self,
        context: &SaasContext,
        scope: &OauthScope,
        state: &String,
    ) -> WechatResult<String> {
        let config = self.saas_resolver.resolve_config(self, context).await?;

        let mut url = Url::parse("https://open.weixin.qq.com/connect/oauth2/authorize").unwrap();
        {
            let mut query_kv = url.query_pairs_mut();
            query_kv.append_pair("appid", &config.app_id);
            query_kv.append_pair("redirect_uri", &config.oauth_redirect_url);
            query_kv.append_pair("response_type", "code");
            query_kv.append_pair("scope", &scope.to_string());
            query_kv.append_pair("state", state);
        }
        let url = format!("{}#wechat_redirect", url.to_string());
        Ok(url)
    }

    async fn get_authorize_access_token(
        &self,
        context: &SaasContext,
        code: &String,
    ) -> WechatResult<OauthAccessToken> {
        use crate::req_utils::ToApiResult;

        let config = self.saas_resolver.resolve_config(self, context).await?;
        let mut url = Url::parse(
            "https://api.weixin.qq.com/sns/oauth2/access_token?grant_type=authorization_code",
        )
        .unwrap();
        {
            let mut query_kv = url.query_pairs_mut();
            query_kv.append_pair("appid", &config.app_id);
            query_kv.append_pair("secret", &config.app_secret);
            query_kv.append_pair("code", code);
        }
        let result: OauthAccessToken = reqwest::get(url).await?.text().await?.to_api_result()?;
        Ok(result)
    }

    async fn refresh_authorize_access_token(
        &self,
        context: &SaasContext,
        refresh_token: &String,
    ) -> WechatResult<OauthAccessToken> {
        use crate::req_utils::ToApiResult;

        let config = self.saas_resolver.resolve_config(self, context).await?;
        let mut url = Url::parse(
            "https://api.weixin.qq.com/sns/oauth2/refresh_token?grant_type=refresh_token",
        )
        .unwrap();
        {
            let mut query_kv = url.query_pairs_mut();
            query_kv.append_pair("appid", &config.app_id);
            query_kv.append_pair("refresh_token", refresh_token);
        }
        let result: OauthAccessToken = reqwest::get(url).await?.text().await?.to_api_result()?;
        Ok(result)
    }

    async fn verify_authorize_access_token(
        &self,
        context: &SaasContext,
        openid: &String,
        access_token: &String,
    ) -> WechatResult<()> {
        use crate::req_utils::{EmptyApiResult, ToApiResult};

        let mut url = Url::parse("https://api.weixin.qq.com/sns/auth").unwrap();
        {
            let mut query_kv = url.query_pairs_mut();
            query_kv.append_pair("openid", openid);
            query_kv.append_pair("access_token", access_token);
        }
        let _: EmptyApiResult = reqwest::get(url).await?.text().await?.to_api_result()?;
        Ok(())
    }

    async fn get_user_info_by_access_token(
        &self,
        context: &SaasContext,
        openid: &String,
        access_token: &String,
    ) -> WechatResult<OauthUserInfo> {
        use crate::req_utils::{EmptyApiResult, ToApiResult};
        let mut url = Url::parse("https://api.weixin.qq.com/sns/userinfo?lang=zh_CN").unwrap();
        {
            let mut query_kv = url.query_pairs_mut();
            query_kv.append_pair("openid", openid);
            query_kv.append_pair("access_token", access_token);
        }
        let user: OauthUserInfo = reqwest::get(url).await?.text().await?.to_api_result()?;
        Ok(user)
    }
}
