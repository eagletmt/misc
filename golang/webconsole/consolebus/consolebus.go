package consolebus

import (
	"sync"
)

type ConsoleBus struct {
	channel     chan []byte
	mutex       sync.Mutex
	subscribers map[int]func(buffer, current []byte)
	serial      int
	Buffer      []byte
}

func (bus *ConsoleBus) Publish(p []byte) {
	bus.channel <- p
}

func (bus *ConsoleBus) Subscribe(f func(buffer, current []byte)) int {
	bus.mutex.Lock()
	defer bus.mutex.Unlock()
	id := bus.serial
	bus.subscribers[id] = f
	bus.serial++
	return id
}

func (bus *ConsoleBus) Unsubscribe(id int) {
	bus.mutex.Lock()
	defer bus.mutex.Unlock()
	delete(bus.subscribers, id)
}

func (bus *ConsoleBus) Close() {
	close(bus.channel)
}

func New() *ConsoleBus {
	bus := new(ConsoleBus)
	bus.channel = make(chan []byte)
	bus.subscribers = make(map[int]func(buffer, current []byte))
	bus.Buffer = make([]byte, 0)
	go func() {
		for p := range bus.channel {
			bus.mutex.Lock()
			for _, f := range bus.subscribers {
				f(bus.Buffer, p)
			}
			bus.mutex.Unlock()
			if p != nil {
				bus.Buffer = append(bus.Buffer, p...)
			}
		}
	}()
	return bus
}
