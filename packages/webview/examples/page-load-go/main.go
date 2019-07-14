package main

import (
	"io/ioutil"
	"log"
	"net"
	"net/http"
	"net/url"
	"os"
	"path/filepath"

	"github.com/zserge/webview"
)

var indexHTML = `
<!doctype html>
<html>
	<head>
		<meta http-equiv="X-UA-Compatible" content="IE=edge">
	</head>
	<body>
	  <h1>Hello, world</h1>
	</body>
</html>
`

func runLocalHTTP() {
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		log.Fatal(err)
	}
	go func() {
		defer ln.Close()
		http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
			w.Write([]byte(indexHTML))
		})
		log.Fatal(http.Serve(ln, nil))
	}()
	url := "http://" + ln.Addr().String()
	w := webview.New(webview.Settings{
		Title: "Loaded: Local HTTP Server",
		URL:   url,
	})
	defer w.Exit()
	w.Run()
}

func runLocalFile() {
	dir, err := ioutil.TempDir("", "webview")
	if err != nil {
		log.Fatal(err)
	}
	defer os.RemoveAll(dir)
	tmpfn := filepath.Join(dir, "index.html")
	if err := ioutil.WriteFile(tmpfn, []byte(indexHTML), 0666); err != nil {
		log.Fatal(err)
	}
	abs, err := filepath.Abs(tmpfn)
	if err != nil {
		log.Fatal(err)
	}
	log.Println("local tmp file: ", abs)
	w := webview.New(webview.Settings{
		Title: "Loaded: Local file URL",
		URL:   "file://" + abs,
	})
	defer w.Exit()
	w.Run()
}

func runDataURL() {
	w := webview.New(webview.Settings{
		Title: "Loaded: Data URL",
		URL:   "data:text/html," + url.PathEscape(indexHTML),
	})
	defer w.Exit()
	w.Run()
}

func runInjectJS() {
	w := webview.New(webview.Settings{
		Title: "Loaded: Injected via JavaScript",
		URL:   `data:text/html,<html><script type="text/javascript"></script></html>`,
	})
	defer w.Exit()
	w.Dispatch(func() {
		w.Eval(`document.body.innerHTML = "<h1>Hello, world</h1>";`)
	})
	w.Run()
}

func main() {
	runLocalHTTP()
	//runLocalFile()
	//runDataURL()
	//runInjectJS()
}
