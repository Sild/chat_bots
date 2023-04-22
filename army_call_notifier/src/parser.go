package main

import (
	log "github.com/sirupsen/logrus"
)

type Parser struct {
}

func NewParser() (*Parser, error) {
	return &Parser{}, nil
}

func (parser *Parser) GetData() ([]*UserInfo, error) {
	log.Info("Parser: creating...")
	defer log.Info("Parser: created...")

	data := []*UserInfo{}
	data = append(data, &UserInfo{Name: "test1", Surname: "test2"})
	return data, nil
}
