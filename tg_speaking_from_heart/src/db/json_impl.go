package db

import (
	"math/rand"
	"sync"
	"sync/atomic"

	"github.com/sild/gosk/serial"
)

type jsonDB struct {
	Subscribers []string     `json:"subscribers"`
	MsgsSent    atomic.Int64 `json:"msgs_sent"`
	mtx         *sync.Mutex  `json:"-"`
}

func NewJsonDB(jsonData string) DB {
	if jsonData == "" {
		return &jsonDB{
			Subscribers: []string{},
			MsgsSent:    atomic.Int64{},
			mtx:         &sync.Mutex{},
		}
	}
	db := serial.JsonSToObj[jsonDB](jsonData)
	db.mtx = &sync.Mutex{}
	return &db
}

func (db *jsonDB) AddSubscriber(tgLogin string) {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	db.Subscribers = append(db.Subscribers, tgLogin)
}

func (db *jsonDB) RemoveSubscriber(tgLogin string) {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	for i, login := range db.Subscribers {
		if login == tgLogin {
			db.Subscribers = append(db.Subscribers[:i], db.Subscribers[i+1:]...)
			return
		}
	}
}

func (db *jsonDB) IsSubscriber(tgLogin string) bool {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	for _, login := range db.Subscribers {
		if login == tgLogin {
			return true
		}
	}
	return false
}

func (db *jsonDB) RandomSubscriber() string {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	position := rand.Intn(len(db.Subscribers))
	return db.Subscribers[position]
}

func (db *jsonDB) IncMsgSend() {
	db.MsgsSent.Add(1)
}

func (db *jsonDB) AsJson() string {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	return serial.ObjToJsonS(db)
}
