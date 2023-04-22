package main

import (
	log "github.com/sirupsen/logrus"

	tgapi "github.com/go-telegram-bot-api/telegram-bot-api/v5"
)

type TgCli struct {
	impl *tgapi.BotAPI
}

func NewTgCli(token string) (*TgCli, error) {
	log.Info("TgCli: creating...")
	defer log.Info("TgCli: created")
	bot, err := tgapi.NewBotAPI(token)
	if err != nil {
		return nil, err
	}
	log.Info("Authorized on account ", bot.Self.UserName)
	// bot.Debug = true
	return &TgCli{impl: bot}, nil
}

func (tgcli *TgCli) Send(chatID int64, text string) error {
	msg := tgapi.NewMessage(chatID, text)
	_, err := tgcli.impl.Send(msg)
	if err != nil {
		log.Error("Fail to send tg message: ", err)
	}
	return err
}

func (tgcli *TgCli) GetUpdatesChan() tgapi.UpdatesChannel {

	updateConf := tgapi.NewUpdate(0)
	updateConf.Timeout = 60

	return tgcli.impl.GetUpdatesChan(updateConf)
}
