package db

type DB interface {
	AddSubscriber(tgLogin string)
	RemoveSubscriber(tgLogin string)
	IsSubscriber(tgLogin string) bool
	RandomSubscriber() string
	IncMsgSend()
	AsJson() string
}
