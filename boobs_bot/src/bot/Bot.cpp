#include "Bot.hpp"

Bot::Bot(const std::string &botToken) {
    bot = new TgBot::Bot(botToken);
}

void Bot::addHandler(Handler &handler) {
    const std::string HANDLE_TRIGGER = handler.getTrigger();
    checkHandlerBusy(HANDLE_TRIGGER);
    triggers.push_back(HANDLE_TRIGGER);
    bot->getEvents().onCommand(HANDLE_TRIGGER, [this, &handler](TgBot::Message::Ptr message) {
        this->bot->getApi().sendMessage(message->chat->id, handler.prepareResponse(message->text));
    });
}

void Bot::checkHandlerBusy(const std::string &handler) const {
    if (std::find(triggers.begin(), triggers.end(), handler) != triggers.end()) {
        throw HandlerAlreadyBindException();
    }
}

void Bot::start() {
    addHelpHandler();
    resetOldEvents();
    startCyclePooling();
}

void Bot::addHelpHandler() {
    const std::string HELP_TRIGGER = "help";
    triggers.push_back(HELP_TRIGGER);

    std::string availableTriggers = "Available commands:\n";
    for (std::string it: triggers) {
        availableTriggers += "/" + it + "\n";
    }

    bot->getEvents().onCommand(HELP_TRIGGER, [this, availableTriggers](TgBot::Message::Ptr message) {
        this->bot->getApi().sendMessage(message->chat->id, availableTriggers);
    });
}

void Bot::resetOldEvents() {
    bot->getApi().getUpdates();
}

void Bot::startCyclePooling() {
    try {
        printf("Bot started: %s\n", bot->getApi().getMe()->username.c_str());

        TgBot::TgLongPoll longPoll(*bot);
        while (true) {
            printf("Long poll started\n");
            longPoll.start();
        }
    } catch (std::exception &e) {
        printf("error: %s\n", e.what());
    }
}

