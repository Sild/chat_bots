module army_call_notifier

go 1.19

require (
	github.com/go-telegram-bot-api/telegram-bot-api/v5 v5.5.1
	github.com/gocarina/gocsv v0.0.0-20230406101422-6445c2b15027
	github.com/sirupsen/logrus v1.9.0
)

require (
	golang.org/x/sys v0.7.0 // indirect
	internal/app v0.0.0-00010101000000-000000000000 // indirect
	internal/db v0.0.0-00010101000000-000000000000 // indirect
)

replace internal/db => ./src/internal/db

replace internal/app => ./src/internal/app
