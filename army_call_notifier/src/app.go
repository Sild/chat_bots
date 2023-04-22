package main

import (
	"context"
	"fmt"
	"strings"
	"sync"
	"time"

	tgapi "github.com/go-telegram-bot-api/telegram-bot-api/v5"
	log "github.com/sirupsen/logrus"
)

type App struct {
	tgCli        *TgCli
	parser       *Parser
	dbCli        *DBCli
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
	parser, err := NewParser()
	if err != nil {
		return nil, err
	}
	dbCli, err := NewDBCli(dbFolderPath)
	if err != nil {
		return nil, err
	}
	return &App{
		tgCli:        tgCli,
		parser:       parser,
		dbCli:        dbCli,
		reportChatID: reportChatID,
		wg:           sync.WaitGroup{},
	}, nil
}

func (app *App) Run(ctx context.Context) {

	app.wg.Add(1)
	go app.registryUpdatesLoop(ctx)

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
	if strings.HasPrefix(msg.Text, "/sub") {
		app.handleSubscribe(msg)
	} else if strings.HasPrefix(msg.Text, "/unsub") {
		app.handleUnsubscribe(msg)
	} else if strings.HasPrefix(msg.Text, "/list") {
		app.handleList(msg)
	} else {
		app.handleDefault(msg)
	}
}

func (app *App) handleList(msg *tgapi.Message) {
	subscribers, err := app.dbCli.GetSubscribers()
	if err != nil {
		_ = app.tgCli.Send(msg.Chat.ID, "error happened, please try later")
		return
	}
	chatSubs := []string{}
	for _, subs := range subscribers {
		if subs.ChatID == msg.Chat.ID {
			chatSubs = append(chatSubs, fmt.Sprintf("%s (%s)", subs.Pattern, subs.Status))
		}
	}
	log.Info("New list call for chat: ", msg.Chat.ID, " with ", len(chatSubs), " subscriptions")
	response := fmt.Sprintf("активные подписки (%d):\n", len(chatSubs)) + strings.Join(chatSubs, "\n")
	err = app.tgCli.Send(msg.Chat.ID, response)
	if err != nil {
		log.Error("handleDefault: fail to send reply to telegram: ", err)
		return
	}
}

func (app *App) handleSubscribe(msg *tgapi.Message) {
	pattern := strings.Join(strings.Split(msg.Text, " ")[1:], " ")
	if pattern == "" {
		return
	}
	subscribers, err := app.dbCli.GetSubscribers()
	if err != nil {
		_ = app.tgCli.Send(msg.Chat.ID, "error happened, please try later")
		return
	}

	for _, subs := range subscribers {
		if subs.ChatID == msg.Chat.ID && subs.Pattern == pattern {
			_ = app.tgCli.Send(msg.Chat.ID, "already subscribed")
			return
		}
	}
	newSubs := &Subscribe{ChatID: msg.Chat.ID, Pattern: pattern, Status: UNDEFINED}
	subscribers = append(subscribers, newSubs)
	err = app.dbCli.RewriteSubscribers(subscribers)
	if err != nil {
		_ = app.tgCli.Send(msg.Chat.ID, "error happened, please try later")
		return
	}
	log.Info("New subscription: ", *newSubs)
	if err = app.tgCli.Send(msg.Chat.ID, fmt.Sprintf("subscribed (total count: %d)", len(subscribers))); err != nil {
		log.Error("Fail to send msg to telegram: ", err)
	}
}

func (app *App) handleUnsubscribe(msg *tgapi.Message) {
	pattern := strings.Join(strings.Split(msg.Text, " ")[1:], " ")
	if pattern == "" {
		return
	}

	subscribers, err := app.dbCli.GetSubscribers()
	if err != nil {
		_ = app.tgCli.Send(msg.Chat.ID, "error happened, please try later")
		return
	}

	for pos, s := range subscribers {
		if s.ChatID == msg.Chat.ID && s.Pattern == pattern {
			toRemove := *subscribers[pos]
			subscribers[pos] = subscribers[len(subscribers)-1]
			subscribers = subscribers[:len(subscribers)-1]
			app.dbCli.RewriteSubscribers(subscribers)
			log.Info("New unsubscription: ", toRemove)
			_ = app.tgCli.Send(msg.Chat.ID, fmt.Sprintf("unsubscribed (total count: %d)", len(subscribers)))
			return
		}
	}
	_ = app.tgCli.Send(msg.Chat.ID, "запись не найдена")
}

func (app *App) handleDefault(msg *tgapi.Message) {
	_ = app.tgCli.Send(msg.Chat.ID, "доступные команды:\n/list\n/sub ФИО\n/unsub ФИО")
}

func (app *App) registryUpdatesLoop(ctx context.Context) {
	log.Info("registryUpdatesLoop: started")
	defer log.Info("registryUpdatesLoop: done")

	ticker := time.NewTicker(10 * time.Second)
	for {
		select {
		case <-ctx.Done():
			log.Info("registryUpdatesLoop: done by context")
			app.wg.Done()
			return
		case <-ticker.C:
			startTime := time.Now()
			lastRun, err := app.dbCli.GetLastTime()
			nextRun := lastRun.Add(time.Hour)
			if err != nil || nextRun.Before(startTime) {
				log.Debug("registryUpdatesLoop: check updates at ", startTime)
				defer func() {
					endTime := time.Now()
					log.Debug("registryUpdatesLoop: check updates done at ", endTime, ", duration: ", endTime, endTime.Sub(startTime))
				}()
				app.registryUpdateFunc(ctx)
				app.dbCli.UpdateLastTime(startTime)
			} else {
				log.Trace("registryUpdatesLoop: skipping: last=", lastRun, ", next=", nextRun, ", cur=", startTime)
			}
		}
	}
}

func (app *App) registryUpdateFunc(ctx context.Context) {
	log.Info("registryUpdateFunc: started")
	defer log.Info("registryUpdateFunc: done")

	newData, err := app.parser.GetData()
	for err != nil {
		log.Error(err)
		newData, err = app.parser.GetData()
	}
	oldData, err := app.dbCli.GetRegistry()
	if err != nil {
		log.Error(err)
		return
	}
	log.Trace(newData)
	log.Trace(oldData)
	// read data from reestr
	// something else

	// send msg to channel
	err = app.tgCli.Send(app.reportChatID, "hello")
	if err == nil {
		log.Trace("msg sent")
	}

	// cache data just in case
	// app.dbCli.RewriteData(data)

}
