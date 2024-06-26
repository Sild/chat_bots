use crate::handler::{DefaultHelpHandler, MessageHandler, MessageHandlerPtr};
use crate::state::{HandlerContext, Context};
use anyhow::Result;
use std::ops::Deref;

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

// async fn interactions_dispatcher(
//     event: SlackInteractionEvent,
//     _client: Arc<SlackHyperClient>,
//     _states: SlackClientEventsUserState,
// ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     log::trace!("got new interaction event: {:#?}", event);
//     Ok(())
// }

// async fn commands_dispatcher(
//     event: SlackCommandEvent,
//     client: Arc<SlackHyperClient>,
//     _states: SlackClientEventsUserState,
// ) -> std::result::Result<SlackCommandEventResponse, Box<dyn std::error::Error + Send + Sync>> {
//     log::trace!("got new command: {:#?}", event);
//
//     let token_value: SlackApiTokenValue = config_env_var("SLACK_TEST_TOKEN")?.into();
//     let token = SlackApiToken::new(token_value);
//
//     // Sessions are lightweight and basically just a reference to client and token
//     let session = client.open_session(&token);
//
//     session.api_test(&SlackApiTestRequest::new().with_foo("Test".into())).await?;
//
//     let user_info_resp = session.users_info(&SlackApiUsersInfoRequest::new(event.user_id.clone())).await?;
//
//     println!("{:#?}", user_info_resp);
//
//     Ok(SlackCommandEventResponse::new(SlackMessageContent::new().with_text(
//         format!("Working on it: {:#?}", user_info_resp.user.team_id),
//     )))
// }

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
    let context_lock = state.read().await;
    let context = context_lock.get_user_state::<Context>().unwrap();

    let mut msg_body = message.content.as_ref().unwrap().text.as_ref().unwrap();

    // ignore non-bot messages
    // TODO implement free_reply handler for the other cases
    if !msg_body.starts_with(context.bot_marker.as_str()) {
        log::trace!("event was ignored as non-related to the bot");
        return Ok(());
    }

    let msg_body = msg_body.strip_prefix(context.bot_marker.as_str()).unwrap().trim();

    let channel_id = message.origin.channel.as_ref().unwrap();

    let handler_name = msg_body.split(' ').next().unwrap_or("help");
    let slack_helper = context.slack_helper.read().await;
    match context.get_message_handler(channel_id, handler_name) {
        Some(handler) => {
            handler
                .read()
                .await
                .handle(
                    context.into(),
                    slack_helper.deref(),
                    message.clone(),
                    vec![handler_name.into()],
                )
                .await?;
            return Ok(());
        }
        None => {
            context
                .default_help_handler
                .handle(
                    context.into(),
                    slack_helper.deref(),
                    message.clone(),
                    vec![handler_name.into()],
                )
                .await?;
        }
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

fn build_socket_listener(state: Context) -> Result<SlackClientSocketModeListener<SlackClientHyperHttpsConnector>> {
    let socket_mode_callbacks = SlackSocketModeListenerCallbacks::new()
        // .with_command_events(commands_dispatcher)
        // .with_interaction_events(interactions_dispatcher)
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

pub async fn run(oauth_token: &str, socket_token: &str, message_handlers: Vec<MessageHandlerPtr>) -> Result<()> {
    if log::max_level() >= log::Level::Debug {
        let subscriber = tracing_subscriber::fmt().with_env_filter("slack_morphism=debug").finish();
        tracing::subscriber::set_global_default(subscriber)?;
    }

    let mut state = Context::new(oauth_token).await?;
    state.add_handlers(message_handlers).await?;

    let socket_listener = build_socket_listener(state)?;
    socket_listener.listen_for(&SlackApiToken::new(socket_token.into())).await?;

    socket_listener.serve().await;
    Ok(())
}
