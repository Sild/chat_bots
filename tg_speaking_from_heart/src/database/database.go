package database

type Subscriber struct {
	ChatID int
}
type DB interface {
	AddSubscriber(subs Subscriber)
	RemoveSubscriber(subs Subscriber)
	IsSubscriber(subs Subscriber) bool
	RandomSubscriber() (Subscriber, error)
	IncMsgSent()
	MsgSentCount() int
	SubsCount() int
	AsJson() string
}
