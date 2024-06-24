package database

type Team struct {
	UserTgIDs []string `json:"users"`
}

type Activity struct {
	ID       int    `json:"id"`
	Question string `json:"question"`
}

type Report struct {
	UserTgID   string `json:"user"`
	ActivityID int    `json:"activity_id"`
	Value      string `json:"value"`
	AddTs      int    `json:"add_ts"`
}

type DB interface {
	AddReport(userID string, report Report)
	GetTeams() map[string]Team
	GetActivities() map[string]Activity
	UserExists(userID string) bool
	AsJson() string
}
