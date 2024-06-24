package database

import (
	"encoding/json"
	"os"
	"sync"

	"github.com/sild/gosk/log"
	"github.com/sild/gosk/serial"
)

type jsonDB struct {
	DBPath     string              `json:"-"`
	Teams      map[string]Team     `json:"teams"`
	Activities map[string]Activity `json:"activities"`
	Reports    []Report            `json:"reports"`
	mtx        *sync.Mutex         `json:"-"`
}

func NewJsonDB(dbPath string) DB {
	data, err := os.ReadFile(dbPath)
	if err != nil {
		log.Error("Fail to read dbPath=%s: %v. Will use empty data instead.", dbPath, err)
		data = make([]byte, 0)
	}

	jsonData := string(data)

	if jsonData == "" {
		db := &jsonDB{
			DBPath:     dbPath,
			Teams:      map[string]Team{},
			Activities: map[string]Activity{},
			Reports:    []Report{},
			mtx:        &sync.Mutex{},
		}
		db.save()
		return db
	}
	db := serial.JsonSToObj[jsonDB](jsonData)
	db.DBPath = dbPath
	db.mtx = &sync.Mutex{}
	return &db
}

func (db *jsonDB) save() {
	log.Trace("Saving db to %s", db.DBPath)
	bytes, err := json.MarshalIndent(db, "", "  ")
	if err != nil {
		log.Error("ObjToJsonS: %v", err)
	}
	if err := os.WriteFile(db.DBPath, bytes, 0644); err != nil {
		log.Error("ObjToJsonF: %v", err)
	}
}

func (db *jsonDB) AddReport(userTgID string, report Report) {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	db.Reports = append(db.Reports, report)
	db.save()
}

func (db *jsonDB) GetTeams() map[string]Team {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	return db.Teams
}

func (db *jsonDB) UserExists(userTgID string) bool {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	for _, team := range db.Teams {
		for _, user := range team.UserTgIDs {
			if user == userTgID {
				return true

			}
		}
	}
	return false
}

func (db *jsonDB) GetActivities() map[string]Activity {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	return db.Activities
}

func (db *jsonDB) AsJson() string {
	db.mtx.Lock()
	defer db.mtx.Unlock()
	return serial.ObjToJsonS(db)
}
