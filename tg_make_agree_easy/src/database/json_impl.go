package database

import (
	"fmt"
	"math/rand"
	"os"
	"sync"

	"github.com/sild/gosk/log"
	"github.com/sild/gosk/serial"
)

type jsonDB struct {
	DBPath      string       `json:"-"`
	Subscribers []Subscriber `json:"subscribers"`
	MsgsSent    uint64       `json:"msgs_sent"`
	mtx         *sync.Mutex  `json:"-"`
}

func NewJsonDB(dbPath string) DB {
	data, err := os.ReadFile(dbPath)
	if err != nil {
		log.Error("Fail to read dbPath=%s: %v. Will use empty data instead.", dbPath, err)
		data = make([]byte, 0)
	}

	jsonData := string(data)

	if jsonData == "" {
		return &jsonDB{
			DBPath:      dbPath,
			Subscribers: []Subscriber{},
			MsgsSent:    0,
			mtx:         &sync.Mutex{},
		}
	}
	db := serial.JsonSToObj[jsonDB](jsonData)
	db.DBPath = dbPath
	db.mtx = &sync.Mutex{}
	return &db
}

func (db *jsonDB) save() {
	log.Trace("Saving db to %s", db.DBPath)
	serial.ObjToJsonF(db, db.DBPath)
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
	db.save()
}

func (db *jsonDB) RemoveSubscriber(subs Subscriber) {
	db.mtx.Lock()
	defer db.mtx.Unlock()

	for i, s := range db.Subscribers {
		if s == subs {
			db.Subscribers = append(db.Subscribers[:i], db.Subscribers[i+1:]...)
			db.save()
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
	db.mtx.Lock()
	defer db.mtx.Unlock()
	db.MsgsSent++
	db.save()
}

func (db *jsonDB) MsgSentCount() uint64 {
	return db.MsgsSent
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
