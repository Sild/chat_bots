package main

import (
	"fmt"
	"speaking_from_heart/src/database"

	"github.com/sild/gosk/log"

	"github.com/SakoDroid/telego/v2"
	"github.com/SakoDroid/telego/v2/configs"
)

func createBot(tgToken string, db database.DB) (*telego.Bot, error) {
	bot, err := telego.NewBot(configs.Default(tgToken))
	if err != nil {
		return nil, err
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

	return bot, nil
}

func createDB(systemChatID int, dbPath string) (database.DB, error) {
	db := database.NewJsonDB(dbPath)
	defaultSubscriber := database.Subscriber{
		ChatID: systemChatID,
	}
	db.AddSubscriber(defaultSubscriber)
	return db, nil
}

func runUpdatesLoop(bot *telego.Bot, db database.DB, conf *Config) {
	text := fmt.Sprintf("Bot started\nSubscribers count: %d\nMsgSent count: %d", db.SubsCount(), db.MsgSentCount())
	if _, err := bot.SendMessage(conf.SystemChannelID, text, "", 0, false, false); err != nil {
		log.Fatal("Could not send message to system chat. Reason : " + err.Error())
	}

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

func main() {
	conf, err := NewConfigFromEnv()
	if err != nil {
		log.Fatal("Could not create config. Reason : " + err.Error())
	}

	db, err := createDB(conf.SystemChannelID, conf.DBPath)
	if err != nil {
		log.Fatal("Could not create DB. Reason : " + err.Error())
	}

	bot, err := createBot(conf.TgToken, db)
	if err != nil {
		log.Fatal("Could not create bot. Reason : " + err.Error())
	}

	if err := bot.Run(false); err != nil {
		log.Fatal("Could not run the bot. Reason : " + err.Error())
	}

	runUpdatesLoop(bot, db, conf)
}
