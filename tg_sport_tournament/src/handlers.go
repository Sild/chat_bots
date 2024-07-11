package main

import (
	"fmt"
	"speaking_from_heart/src/database"

	"github.com/SakoDroid/telego/v2"
	"github.com/SakoDroid/telego/v2/objects"
	"github.com/sild/gosk/log"
)

func addKeyboard(bot *telego.Bot, db database.DB, state *AppState, username string) telego.MarkUps {
	selectedActivity := state.GetActivity(username)
	text := "Выбери активность ..."
	if selectedActivity != "" {
		text = fmt.Sprintf("Ты выбрал '%s', теперь напиши отчет ...", selectedActivity)
	}
	kb := bot.CreateKeyboard(true, true, true, false, text)
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
		text := `Привет, пес! Что сделал сегодня? Выбери активность, а затем напиши отчет.`
		err := sendMessage(bot, update, db, state, text)
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
		
		err := sendMessage(bot, update, db, state, text)
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

	reportUser := update.Message.Chat.Username
	if !db.UserExists(reportUser) {
		text := `Ты кто такой, я тебя не знаю. Попроси Настюшку тебя зарегистрировать`
		return sendMessage(bot, update, db, state, text)
	}
	activities := db.GetActivities()
	userMsg := update.Message.Text

	// user pick something from activities list
	if act, found := activities[userMsg]; found {
		state.SetActivity(reportUser, userMsg)
		text := fmt.Sprintf("Отлично, с выбором справился.\n%s", act.Question)
		return sendMessage(bot, update, db, state, text)
	}

	// we're here means user sent some random shit
	selectedActivity := state.GetActivity(reportUser)
	if selectedActivity == "" {
		return sendMessage(bot, update, db, state,  "Что ты тут печатаешь, выбери сначала что делал...")
	}

	if userMsg == "" {
		return sendMessage(bot, update, db, state,  "Отчеты котиками только за доп плату, текстом вводи...")
	}

	if selectedActivity == "драка с колбасой" {
		return sendMessage(bot, update, db, state,  "Выбери что-нибудь другое, а?")
	}

	report := database.Report{
		UserTgID:   reportUser,
		ActivityID: 0,
		Value:      userMsg,
		AddTs:      update.Message.Date,
	}
	db.AddReport(reportUser, report)
	state.SetActivity(reportUser, "")
	return sendMessage(bot, update, db, state, "Запомнил!")
}

func sendMessage(bot *telego.Bot, update *objects.Update, db database.DB, state *AppState, text string) error {
	keyboard := addKeyboard(bot, db, state, update.Message.Chat.Username)
	_, err := bot.AdvancedMode().ASendMessage(update.Message.Chat.Id, text, "", update.Message.MessageId, 0, false, false, nil, false, false, keyboard)
	return err
}
