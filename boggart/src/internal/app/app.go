package app

import (
	"context"
	"fmt"
	"sort"
	"strings"
	"sync"

	"internal/db"

	tgapi "github.com/go-telegram-bot-api/telegram-bot-api/v5"
	log "github.com/sirupsen/logrus"
)

type App struct {
	tgCli        *TgCli
	dbCli        *db.DBCli
	handlers     map[string]func(msg *tgapi.Message)
	reportChatID int64
	wg           sync.WaitGroup
}

func NewApp(tgToken, dbFolderPath string, reportChatID int64) (*App, error) {
	log.Printf("App: creating...")
	defer log.Printf("App: created")
	tgCli, err := NewTgCli(tgToken)
	if err != nil {
		return nil, err
	}
	dbCli, err := db.NewDBCli(dbFolderPath)
	if err != nil {
		return nil, err
	}

	app := &App{
		tgCli:        tgCli,
		dbCli:        dbCli,
		handlers:     map[string]func(msg *tgapi.Message){},
		reportChatID: reportChatID,
		wg:           sync.WaitGroup{},
	}

	app.handlers["/help"] = app.handleHelp
	app.handlers["/ask"] = app.handleHelp
	app.handlers["/register"] = app.handleHelp

	app.handlers["/add"] = app.handleHelp
	app.handlers["/rm"] = app.handleHelp

	app.handlers["/list"] = app.handleList

	return app, nil
}

func (app *App) Run(ctx context.Context) {

	app.wg.Add(1)
	go app.tgHandler(ctx)

	app.wg.Wait()
}

func (app *App) tgHandler(ctx context.Context) {
	log.Info("tgHandler: started")
	defer log.Info("tgHandler: done")

	updates := app.tgCli.GetUpdatesChan()
	for {
		select {
		case <-ctx.Done():
			log.Info("tgHandler: done by context")
			app.wg.Done()
			return
		case newUpdate := <-updates:
			go app.handleTgMessage(newUpdate.Message)
		}
	}
}

func (app *App) handleTgMessage(msg *tgapi.Message) {
	if msg == nil {
		return
	}
	log.Trace("got new message: '", msg.Text, "' from user: '", msg.From.UserName, "'")
	data := strings.Split(msg.Text, " ")
	if len(data) < 1 {
		log.Warn("got message without command: '", msg.Text, "'")
	}
	if handler, found := app.handlers[data[0]]; found {
		handler(msg)
	} else {
		log.Warn(fmt.Sprintf("got message='%s' without unknown command='%s'", msg.Text, data[0]))
		app.handleHelp(msg)
	}
}

func (app *App) handleList(msg *tgapi.Message) {
	// subscribers, err := app.dbCli.GetSubscribers()
	// if err != nil {
	// 	_ = app.tgCli.Send(msg.Chat.ID, "error happened, please try later")
	// 	return
	// }
	// chatSubs := []string{}
	// for _, subs := range subscribers {
	// 	if subs.ChatID == msg.Chat.ID {
	// 		chatSubs = append(chatSubs, fmt.Sprintf("%s (%s)", subs.Pattern, subs.Status))
	// 	}
	// }
	// log.Info("New list call for chat: ", msg.Chat.ID, " with ", len(chatSubs), " subscriptions")
	// response := fmt.Sprintf("активные подписки (%d):\n", len(chatSubs)) + strings.Join(chatSubs, "\n")
	// err = app.tgCli.Send(msg.Chat.ID, response)
	// if err != nil {
	// 	log.Error("handleDefault: fail to send reply to telegram: ", err)
	// 	return
	// }
}

func (app *App) handleSubscribe(msg *tgapi.Message) {
	// pattern := strings.Join(strings.Split(msg.Text, " ")[1:], " ")
	// if pattern == "" {
	// 	return
	// }
	// subscribers, err := app.dbCli.GetSubscribers()
	// if err != nil {
	// 	_ = app.tgCli.Send(msg.Chat.ID, "error happened, please try later")
	// 	return
	// }

	// for _, subs := range subscribers {
	// 	if subs.ChatID == msg.Chat.ID && subs.Pattern == pattern {
	// 		_ = app.tgCli.Send(msg.Chat.ID, "already subscribed")
	// 		return
	// 	}
	// }
	// // newSubs := &Subscribe{ChatID: msg.Chat.ID, Pattern: pattern, Status: UNDEFINED}
	// // subscribers = append(subscribers, newSubs)
	// err = app.dbCli.RewriteSubscribers(subscribers)
	// if err != nil {
	// 	_ = app.tgCli.Send(msg.Chat.ID, "error happened, please try later")
	// 	return
	// }
	// // log.Info("New subscription: ", *newSubs)
	// if err = app.tgCli.Send(msg.Chat.ID, fmt.Sprintf("subscribed (total count: %d)", len(subscribers))); err != nil {
	// 	log.Error("Fail to send msg to telegram: ", err)
	// }
}

func (app *App) handleHelp(msg *tgapi.Message) {
	handlers := []string{}
	for key, _ := range app.handlers {
		handlers = append(handlers, key)
	}
	sort.SliceStable(handlers, func(i, j int) bool {
		return handlers[i] < handlers[j]
	})
	handlersString := strings.Join(handlers, "\n")
	_ = app.tgCli.Send(msg.Chat.ID, fmt.Sprintf("доступные команды:\n%s", handlersString))
}
