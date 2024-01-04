package main

import (
	"encoding/json"
	"math/rand"
	"sync"
	"sync/atomic"
)

type DB struct {
	Subscribers []string
	MsgsSent	atomic.Int64
	mtx 		*sync.Mutex
}

func NewFromJson(jsonData string) (*DB, error) {
	db := &DB{}
	err := json.Unmarshal([]byte(jsonData), db)
	if err != nil {
		return nil, err
	}
	return db, nil
}

func (db *DB) AddSubscriber(tgLogin string) {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	db.Subscribers = append(db.Subscribers, tgLogin)
}

func (db *DB) SelectReceiver() string {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	position := rand.Intn(len(db.Subscribers))
	return db.Subscribers[position]
}

func (db *DB) IncMsgSend() {
	db.MsgsSent.Add(1)
}

func (db *DB) AsJson() (string, error) {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	result, err := json.Marshal(db)
    if err != nil {
        return "", err
    }
	return string(result), nil
}