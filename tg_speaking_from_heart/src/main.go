package main

import (
	"fmt"
	"os"
	"speaking_from_heart/src/database"
	"time"

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

func dbBackupLoop(bot *telego.Bot, conf *Config) {
	lastBackupTS := int64(0)
	lastContent := ""
	for {
		time.Sleep(time.Second * 1)
		curTS := time.Now().Unix()
		if curTS-lastBackupTS < 60*60*24 {
			continue
		}

		dbData, err := os.ReadFile(conf.DBPath)
		if err != nil {
			log.Error("Fail to hash backup. Reason : " + err.Error())
			continue
		}
		if lastContent == string(dbData) {
			log.Debug("Skipping db backup: data didn't change")
			lastBackupTS = curTS
			continue
		}

		mediaSender := bot.SendDocument(conf.SystemChannelID, 0, "db.json", "")
		file, err := os.Open(conf.DBPath)
		if err != nil {
			_, _ = bot.SendMessage(conf.SystemChannelID, "Could not open DB file to make backup. Reason : "+err.Error(), "", 0, false, false)
			log.Error("Could not open DB file to make backup. Reason : " + err.Error())
			continue
		}
		_, err = mediaSender.SendByFile(file, false, false)
		if err != nil {
			_, _ = bot.SendMessage(conf.SystemChannelID, "Fail to send db backup. Reason : "+err.Error(), "", 0, false, false)
			log.Error("Fail to send db backup. Reason : " + err.Error())
			continue
		}
		lastBackupTS = curTS
		lastContent = string(dbData)
	}
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

	go dbBackupLoop(bot, conf)

	if err := bot.Run(false); err != nil {
		log.Fatal("Could not run the bot. Reason : " + err.Error())
	}

	runUpdatesLoop(bot, db, conf)
}
