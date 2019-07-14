package main

import (
	"fmt"
	"log"
	"math/rand"
	"net"
	"net/http"
	"time"

	"github.com/zserge/webview"
)

const (
	windowWidth  = 480
	windowHeight = 320
)

var indexHTML = fmt.Sprintf(`
<!doctype html>
<html>
	<head>
		<meta http-equiv="X-UA-Compatible" content="IE=edge">
		<style>* { margin: 0; padding: 0; box-sizing: border-box; }</style>
	</head>
	<body>
		<canvas id="canvas" width="%d" height="%d">
			Your browser doesn't support HTML5 canvas element.
		</canvas>
		<script type="text/javascript">
			window.drawData = {};
			function draw() {
				window.external.invoke('draw');
				var canvas = document.getElementById('canvas');
				var ctx = canvas.getContext('2d');
				ctx.clearRect(0, 0, canvas.width, canvas.height);
				ctx.beginPath();
				ctx.moveTo(drawData.x1, drawData.y1);
				ctx.lineTo(drawData.x2, drawData.y2);
				ctx.stroke();
				window.requestAnimationFrame(draw);
			}
			draw();
		</script>
	</body>
</html>
`, windowWidth, windowHeight)

func startServer() string {
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
	return "http://" + ln.Addr().String()
}

var (
	numFrames int
	prevTime  time.Time
	totalTime time.Duration
)

func handleRPC(w webview.WebView, data string) {
	numFrames++
	if !prevTime.IsZero() {
		totalTime = totalTime + time.Since(prevTime)
	}
	prevTime = time.Now()
	if numFrames%100 == 0 {
		d := totalTime / time.Duration(numFrames)
		log.Println("time per frame:", d, " fps:", int(time.Second/d))
	}
	s := fmt.Sprintf(`drawData = {x1:%d,y1:%d,x2:%d,y2:%d}`,
		rand.Intn(windowWidth), rand.Intn(windowHeight),
		rand.Intn(windowWidth), rand.Intn(windowHeight))
	w.Eval(s)
}

func main() {
	url := startServer()
	w := webview.New(webview.Settings{
		Width:  windowWidth,
		Height: windowHeight,
		Title:  "Simple canvas demo",
		URL:    url,
		ExternalInvokeCallback: handleRPC,
	})
	defer w.Exit()
	w.Run()
}
