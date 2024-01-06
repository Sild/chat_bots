package main

import (
	"fmt"
	"speaking_from_heart/src/database"
	"strings"

	"github.com/SakoDroid/telego/v2"
	"github.com/SakoDroid/telego/v2/objects"
	"github.com/sild/gosk/log"
)

func addKeyboard(bot *telego.Bot, subs database.Subscriber, db database.DB) telego.MarkUps {
	kb := bot.CreateKeyboard(true, false, false, false, "enter letter text ...")
	text := "/subscribe"
	if db.IsSubscriber(subs) {
		text = "/unsubscribe"
	}
	kb.AddButton(text, 1)
	return kb
}

type addHandlerFunc func(*telego.Bot, database.DB) error

func addStartHandler(bot *telego.Bot, db database.DB) error {
	return bot.AddHandler("/start", func(u *objects.Update) {
		if u.Message == nil || u.Message.Chat == nil || u.Message.Chat.Username == "" {
			return
		}
		subs := database.Subscriber{
			ChatID: u.Message.Chat.Id,
		}

		keyboard := addKeyboard(bot, subs, db)
		text := `
Greetings!

Utilize this bot to send anonymous letters to someone from our subscription list. The bot prioritizes user privacy and retains minimal data, limited to statistics on sent letters and chat_ids of subscribers necessary for message delivery.

Feel free to subscribe to receive letters periodically. The selection of recipients is completely random, employing a straightforward approach without intricate routing.

Whether you wish to subscribe or send a letter (must contain at least 5 words), we hope you'll have some fun!

You can freely examine the source code at the following GitHub repository: https://github.com/Sild/chat_bots/tree/master/tg_speaking_from_heart

Thank you!
		`
		_, err := bot.AdvancedMode().ASendMessage(u.Message.Chat.Id, text, "", u.Message.MessageId, 0, false, false, nil, false, false, keyboard)
		if err != nil {
			fmt.Println(err)
		}
	}, "private")
}

func addSubsHandler(bot *telego.Bot, db database.DB) error {
	return bot.AddHandler("/subscribe", func(u *objects.Update) {
		if u.Message == nil || u.Message.Chat == nil || u.Message.Chat.Username == "" {
			return
		}
		subs := database.Subscriber{
			ChatID: u.Message.Chat.Id,
		}
		db.AddSubscriber(subs)

		keyboard := addKeyboard(bot, subs, db)

		text := fmt.Sprintf("You're subscribed!\nSubscribers count: %d", db.SubsCount())

		_, err := bot.AdvancedMode().ASendMessage(u.Message.Chat.Id, text, "", u.Message.MessageId, 0, false, false, nil, false, false, keyboard)
		if err != nil {
			fmt.Println(err)
		}
	}, "private")
}

func addUnsubsHandler(bot *telego.Bot, db database.DB) error {
	return bot.AddHandler("/unsubscribe", func(u *objects.Update) {
		if u.Message == nil || u.Message.Chat == nil || u.Message.Chat.Username == "" {
			return
		}
		subs := database.Subscriber{
			ChatID: u.Message.Chat.Id,
		}
		db.RemoveSubscriber(subs)

		keyboard := addKeyboard(bot, subs, db)

		text := fmt.Sprintf("You're unsubscribed =(\nSubscribers count: %d", db.SubsCount())

		_, err := bot.AdvancedMode().ASendMessage(u.Message.Chat.Id, text, "", u.Message.MessageId, 0, false, false, nil, false, false, keyboard)
		if err != nil {
			fmt.Println(err)
		}
	}, "private")
}

func handleUpdate(bot *telego.Bot, update *objects.Update, db database.DB) error {
	if update.Message == nil {
		log.Debug("update.Message is nil")
		return nil
	}
	letter := update.Message.Text
	if len(strings.Split(letter, " ")) < 5 {
		_, err := bot.SendMessage(update.Message.Chat.Id, "Your message is too short, please add more details (at least 5 words)", "", update.Message.MessageId, false, false)
		return err
	}

	subs, err := db.RandomSubscriber()
	if err != nil {
		_, err := bot.SendMessage(update.Message.Chat.Id, "No subscribes =(\nPlease try again later", "", update.Message.MessageId, false, false)
		return err

	}
	for subs.ChatID == update.Message.Chat.Id {
		if db.SubsCount() == 1 {
			_, err := bot.SendMessage(update.Message.Chat.Id, "There are only one subscriber, and it's you =(\nSo your message was not sent", "", update.Message.MessageId, false, false)
			return err
		}
		subs, _ = db.RandomSubscriber()
	}
	_, err = bot.SendMessage(subs.ChatID, "Your message was sent to somebody...", "", update.Message.MessageId, false, false)
	if err != nil {
		_, err2 := bot.SendMessage(update.Message.Chat.Id, "Some error happened while we're sending a letter\nPlease try again later", "", update.Message.MessageId, false, false)
		if err2 != nil {
			log.Error("Could not send error notification to user. Reason : " + err2.Error())
		}
		return err
	}
	return nil
}
