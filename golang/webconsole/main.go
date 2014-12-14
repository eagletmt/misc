package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"sync"
	"syscall"

	"github.com/zenazn/goji"
	"github.com/zenazn/goji/web"

	"./consolebus"
)

type Buses struct {
	buses map[int]*consolebus.ConsoleBus
	mutex sync.Mutex
}

func NewBuses() *Buses {
	buses := new(Buses)
	buses.buses = make(map[int]*consolebus.ConsoleBus)
	return buses
}

func main() {
	goji.Get("/", func(w http.ResponseWriter, r *http.Request) {
		http.ServeFile(w, r, "views/index.html")
	})
	goji.Get("/js/webconsole.js", func(w http.ResponseWriter, r *http.Request) {
		http.ServeFile(w, r, "js/webconsole.js")
	})

	buses := NewBuses()
	goji.Post("/executions", func(w http.ResponseWriter, r *http.Request) {
		createExecution(w, r, buses)
	})
	goji.Get("/executions/:id", getExecution)
	goji.Get("/executions/:id/console", func(c web.C, w http.ResponseWriter, r *http.Request) {
		getExecutionConsole(c, w, r, buses)
	})
	goji.Serve()
}

func serverError(w http.ResponseWriter, err error) {
	log.Println(err)
	w.WriteHeader(http.StatusInternalServerError)
}

type Buffer struct {
	bus *consolebus.ConsoleBus
}

func (b *Buffer) Write(p []byte) (int, error) {
	b.bus.Publish(p)
	return len(p), nil
}

func createExecution(w http.ResponseWriter, r *http.Request, buses *Buses) {
	command := r.FormValue("command")

	if command == "" {
		w.WriteHeader(http.StatusBadRequest)
		return
	}
	command = strings.Replace(command, "\r\n", "\n", -1)

	db, err := OpenDB()
	if err != nil {
		serverError(w, err)
		return
	}
	defer db.Close()

	buses.mutex.Lock()
	defer buses.mutex.Unlock()
	id, err := db.Insert(command)
	if err != nil {
		serverError(w, err)
		return
	}

	bus := consolebus.New()
	cmd := exec.Command("bash", "-c", command)
	null, err := os.Open(os.DevNull)
	if err != nil {
		serverError(w, err)
		return
	}
	cmd.Stdin = null
	buffer := &Buffer{bus: bus}
	cmd.Stdout = buffer
	cmd.Stderr = buffer
	buses.buses[id] = bus
	cmd.Start()
	go waitCommand(buses, id, bus, cmd)

	http.Redirect(w, r, fmt.Sprintf("/executions/%d", id), http.StatusFound)
}

func waitCommand(buses *Buses, id int, bus *consolebus.ConsoleBus, cmd *exec.Cmd) {
	cmdErr := cmd.Wait()
	db, err := OpenDB()
	if err == nil {
		defer db.Close()
		status := 0
		if cmdErr != nil {
			status = determineExitCode(cmdErr)
		}
		db.Finish(id, status, bus.Buffer)
	} else {
		log.Println(err)
		return
	}
	bus.Publish(nil)
	buses.mutex.Lock()
	defer buses.mutex.Unlock()
	delete(buses.buses, id)
	bus.Close()
}

const UNKOWN_EXIT_CODE = 1024

func determineExitCode(err error) int {
	exitErr, ok := err.(*exec.ExitError)
	if !ok {
		return UNKOWN_EXIT_CODE
	}
	status, ok := exitErr.Sys().(syscall.WaitStatus)
	if !ok {
		return UNKOWN_EXIT_CODE
	}
	return status.ExitStatus()
}

func getExecution(c web.C, w http.ResponseWriter, r *http.Request) {
	idStr := c.URLParams["id"]
	isJson := false
	if strings.HasSuffix(idStr, ".json") {
		idStr = idStr[:len(idStr)-5]
		isJson = true
	}
	id, err := strconv.Atoi(idStr)
	if err != nil {
		http.NotFound(w, r)
		return
	}

	db, err := OpenDB()
	if err != nil {
		serverError(w, err)
		return
	}
	defer db.Close()

	execution, err := db.Get(id)
	if err != nil {
		serverError(w, err)
		return
	} else if execution == nil {
		http.NotFound(w, r)
		return
	}

	if !isJson {
		http.ServeFile(w, r, "views/execution.html")
		return
	}

	body, err := json.Marshal(execution)
	if err != nil {
		serverError(w, err)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write(body)
}

type ConsoleOutput struct {
	Output string `json:"output"`
}

func getExecutionConsole(c web.C, w http.ResponseWriter, r *http.Request, buses *Buses) {
	idStr := c.URLParams["id"]
	id, err := strconv.Atoi(idStr)
	if err != nil {
		http.NotFound(w, r)
		return
	}

	f, ok := w.(http.Flusher)
	if !ok {
		w.WriteHeader(http.StatusPreconditionFailed)
		return
	}

	buses.mutex.Lock()
	bus, ok := buses.buses[id]
	buses.mutex.Unlock()
	if !ok {
		http.NotFound(w, r)
		return
	}

	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Connection", "keep-alive")
	w.WriteHeader(http.StatusOK)

	initialized := false
	finish := make(chan bool)
	bus.Subscribe(func(buffer, current []byte) {
		if current == nil {
			finish <- true
		} else {
			if !initialized {
				initialized = true
				consoleOutput(buffer, w)
			}
			consoleOutput(current, w)
			f.Flush()
		}
	})
	<-finish
	consoleExit(w)
}

func consoleOutput(output []byte, w http.ResponseWriter) {
	fmt.Fprintln(w, "event: console-output")
	fmt.Fprint(w, "data: ")
	json, _ := json.Marshal(ConsoleOutput{Output: string(output)})
	w.Write(json)
	fmt.Fprintln(w, "\n")
}

func consoleExit(w http.ResponseWriter) {
	fmt.Fprintln(w, "event: console-exit")
	fmt.Fprintln(w, "data: 0\n")
}
