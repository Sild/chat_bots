package db

type Role uint32

const (
	ADMIN Role = 0
	USER  Role = 1
)

type Person struct {
	Role     Role
	FlatNum  string
	Name     string
	Surname  string
	TgAcc    string
	PhoneNum string
}
