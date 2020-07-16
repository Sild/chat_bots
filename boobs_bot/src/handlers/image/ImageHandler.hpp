#ifndef TELEGRAM_BOT_IMAGEHANDLER_H
#define TELEGRAM_BOT_IMAGEHANDLER_H

#include "../Handler.hpp"
#include <string>
#include <map>
#include <queue>
#include <iostream>
#include <fstream>
#include <ctime>
#include <algorithm>

struct ImageConfig {
    std::string link;
    int min;
    int max;
};

class ImageHandler: public Handler {
private:
    const char* CONFIG_DIR = "../src/handlers/image";
    //TODO async preparing for links required
    std::map<std::string, std::queue<std::string> > linkPool;
    std::map<std::string, std::vector<ImageConfig> > config;
    void readConfig(const std::vector<std::string> &topics);

public:
    ImageHandler();
    const char* trigger = "image";
    std::string prepareResponse(const std::string &request);
    const std::string getTrigger() const;
};

#endif
