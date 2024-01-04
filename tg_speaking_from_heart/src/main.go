package main

import (
	"fmt"
	"log"

	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api/v5"
)

func dumpDB(db *DB) {
	json, err := db.AsJson()
	if err != nil {
		log.Fatal(err)
	}
	log.Println(json)

}



func main() {
	bot, err := tgbotapi.NewBotAPI("")
	if err != nil {
		log.Panic(err)
	}

	backupChatId := int64(171246434)

	bot.Send(tgbotapi.NewMessage(backupChatId, "Bot started"))

	bot.Debug = true

	log.Printf("Authorized on account %s", bot.Self.UserName)

	u := tgbotapi.NewUpdate(0)
	u.Timeout = 60

	updates := bot.GetUpdatesChan(u)

	for update := range updates {
		if update.Message != nil { // If we got a message
			fmt.Println(update)
			log.Printf("[%s] %s", update.Message.From.UserName, update.Message.Text)

			msg := tgbotapi.NewMessage(update.Message.Chat.ID, update.Message.Text)
			msg.ReplyToMessageID = update.Message.MessageID

			if _, err := bot.Send(msg); err != nil {
				log.Println("Fail to send message {} to chat {}", msg, update.Message.Chat.ID)
			}
		}
	}
}
