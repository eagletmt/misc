package main

import (
	"database/sql"

	_ "github.com/mattn/go-sqlite3"
)

const DB_NAME = "webconsole.sqlite3"

type DB struct {
	db *sql.DB
}

type Execution struct {
	Id      int    `json:"id"`
	Command string `json:"command"`
	Status  int    `json:"status"`
	Output  string `json:"output"`
}

func OpenDB() (*DB, error) {
	db, err := sql.Open("sqlite3", DB_NAME)
	if err != nil {
		return nil, err
	}
	return &DB{db: db}, nil
}

func (db *DB) Close() error {
	return db.db.Close()
}

func (db *DB) Insert(command string) (int, error) {
	result, err := db.db.Exec("INSERT INTO executions (command) VALUES (?)", command)
	if err != nil {
		return 0, err
	}
	id64, err := result.LastInsertId()
	if err != nil {
		return 0, err
	}
	return int(id64), nil
}

func (db *DB) Finish(id int, status int, buffer []byte) error {
	_, err := db.db.Exec("UPDATE executions SET status = ?, output = ? WHERE id = ?", status, buffer, id)
	return err
}

func (db *DB) Get(id int) (*Execution, error) {
	var command, output string
	var status int
	err := db.db.QueryRow("SELECT command, status, output FROM executions WHERE id = ? LIMIT 1", id).Scan(&command, &status, &output)
	switch {
	case err == sql.ErrNoRows:
		return nil, nil
	case err != nil:
		return nil, err
	default:
		return &Execution{Id: id, Command: command, Status: status, Output: output}, nil
	}
}
