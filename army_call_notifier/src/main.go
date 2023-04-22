package main

import (
	"context"
	"os"
	"os/signal"
	"strconv"

	log "github.com/sirupsen/logrus"
)

func main() {
	log.Warn("main: started")
	defer log.Warn("main: finished")

	tgToken := os.Getenv("ARMY_CALL_NOTIFIER_TG_TOKEN")
	dbFolderPath := os.Getenv("ARMY_CALL_NOTIFIER_DB_FOLDER")
	reportChatID, err := strconv.Atoi(os.Getenv("ARMY_CALL_NOTIFIER_CHAT_ID"))
	if err != nil {
		log.Fatal(err)
	}
	logLevel, err := strconv.Atoi(os.Getenv("ARMY_CALL_NOTIFIER_LOG_LEVEL"))
	if err != nil {
		log.Fatal(err)
	}

	customFormatter := new(log.TextFormatter)
	customFormatter.TimestampFormat = "2006-01-02 15:04:05"
	log.SetFormatter(customFormatter)
	log.SetLevel(log.Level(logLevel))

	app, err := NewApp(tgToken, dbFolderPath, int64(reportChatID))
	if err != nil {
		log.Fatal(err)
	}

	ctx, cancel := context.WithCancel(context.TODO())

	signalChannel := make(chan os.Signal, 1)
	signal.Notify(signalChannel, os.Interrupt)
	go func() {
		for _ = range signalChannel {
			cancel()
		}
	}()

	app.Run(ctx)
}
