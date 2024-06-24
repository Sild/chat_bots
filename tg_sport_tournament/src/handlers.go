package main

import (
	"fmt"
	"speaking_from_heart/src/database"

	"github.com/SakoDroid/telego/v2"
	"github.com/SakoDroid/telego/v2/objects"
	"github.com/sild/gosk/log"
)

func addKeyboard(bot *telego.Bot, db database.DB) telego.MarkUps {
	kb := bot.CreateKeyboard(true, true, true, false, "prizes are waiting for you ...")
	for k := range db.GetActivities() {
		kb.AddButton(k, 1)
	}
	return kb
}

type addHandlerFunc func(*telego.Bot, database.DB, *AppState) error

func addStartHandler(bot *telego.Bot, db database.DB, state *AppState) error {
	return bot.AddHandler("/start", func(update *objects.Update) {
		if update.Message == nil || update.Message.Chat == nil || update.Message.Chat.Username == "" {
			return
		}
		keyboard := addKeyboard(bot, db)
		text := `Привет, пес! Что сделал сегодня? Выбери активность а затем напиши отчет.`
		_, err := bot.AdvancedMode().ASendMessage(update.Message.Chat.Id, text, "", update.Message.MessageId, 0, false, false, nil, false, false, keyboard)
		if err != nil {
			fmt.Println(err)
		}
	}, "private")
}

func addShowTeamsHandler(bot *telego.Bot, db database.DB, state *AppState) error {
	return bot.AddHandler("/show_teams", func(update *objects.Update) {
		if update.Message == nil || update.Message.Chat == nil || update.Message.Chat.Username == "" {
			return
		}
		text := ""
		for tName, team := range db.GetTeams() {
			members := ""
			for _, user := range team.UserTgIDs {
				members += "@" + user + "\n"
			}
			text += fmt.Sprintf("Team: %s\nMembers:\n%s\n", tName, members)
		}
		keyboard := addKeyboard(bot, db)
		_, err := bot.AdvancedMode().ASendMessage(update.Message.Chat.Id, text, "", update.Message.MessageId, 0, false, false, nil, false, false, keyboard)
		if err != nil {
			fmt.Println(err)
		}
	}, "private")
}

func handleUpdate(bot *telego.Bot, update *objects.Update, db database.DB, state *AppState) error {
	if update.Message == nil {
		log.Debug("update.Message is nil")
		return nil
	}
	keyboard := addKeyboard(bot, db)

	reportUser := update.Message.Chat.Username
	if !db.UserExists(reportUser) {
		text := `Ты кто такой, я тебя не знаю. Попроси Настюшку тебя зарегистрировать`
		_, err := bot.AdvancedMode().ASendMessage(update.Message.Chat.Id, text, "", update.Message.MessageId, 0, false, false, nil, false, false, keyboard)
		if err != nil {
			log.Error("Could not send message to user %s. Reason : %v", reportUser, err)
		}
		return nil
	}
	activities := db.GetActivities()
	userMsg := update.Message.Text

	// user pick something from activities list
	if act, found := activities[userMsg]; found {
		state.SetActivity(reportUser, userMsg)
		text := fmt.Sprintf("Отлично, с выбором справился.\n%s", act.Question)
		_, err := bot.AdvancedMode().ASendMessage(update.Message.Chat.Id, text, "", update.Message.MessageId, 0, false, false, nil, false, false, keyboard)
		return err
	}

	// we're here means user sent some random shit
	selectedActivity := state.GetActivity(reportUser)
	if selectedActivity == "" {
		text := "Что ты тут печатаешь, выбери сначала что делал..."
		_, err := bot.AdvancedMode().ASendMessage(update.Message.Chat.Id, text, "", update.Message.MessageId, 0, false, false, nil, false, false, keyboard)
		return err
	}

	if userMsg == "" {
		_, err := bot.AdvancedMode().ASendMessage(update.Message.Chat.Id, "Отчеты котиками только за доп плату, текстом вводи...", "", update.Message.MessageId, 0, false, false, nil, false, false, keyboard)
		return err
	}

	report := database.Report{
		UserTgID:   reportUser,
		ActivityID: 0,
		Value:      userMsg,
		AddTs:      update.Message.Date,
	}
	db.AddReport(reportUser, report)
	state.SetActivity(reportUser, "")
	_, err := bot.AdvancedMode().ASendMessage(update.Message.Chat.Id, "Запомнил!", "", update.Message.MessageId, 0, false, false, nil, false, false, keyboard)
	return err
}
