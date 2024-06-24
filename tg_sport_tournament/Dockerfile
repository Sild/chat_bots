FROM golang:1.21 as go_builder

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download
COPY src ./src

RUN go build ./...
RUN go test ./...

RUN CGO_ENABLED=0 GOOS=linux go build -o /tg_speaking_from_heart src/*.go


FROM alpine:latest
COPY --from=go_builder /tg_sport_tournament /tg_sport_tournament
CMD ["/tg_sport_tournament"]
