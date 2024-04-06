use anyhow::{anyhow, Result};
use slack_morphism::api::{SlackApiChatDeleteRequest, SlackApiChatPostMessageRequest};
use slack_morphism::prelude::{SlackApiBotsInfoRequest, SlackClientHyperConnector, SlackClientHyperHttpsConnector};
use slack_morphism::{
    SlackApiToken, SlackBotInfo, SlackChannelId, SlackClient, SlackClientSession, SlackMessage, SlackMessageContent,
    SlackTs,
};
use tracing::log;

pub struct SlackHelper {
    token: SlackApiToken,
    client: SlackClient<SlackClientHyperHttpsConnector>,
}

impl SlackHelper {
    pub fn new(token: &str) -> Result<Self> {
        let client = SlackClient::new(SlackClientHyperConnector::new()?);
        Ok(Self {
            token: SlackApiToken::new(token.into()),
            client,
        })
    }

    // allow to get raw session for custom workflow
    pub fn get_session(&self) -> SlackClientSession<SlackClientHyperHttpsConnector> {
        self.client.open_session(&self.token)
    }

    // return all messages from thread which contains msg with given msg_ts
    // sorted by ts in ascending order (first message is the oldest)
    pub async fn get_thread(&self, _channel: &SlackChannelId, _msg_ts: &SlackTs) -> Result<Vec<SlackMessage>> {
        Ok(vec![])
    }

    pub async fn get_message(&self, _channel: &SlackChannelId, msg_ts: &SlackTs) -> Result<()> {
        let session = self.client.open_session(&self.token);
        todo!();
    }

    pub async fn delete_msg(&self, channel: &SlackChannelId, msg_ts: &SlackTs) -> Result<()> {
        let req = SlackApiChatDeleteRequest::new(channel.clone(), msg_ts.clone());
        match self.get_session().chat_delete(&req).await {
            Ok(_) => Ok(()),
            Err(err) => {
                log::error!(
                    "Fail to delete msg from channel='{channel}' with ts='{msg_ts}, err='{:?}'",
                    err
                );
                Err(anyhow!(err))
            }
        }
    }

    pub async fn send_msg(&self, channel: &SlackChannelId, msg: &str) -> Result<()> {
        self.send_msg_impl(channel, None, msg).await
    }

    pub async fn send_reply(&self, channel: &SlackChannelId, thread_ts: &SlackTs, msg: &str) -> Result<()> {
        self.send_msg_impl(channel, Some(thread_ts), msg).await
    }

    pub async fn get_bot_info(&self) -> Result<SlackBotInfo> {
        let session = self.get_session();
        let auth_info = session.auth_test().await?;
        let rsp = session
            .bots_info(&SlackApiBotsInfoRequest::new().with_bot(auth_info.bot_id.unwrap().to_string()))
            .await;
        match rsp {
            Ok(rsp) => Ok(rsp.bot),
            Err(err) => {
                log::error!("Fail to get bot info, err='{:?}'", err);
                Err(anyhow!(err))
            }
        }
    }

    async fn send_msg_impl(&self, channel: &SlackChannelId, thread_ts: Option<&SlackTs>, msg: &str) -> Result<()> {
        log::trace!(
            "send_msg_impl: channel_id='{channel}', thread_ts='{:?}', msg='{msg}'",
            thread_ts
        );
        let mut req = SlackApiChatPostMessageRequest::new(
            format!("{}", channel).into(),
            SlackMessageContent::new().with_text(msg.into()),
        );
        if let Some(thread_ts) = thread_ts {
            req = req.with_thread_ts(thread_ts.clone());
        }
        match self.get_session().chat_post_message(&req).await {
            Ok(_) => Ok(()),
            Err(err) => {
                log::error!("Fail to send msg='{msg}' to channel='{channel}', err='{:?}'", err);
                Err(anyhow!(err))
            }
        }
    }
}
