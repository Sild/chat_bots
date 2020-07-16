//
// Created by Dmitry Korchagin on 22/02/2017.
//

#ifndef TELEGRAM_BOT_BOT_H
#define TELEGRAM_BOT_BOT_H

#include <stdio.h>
#include <exception>
#include <include/tgbot/tgbot.h>
#include "src/handlers/Handler.hpp"
#include <vector>
#include <iostream>
#include <include/tgbot/Bot.h>


class Bot {
private:
    TgBot::Bot *bot;
    std::vector<std::string> triggers;
    void addHelpHandler();
    void startCyclePooling();
    void resetOldEvents();
    void checkHandlerBusy(const std::string &handler) const;

public:
    Bot(const std::string &botToken);
    void addHandler(Handler &handler);
    void start();

    class HandlerAlreadyBindException {};
};
#endif //TELEGRAM_BOT_BOT_H
