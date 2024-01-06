package main

import (
	"fmt"
	"speaking_from_heart/src/db"

	"github.com/SakoDroid/telego/v2"
	"github.com/SakoDroid/telego/v2/objects"
	"github.com/sild/gosk/log"
)

func addKeyboard(bot *telego.Bot, subscribed bool) telego.MarkUps {
	kb := bot.CreateKeyboard(true, false, false, false, "enter letter text ...")
	text := "/subscribe"
	if subscribed {
		text = "/unsubscribe"
	}
	kb.AddButton(text, 1)
	return kb
}

type addHandlerFunc func(*telego.Bot, db.DB) error

func addStartHandler(bot *telego.Bot, db db.DB) error {
	return bot.AddHandler("/start", func(u *objects.Update) {
		if u.Message == nil || u.Message.Chat == nil || u.Message.Chat.Username == "" {
			return
		}
		keyboard := addKeyboard(bot, db.IsSubscriber(u.Message.Chat.Username))
		_, err := bot.AdvancedMode().ASendMessage(u.Message.Chat.Id, "Send a letter, or subscribe to the other letters (more than 5 words)", "", u.Message.MessageId, 0, false, false, nil, false, false, keyboard)
		if err != nil {
			fmt.Println(err)
		}
	}, "private")
}

func addSubsHandler(bot *telego.Bot, db db.DB) error {
	return bot.AddHandler("/subscribe", func(u *objects.Update) {
		if u.Message == nil || u.Message.Chat == nil || u.Message.Chat.Username == "" {
			return
		}
		db.AddSubscriber(u.Message.Chat.Username)

		keyboard := addKeyboard(bot, db.IsSubscriber(u.Message.Chat.Username))
		_, err := bot.AdvancedMode().ASendMessage(u.Message.Chat.Id, "You're subscribed!\nSubscribers count: 10", "", u.Message.MessageId, 0, false, false, nil, false, false, keyboard)
		if err != nil {
			fmt.Println(err)
		}
	}, "private")
}

func addUnsubsHandler(bot *telego.Bot, db db.DB) error {
	return bot.AddHandler("/unsubscribe", func(u *objects.Update) {
		if u.Message == nil || u.Message.Chat == nil || u.Message.Chat.Username == "" {
			return
		}
		db.RemoveSubscriber(u.Message.Chat.Username)

		keyboard := addKeyboard(bot, db.IsSubscriber(u.Message.Chat.Username))
		_, err := bot.AdvancedMode().ASendMessage(u.Message.Chat.Id, "You're unsubscribed =(\nSubscribers count: 9", "", u.Message.MessageId, 0, false, false, nil, false, false, keyboard)
		if err != nil {
			fmt.Println(err)
		}
	}, "private")
}

func handleUpdate(bot *telego.Bot, update *objects.Update) error {
	if update.Message == nil {
		log.Debug("update.Message is nil")
		return nil
	}
	_, err := bot.SendMessage(update.Message.Chat.Id, "Your message was sent to somebody...", "", update.Message.MessageId, false, false)
	return err
}
