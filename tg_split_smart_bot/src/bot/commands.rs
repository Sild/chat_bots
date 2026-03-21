use teloxide::utils::command::BotCommands;

#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
    #[command(description = "Open the SplitSmart mini app")]
    Start,
    #[command(description = "Post the current trip report")]
    Report,
    #[command(description = "Reset the current trip session")]
    Reset,
}
