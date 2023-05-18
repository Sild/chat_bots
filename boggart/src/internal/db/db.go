package db

import (
	"os"
	"strings"
	"sync"

	"github.com/gocarina/gocsv"
	log "github.com/sirupsen/logrus"
)

type UserInfo struct {
	Name     string `csv:"name"`
	Surname  string `csv:"surname"`
	Birthday string `csv:"birthday"`
	Status   string `csv:"status"`
	Updated  string `csv:"updated"`
}

type DBCli struct {
	dbPath       string
	dbMtx        sync.Mutex
	personsCache []*Person
}

func NewDBCli(dbPath string) (*DBCli, error) {
	return &DBCli{
		dbPath: dbPath,
	}, nil
}

func (dbCli *DBCli) GetAllPersons() ([]*Person, error) {
	dbCli.dbMtx.Lock()
	defer dbCli.dbMtx.Unlock()

	if dbCli.personsCache == nil {
		log.Info("GetPersona: load from disk")
		persons := []*Person{}
		if err := readCsv(dbCli.dbPath, &persons); err != nil {
			return nil, err
		}
		dbCli.personsCache = persons
	} else {
		log.Trace("GetSubscribers: load from memory")
	}

	return dbCli.personsCache, nil
}

func (dbCli *DBCli) RewritePersons(data []*Person) error {
	dbCli.dbMtx.Lock()
	defer dbCli.dbMtx.Unlock()
	dbCli.personsCache = data
	return writeCsv(dbCli.dbPath, &dbCli.personsCache)
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
