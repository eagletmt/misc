package main

import (
	"log"
	"net/http"
	"os/exec"
)

func main() {
	http.HandleFunc("/qrencode", func(w http.ResponseWriter, r *http.Request) {
		body, err := generateQrCode(r)
		if err == nil {
			if body == nil {
				w.Header().Set("Content-Type", "text/plain")
				w.WriteHeader(http.StatusBadRequest)
			} else {
				w.Header().Set("Content-Type", "image/svg+xml")
				w.WriteHeader(http.StatusOK)
				w.Write(body)
			}
		} else {
			log.Printf("ParseForm error: %s", err)
			w.Header().Set("Content-Type", "text/plain")
			w.WriteHeader(http.StatusInternalServerError)
		}
	})
	log.Fatal(http.ListenAndServe(":8994", nil))
}

func generateQrCode(r *http.Request) ([]byte, error) {
	text := r.URL.Query().Get("text")
	if text == "" {
		return nil, nil
	}
	return exec.Command("/usr/bin/qrencode", "-t", "svg", "-o", "-", text).Output()
}
