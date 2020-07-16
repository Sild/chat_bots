//
// Created by Dmitry Korchagin on 22/02/2017.
//

#ifndef TELEGRAM_BOT_HANDLER_H
#define TELEGRAM_BOT_HANDLER_H

#include <string>
class Handler {
public:
    virtual std::string prepareResponse(const std::string &request) = 0;
    virtual const std::string getTrigger() const = 0;
};
#endif //TELEGRAM_BOT_HANDLER_H
