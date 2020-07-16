cmake_minimum_required(VERSION 2.6)
project(telegram_bot)

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++11 -Werror")
set(Boost_USE_MULTITHREADED ON)
#if(NOT ${OPENSSL_ROOT_DIR})
    set(OPENSSL_ROOT_DIR /usr/local/Cellar/openssl/1.0.2k/)
#endif(NOT ${OPENSSL_ROOT_DIR})

find_package(Threads REQUIRED)
find_package(OpenSSL REQUIRED)
find_package(Boost COMPONENTS system iostreams REQUIRED)

include_directories(
        ${Boost_INCLUDE_DIR}
        ${OPENSSL_INCLUDE_DIR}
        includes/tgbot-cpp/
        includes/tgbot-cpp/include/
        .
)

add_subdirectory(includes/tgbot-cpp)

add_library(bot_wrapper src/bot/Bot.hpp src/bot/Bot.cpp)

target_compile_options(bot_wrapper PUBLIC -Wno-error=cpp)
target_compile_options(TgBot PUBLIC -Wno-error=cpp -Wno-error=address -Wno-error=nonnull-compare)

add_library(imageHandler
        src/handlers/Handler.hpp
        src/handlers/image/ImageHandler.hpp
        src/handlers/image/ImageHandler.cpp)

set(LIBRARIES
        ${Boost_LIBRARIES}
        bot_wrapper
        TgBot
        ${CMAKE_THREAD_LIBS_INIT}
        ${OPENSSL_LIBRARIES}
        imageHandler
        )


add_executable(assistant src/main.cpp src/handlers/Handler.hpp src/bot/Bot.hpp)

target_link_libraries(assistant ${LIBRARIES})
