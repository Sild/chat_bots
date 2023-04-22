package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/gocarina/gocsv"
	log "github.com/sirupsen/logrus"
)

type SubscribeStatus = string

const (
	REGISTRY_FNAME    = "registry_data.csv"
	SUBSCRIBERS_FNAME = "subscribers.csv"
	LAST_MSG_TS_FNAME = "last_msg_ts.txt"

	UNDEFINED SubscribeStatus = "undef"
	NOT_FOUND SubscribeStatus = "not_found"
	SEARCHING SubscribeStatus = "searching"
	CHECKED   SubscribeStatus = "checked"
)

type UserInfo struct {
	Name     string `csv:"name"`
	Surname  string `csv:"surname"`
	Birthday string `csv:"birthday"`
	Status   string `csv:"status"`
	Updated  string `csv:"updated"`
}

type Subscribe struct {
	ChatID  int64           `csv:"chat_id"`
	Pattern string          `csv:"pattern"`
	Status  SubscribeStatus `csv:"status"`
}

type DBCli struct {
	registryPath    string
	subscribersPath string
	lastMsgTsPath   string
	registry        []*UserInfo
	subscribers     []*Subscribe
	lastMsgTs       time.Time
	coldCacheMtx    sync.Mutex
	subscribersMtx  sync.Mutex
	lastMsgMtx      sync.Mutex
}

func NewDBCli(dbFolderPath string) (*DBCli, error) {
	return &DBCli{
		registryPath:    dbFolderPath + "/" + REGISTRY_FNAME,
		subscribersPath: dbFolderPath + "/" + SUBSCRIBERS_FNAME,
		lastMsgTsPath:   dbFolderPath + "/" + LAST_MSG_TS_FNAME,
		registry:        nil,
		subscribers:     nil,
		lastMsgTs:       time.Time{},
	}, nil
}

func (dbCli *DBCli) GetRegistry() ([]*UserInfo, error) {
	dbCli.coldCacheMtx.Lock()
	defer dbCli.coldCacheMtx.Unlock()

	if dbCli.registry == nil {
		log.Info("GetRegistry: load from disk")
		usersInfo := []*UserInfo{}
		if err := readCsv(dbCli.registryPath, &usersInfo); err != nil {
			return nil, err
		}
		dbCli.registry = usersInfo
	} else {
		log.Trace("GetRegistry: load from memory")
	}

	return dbCli.registry, nil
}

func (dbCli *DBCli) RewriteRegistry(data []*UserInfo) error {
	dbCli.coldCacheMtx.Lock()
	defer dbCli.coldCacheMtx.Unlock()
	dbCli.registry = data
	return writeCsv(dbCli.registryPath, &data)
}

func (dbCli *DBCli) GetSubscribers() ([]*Subscribe, error) {
	dbCli.subscribersMtx.Lock()
	defer dbCli.subscribersMtx.Unlock()

	if dbCli.subscribers == nil {
		log.Info("GetSubscribers: load from disk")
		subs := []*Subscribe{}
		if err := readCsv(dbCli.subscribersPath, &subs); err != nil {
			return nil, err
		}
		dbCli.subscribers = subs
	} else {
		log.Trace("GetSubscribers: load from memory")
	}

	return dbCli.subscribers, nil
}

func (dbCli *DBCli) RewriteSubscribers(data []*Subscribe) error {
	dbCli.subscribersMtx.Lock()
	defer dbCli.subscribersMtx.Unlock()
	dbCli.subscribers = data
	return writeCsv(dbCli.subscribersPath, &dbCli.subscribers)
}

func (dbCli *DBCli) GetLastTime() (time.Time, error) {
	dbCli.lastMsgMtx.Lock()
	defer dbCli.lastMsgMtx.Unlock()

	if dbCli.lastMsgTs.Unix() == (time.Time{}).Unix() {
		log.Info("GetLastTime: load from disk")
		readFile, err := os.Open(dbCli.lastMsgTsPath)
		if err != nil {
			return time.Time{}, err
		}
		defer readFile.Close()
		fileScanner := bufio.NewScanner(readFile)
		fileScanner.Split(bufio.ScanLines)
		for fileScanner.Scan() {
			ts, err := strconv.Atoi(fileScanner.Text())
			if err != nil {
				return time.Time{}, err
			}
			dbCli.lastMsgTs = time.Unix(int64(ts), 0)
			return dbCli.lastMsgTs, nil
		}
	} else {
		log.Trace("GetLastTime: load from memory")
	}

	return dbCli.lastMsgTs, nil
}

func (dbCli *DBCli) UpdateLastTime(ts time.Time) {
	dbCli.lastMsgMtx.Lock()
	defer dbCli.lastMsgMtx.Unlock()

	dbCli.lastMsgTs = ts

	writeFile, err := os.OpenFile(dbCli.lastMsgTsPath, os.O_RDWR|os.O_CREATE|os.O_TRUNC, os.ModePerm)
	if err != nil {
		return
	}
	defer writeFile.Close()
	if _, err := writeFile.WriteString(fmt.Sprintf("%d", dbCli.lastMsgTs.Unix())); err != nil {
		log.Error(err)
	}
}

func readCsv(path string, out interface{}) error {
	file, err := os.OpenFile(path, os.O_RDWR|os.O_CREATE, os.ModePerm)
	if err != nil {
		log.Error(err)
		return err
	}
	defer file.Close()

	if err := gocsv.UnmarshalFile(file, out); err != nil {
		if strings.Contains(err.Error(), "empty csv file given") {
			return nil
		}
		log.Error(err)
		return err
	}
	return nil
}

func writeCsv(path string, in interface{}) error {
	file, err := os.OpenFile(path, os.O_RDWR|os.O_CREATE, os.ModePerm)
	if err != nil {
		log.Error(err)
		return err
	}
	defer file.Close()

	if err = file.Truncate(0); err != nil {
		log.Error(err)
		return err
	}
	if err := gocsv.MarshalFile(in, file); err != nil {
		log.Error(err)
		return err
	}
	return nil
}
