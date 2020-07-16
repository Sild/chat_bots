
#include "src/bot/Bot.hpp"
#include "src/handlers/image/ImageHandler.hpp"

int main() {
    const std::string token = "";
    Bot bot(token);
    ImageHandler imgHandler;
    bot.addHandler(imgHandler);
    bot.start();
    return 0;
}
