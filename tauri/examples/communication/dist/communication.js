document.getElementById("log").addEventListener("click", function () {
  console.log("log");
  window.__TAURI__.tauri.invoke({
    cmd: "logOperation",
    event: "tauri-click",
    payload: "this payload is optional because we used Option in Rust",
  });
});

document.getElementById("request").addEventListener("click", function () {
  window.__TAURI__.tauri
    .invoke({
      cmd: "performRequest",
      endpoint: "dummy endpoint arg",
      body: {
        id: 5,
        name: "test",
      },
    })
    .then(registerResponse)
    .catch(registerResponse);
});

document.getElementById("event").addEventListener("click", function () {
  window.__TAURI__.event.emit("js-event", "this is the payload string");
});
