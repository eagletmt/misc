package main

import (
	"html/template"
	"log"
	"net/http"
	"os/exec"
)

func main() {
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		writeHtml(w, r)
	})
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

func writeHtml(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/html")
	w.WriteHeader(http.StatusOK)

	text := r.URL.Query().Get("text")

	const TOP_TEMPLATE = `<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta content="IE=edge" http-equiv="X-UA-Compatible" />
    <title>qrencode</title>
  </head>
  <body>
    <form action="/" method="GET">
      <input type="text" name="text" value="{{.}}" size=64 />
      <input type="submit" value="Generate QR" />
      <button id="generate-random" type="button">Random</button>
    </form>
    {{ if ne (len .) 0 }}
    <img src="/qrencode?text={{.}}" alt="QR code {{.}}" />
    {{ end }}
    <script>
      (function() {
        function randomString(len) {
          const table = 'abcdefghijklmnopqrstuvwxyz0123456789';
          var i, ret = '';
          for (i = 0; i < len; i++) {
            ret += table[Math.floor(Math.random() * table.length)];
          }
          return ret;
        }

        document.getElementById("generate-random").addEventListener('click', function(evt) {
          evt.preventDefault();
          var form = this.form;
          form['text'].value = randomString(32);
          form.submit();
        });
      })();
    </script>
    <p><a href="https://github.com/eagletmt/misc/tree/master/golang/qrencode-server">Source code</a>
  </body>
</html>`
	tmpl := template.Must(template.New("top").Parse(TOP_TEMPLATE))
	if err := tmpl.Execute(w, text); err != nil {
		log.Println(err)
	}
}
