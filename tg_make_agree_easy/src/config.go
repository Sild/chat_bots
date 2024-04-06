package main

import (
	"context"
	"fmt"

	"github.com/sethvargo/go-envconfig"
)

type Config struct {
	TgToken         string `env:"SPEAKING_FROM_HEART_TOKEN"`
	SystemChannelID int    `env:"SPEAKING_FROM_HEART_SYSTEM_CHANNEL_ID"`
	DBPath          string `env:"SPEAKING_FROM_HEART_DB_PATH"`
}

func (c *Config) isValid() bool {
	return c.TgToken != "" && c.SystemChannelID != 0 && c.DBPath != ""
}

func NewConfigFromEnv() (*Config, error) {
	var conf Config
	if err := envconfig.Process(context.Background(), &conf); err != nil {
		return nil, err
	}
	if !conf.isValid() {
		return nil, fmt.Errorf("Some of config fields is empty")
	}
	return &conf, nil
}
