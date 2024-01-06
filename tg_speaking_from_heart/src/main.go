package main

import (
	"fmt"
	"os"
	"speaking_from_heart/src/database"
	"strconv"

	"github.com/sild/gosk/log"

	"github.com/SakoDroid/telego/v2"
	"github.com/SakoDroid/telego/v2/configs"
)

func createBot() (*telego.Bot, error) {
	token := os.Getenv("SPEAKING_FROM_HEART_TOKEN")

	bot, err := telego.NewBot(configs.Default(token))
	if err != nil {
		return nil, err
	}
	return bot, nil
}

func createDB(bot *telego.Bot, systemChatID int) (database.DB, error) {
	db := database.NewJsonDB("")
	defaultSubscriber := database.Subscriber{
		ChatID: systemChatID,
	}
	db.AddSubscriber(defaultSubscriber)
	return db, nil
}

func main() {
	bot, err := createBot()
	if err != nil {
		log.Fatal("Could not create bot. Reason : " + err.Error())
	}

	systemChatID64, err := strconv.ParseInt(os.Getenv("SPEAKING_FROM_HEART_PRIVATE"), 10, 64)
	if err != nil {
		fmt.Println(err)
		return
	}
	systemChatID := int(systemChatID64)

	db, err := createDB(bot, systemChatID)
	if err != nil {
		log.Fatal("Could not create DB. Reason : " + err.Error())
	}

	for _, handler := range []addHandlerFunc{
		addStartHandler,
		addSubsHandler,
		addUnsubsHandler,
	} {
		if err = handler(bot, db); err != nil {
			log.Fatal("Could not add handler. Reason : " + err.Error())
		}
	}

	if err := bot.Run(false); err != nil {
		log.Fatal("Could not run the bot. Reason : " + err.Error())
	}

	text := fmt.Sprintf("Bot started\nSubscribers count: %d\nMsgSent count: %d", db.SubsCount(), db.MsgSentCount())
	if _, err = bot.SendMessage(systemChatID, text, "", 0, false, false); err != nil {
		log.Fatal("Could not send message to system chat. Reason : " + err.Error())
	}

	start(bot, db)
}

func start(bot *telego.Bot, db database.DB) {
	updateChannel, err := bot.AdvancedMode().RegisterChannel("", "message")
	if err != nil {
		fmt.Println(err)
		return
	}

	for {
		update := <-*updateChannel
		if err := handleUpdate(bot, update, db); err != nil {
			log.Error("Could not handle update. Reason : " + err.Error())
		}
	}
}
