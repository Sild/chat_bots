package database

import (
	"fmt"
	"math/rand"
	"sync"
	"sync/atomic"

	"github.com/sild/gosk/log"
	"github.com/sild/gosk/serial"
)

type jsonDB struct {
	Subscribers []Subscriber `json:"subscribers"`
	MsgsSent    atomic.Int64 `json:"msgs_sent"`
	mtx         *sync.Mutex  `json:"-"`
}

func NewJsonDB(jsonData string) DB {
	if jsonData == "" {
		return &jsonDB{
			Subscribers: []Subscriber{},
			MsgsSent:    atomic.Int64{},
			mtx:         &sync.Mutex{},
		}
	}
	db := serial.JsonSToObj[jsonDB](jsonData)
	db.mtx = &sync.Mutex{}
	return &db
}

func (db *jsonDB) AddSubscriber(subs Subscriber) {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	for _, s := range db.Subscribers {
		if s == subs {
			return
		}
	}
	db.Subscribers = append(db.Subscribers, subs)
}

func (db *jsonDB) RemoveSubscriber(subs Subscriber) {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	for i, s := range db.Subscribers {
		if s == subs {
			db.Subscribers = append(db.Subscribers[:i], db.Subscribers[i+1:]...)
			return
		}
	}
}

func (db *jsonDB) IsSubscriber(subs Subscriber) bool {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	for _, s := range db.Subscribers {
		if s == subs {
			return true
		}
	}
	return false
}

func (db *jsonDB) RandomSubscriber() (Subscriber, error) {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	if len(db.Subscribers) == 0 {
		return Subscriber{}, fmt.Errorf("No subscribers")
	}
	log.Info("Subscribers count: %d", len(db.Subscribers))
	position := rand.Intn(len(db.Subscribers))
	return db.Subscribers[position], nil
}

func (db *jsonDB) IncMsgSent() {
	db.MsgsSent.Add(1)
}

func (db *jsonDB) MsgSentCount() int {
	return int(db.MsgsSent.Load())
}

func (db *jsonDB) SubsCount() int {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	return len(db.Subscribers)
}

func (db *jsonDB) AsJson() string {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	return serial.ObjToJsonS(db)
}
