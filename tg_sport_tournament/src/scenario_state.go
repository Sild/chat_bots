package main

import "sync"

const (
	SelectActivities int = 0
	AddReport          = 1
)

type AppState struct {
	chatStates map[string]string // userTgID -> selectedActivity
	mtx        *sync.Mutex
}

func NewState() *AppState {
	return &AppState{
		chatStates: make(map[string]string),
		mtx:        new(sync.Mutex),
	}

}

func (this *AppState) GetActivity(userTgID string) string {
	this.mtx.Lock()
	defer this.mtx.Unlock()
	return this.chatStates[userTgID]
}

func (this *AppState) SetActivity(userTgID string, value string) {
	this.mtx.Lock()
	defer this.mtx.Unlock()
	this.chatStates[userTgID] = value
}
