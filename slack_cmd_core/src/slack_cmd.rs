use crate::config_env_var;
use crate::handler::Handler;
use crate::state::State;
use anyhow::Result;

use slack_morphism::prelude::{
    HttpStatusCode, SlackApiTestRequest, SlackApiUsersInfoRequest, SlackClientEventsListenerEnvironment,
    SlackClientEventsUserState, SlackClientHyperConnector, SlackClientHyperHttpsConnector, SlackCommandEvent,
    SlackCommandEventResponse, SlackEventCallbackBody, SlackHyperClient, SlackInteractionEvent, SlackPushEventCallback,
};
use slack_morphism::{
    SlackApiToken, SlackApiTokenValue, SlackClient, SlackClientSocketModeConfig, SlackClientSocketModeListener,
    SlackMessageContent, SlackSocketModeListenerCallbacks,
};
use std::sync::Arc;
use tokio::sync::RwLock;

// inspired by https://github.com/abdolence/slack-morphism-rust/blob/master/examples/socket_mode.rs

async fn interactions_dispatcher(
    event: SlackInteractionEvent,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::trace!("got new interaction event: {:#?}", event);
    Ok(())
}

async fn commands_dispatcher(
    event: SlackCommandEvent,
    client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> std::result::Result<SlackCommandEventResponse, Box<dyn std::error::Error + Send + Sync>> {
    log::trace!("got new command: {:#?}", event);

    let token_value: SlackApiTokenValue = config_env_var("SLACK_TEST_TOKEN")?.into();
    let token = SlackApiToken::new(token_value);

    // Sessions are lightweight and basically just a reference to client and token
    let session = client.open_session(&token);

    session.api_test(&SlackApiTestRequest::new().with_foo("Test".into())).await?;

    let user_info_resp = session.users_info(&SlackApiUsersInfoRequest::new(event.user_id.clone())).await?;

    println!("{:#?}", user_info_resp);

    Ok(SlackCommandEventResponse::new(SlackMessageContent::new().with_text(
        format!("Working on it: {:#?}", user_info_resp.user.team_id),
    )))
}

async fn push_events_dispatcher(
    event: SlackPushEventCallback,
    _client: Arc<SlackHyperClient>,
    state: SlackClientEventsUserState,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::trace!("got new push event: {:#?}", &event);

    // process only messages here
    let message = match event.event {
        SlackEventCallbackBody::Message(ref m) => {
            if m.subtype.is_some() {
                log::trace!("event was ignored as non-msg by subtype");
                return Ok(());
            }
            m
        }
        _ => {
            log::trace!("event was ignored as non-msg type");
            return Ok(());
        }
    };
    let state_lock = state.read().await;
    let state = state_lock.get_user_state::<State>().unwrap();

    let msg_body = message.content.as_ref().unwrap().text.as_ref().unwrap();

    // ignore non-bot messages
    // TODO implement free_reply handler for the other cases
    if !msg_body.starts_with(format!("<@{}>", state.bot_info.user_id).as_str()) {
        log::trace!("event was ignored as non-related to the bot");
        return Ok(());
    }

    let channel_id = message.origin.channel.as_ref().unwrap();
    let thread_ts = message.origin.thread_ts.as_ref().unwrap_or(&message.origin.ts);
    if msg_body.contains("reply") {
        state
            .slack_helper
            .read()
            .await
            .send_reply(channel_id, thread_ts, "reply for Hey there! I'm a bot! Try `!test`")
            .await?;
    } else {
        state
            .slack_helper
            .read()
            .await
            .send_msg(channel_id, "reply for Hey there! I'm a bot! Try `!test`")
            .await?;
    }
    Ok(())
}

fn error_handler(
    err: Box<dyn std::error::Error + Send + Sync>,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> HttpStatusCode {
    println!("{:#?}", err);
    HttpStatusCode::OK
}

fn build_socket_listener(state: State) -> Result<SlackClientSocketModeListener<SlackClientHyperHttpsConnector>> {
    let socket_mode_callbacks = SlackSocketModeListenerCallbacks::new()
        .with_command_events(commands_dispatcher)
        .with_interaction_events(interactions_dispatcher)
        .with_push_events(push_events_dispatcher);

    let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()?));
    let listener_environment = Arc::new(
        SlackClientEventsListenerEnvironment::new(client.clone())
            .with_error_handler(error_handler)
            .with_user_state(state),
    );

    Ok(SlackClientSocketModeListener::new(
        &SlackClientSocketModeConfig::new(),
        listener_environment.clone(),
        socket_mode_callbacks,
    ))
}

pub async fn run(socket_token: &str, state: State) -> Result<()> {
    if log::max_level() >= log::Level::Debug {
        let subscriber = tracing_subscriber::fmt().with_env_filter("slack_morphism=debug").finish();
        tracing::subscriber::set_global_default(subscriber)?;
    }

    let socket_listener = build_socket_listener(state)?;
    socket_listener.listen_for(&SlackApiToken::new(socket_token.into())).await?;

    socket_listener.serve().await;
    Ok(())
}
