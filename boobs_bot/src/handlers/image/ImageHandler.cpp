#include <numeric>
#include "ImageHandler.hpp"


ImageHandler::ImageHandler() {
    srand( time(0) );
    std::vector<std::string> topics = {"boobs"};
    readConfig(topics);
}

std::string ImageHandler::prepareResponse(const std::string &request) {
    for(const auto &cnf: config) {
        if(request.find(cnf.first) != std::string::npos) {
            std::string link = cnf.second[0].link;
            int imgId = cnf.second[0].min + (rand() % (cnf.second[0].max - cnf.second[0].min + 1));
            link.replace(link.find("{}"), 2, std::to_string(imgId));
            return link;
        }
    }
    return "No topics found. Available topics:\n" +
    std::accumulate(config.begin(), config.end(), std::string(),
    [](const std::string &a, const std::map<std::string, std::vector<ImageConfig>>::value_type &e) -> std::string{ return a + "\n" + e.first;});
}

const std::string ImageHandler::getTrigger() const {
    return trigger;
}

void ImageHandler::readConfig(const std::vector<std::string> &topics) {
    std::string link;
    int min, max;
    for(auto &it: topics) {
        ImageConfig config = {};
        std::ifstream input(std::string(CONFIG_DIR) + std::string("/") + it + ".conf");
        if(!input.is_open()) {
            throw std::ios::failure("Fail to open config file");
        }
        for(std::string line; getline(input, line);) {
            if(!line.substr(0, 5).compare("link:")) {
                link = line.substr(6, line.length() - 1);
                config.link = link;
            }
            if(!line.substr(0, 4).compare("min:")) {
                min = std::stoi(line.substr(5, line.length() - 1));
                config.min = min;
            }
            if(!line.substr(0, 4).compare("max:")) {
                max = std::stoi(line.substr(5, line.length() - 1));
                config.max = max;
            }
        }
        this->config[it].push_back(config);
    }
}
