package main

//go:generate go get -u github.com/jteeuwen/go-bindata/...
//go:generate go-bindata -pkg $GOPACKAGE -o assets.go -prefix assets/ assets/

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"mime"
	"net"
	"net/http"
	"path/filepath"

	"github.com/zserge/webview"
)

func startServer() string {
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		log.Fatal(err)
	}
	go func() {
		defer ln.Close()
		http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
			path := r.URL.Path
			if len(path) > 0 && path[0] == '/' {
				path = path[1:]
			}
			if path == "" {
				path = "index.html"
			}
			if bs, err := Asset(path); err != nil {
				w.WriteHeader(http.StatusNotFound)
			} else {
				w.Header().Add("Content-Type", mime.TypeByExtension(filepath.Ext(path)))
				io.Copy(w, bytes.NewBuffer(bs))
			}
		})
		log.Fatal(http.Serve(ln, nil))
	}()
	return "http://" + ln.Addr().String()
}

// Task is a data model type, it contains information about task name and status (done/not done).
type Task struct {
	Name string `json:"name"`
	Done bool   `json:"done"`
}

// Tasks is a global data model, to keep things simple.
var Tasks = []Task{}

func render(w webview.WebView, tasks []Task) {
	b, err := json.Marshal(tasks)
	if err == nil {
		w.Eval(fmt.Sprintf("rpc.render(%s)", string(b)))
	}
}

func handleRPC(w webview.WebView, data string) {
	cmd := struct {
		Name string `json:"cmd"`
	}{}
	if err := json.Unmarshal([]byte(data), &cmd); err != nil {
		log.Println(err)
		return
	}
	switch cmd.Name {
	case "init":
		render(w, Tasks)
	case "log":
		logInfo := struct {
			Text string `json:"text"`
		}{}
		if err := json.Unmarshal([]byte(data), &logInfo); err != nil {
			log.Println(err)
		} else {
			log.Println(logInfo.Text)
		}
	case "addTask":
		task := Task{}
		if err := json.Unmarshal([]byte(data), &task); err != nil {
			log.Println(err)
		} else if len(task.Name) > 0 {
			Tasks = append(Tasks, task)
			render(w, Tasks)
		}
	case "markTask":
		taskInfo := struct {
			Index int  `json:"index"`
			Done  bool `json:"done"`
		}{}
		if err := json.Unmarshal([]byte(data), &taskInfo); err != nil {
			log.Println(err)
		} else if taskInfo.Index >= 0 && taskInfo.Index < len(Tasks) {
			Tasks[taskInfo.Index].Done = taskInfo.Done
			render(w, Tasks)
		}
	case "clearDoneTasks":
		newTasks := []Task{}
		for _, task := range Tasks {
			if !task.Done {
				newTasks = append(newTasks, task)
			}
		}
		Tasks = newTasks
		render(w, Tasks)
	}
}

func main() {
	url := startServer()
	w := webview.New(webview.Settings{
		Width:  320,
		Height: 480,
		Title:  "Todo App",
		URL:    url,
		ExternalInvokeCallback: handleRPC,
	})
	defer w.Exit()
	w.Run()
}
