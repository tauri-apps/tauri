<script>
  import { onMount } from "svelte";
  import { Dir } from "tauri/api/fs";
  import { listen, emit } from "tauri/api/event";
  import { getMatches } from "tauri/api/cli";
  import { setTitle, open as browerOpen } from "tauri/api/window";
  import { invoke, promisified } from "tauri/api/tauri";
  import { request } from "tauri/api/http";

  import initDialog from "./api/dialog";
  import initFs from "./api/fs";

  onMount(() => {
    initDialog();
    initFs();
  });

  let response;
  let urlValue = "https://tauri.studio";
  let httpMethod = "GET";
  let httpUrl = "";
  let httpBody = "";

  function registerResponse(value) {
    response = typeof value === "string" ? value : JSON.stringify(value);
  }
  window.registerResponse = registerResponse;

  const DirOptions = Object.keys(Dir)
    .filter(key => isNaN(parseInt(key)))
    .map(dir => [dir, Dir[dir]]);

  listen("rust-event", function(res) {
    response = JSON.stringify(res);
  });

  function onLogoClick() {
    browerOpen("https://tauri.studio/");
  }

  function cliMatches() {
    getMatches()
      .then(registerResponse)
      .catch(registerResponse);
  }

  function log() {
    invoke({
      cmd: "logOperation",
      event: "tauri-click",
      payload: "this payload is optional because we used Option in Rust"
    });
  }

  function performRequest() {
    promisified({
      cmd: "performRequest",
      endpoint: "dummy endpoint arg",
      body: {
        id: 5,
        name: "test"
      }
    })
      .then(registerResponse)
      .catch(registerResponse);
  }

  function emitEvent() {
    emit("js-event", "this is the payload string");
  }

  function openUrl() {
    browerOpen(urlValue);
  }

  function setWindowTitle() {
    setTitle(urlValue);
  }

  function _sendNotification() {
    new Notification("Notification title", {
      body: "This is the notification body"
    });
  }

  function notification(registerResponse) {
    if (Notification.permission === "default") {
      Notification.requestPermission()
        .then(function(response) {
          if (response === "granted") {
            _sendNotification();
          } else {
            registerResponse("Permission is " + response);
          }
        })
        .catch(registerResponse);
    } else if (Notification.permission === "granted") {
      _sendNotification();
    } else {
      registerResponse("Permission is denied");
    }
  }

  function makeHttpRequest() {
    let method = httpMethod || "GET";
    let url = httpUrl || "";
    let body = httpBody || "";

    const options = {
      url: url || "",
      method: method || "GET"
    };

    if (
      (body.startsWith("{") && body.endsWith("}")) ||
      (body.startsWith("[") && body.endsWith("]"))
    ) {
      body = JSON.parse(body);
    } else if (body.startsWith("/") || body.match(/\S:\//g)) {
      options.bodyAsFile = true;
    }
    options.body = body;

    request(options)
      .then(registerResponse)
      .catch(registerResponse);
  }
</script>

<style>
  * {
    font-family: Arial, Helvetica, sans-serif;
  }

  body {
    background: #889;
  }

  .logo-container {
    width: 95%;
    margin: 0px auto;
    overflow: hidden;
  }

  .logo-link {
    font-weight: 700;
    position: absolute;
    top: 150px;
    right: 10px;
  }

  .logo {
    width: 32px;
    height: 32px;
    cursor: pointer;
    position: fixed;
    z-index: 10;
    top: 7px;
    right: 10px;
  }

  #response {
    position: absolute;
    left: 10px;
    right: 10px;
    top: 440px;
    min-height: 110px;
    background: #aab;
    font-family: "Courier New", Courier, monospace;
    font-size: 12px;
    word-wrap: break-word;
    padding: 5px;
    border-radius: 5px;
    overflow-y: auto;
  }

  input,
  select {
    background: white;
    font-family: system-ui, sans-serif;
    border: 0;
    border-radius: 0.25rem;
    font-size: 1rem;
    line-height: 1.2;
    padding: 0.25rem 0.5rem;
    margin: 0.25rem;
  }

  button:hover,
  button:focus {
    background: #0053ba;
  }

  button:focus {
    outline: 1px solid #fff;
    outline-offset: -4px;
  }

  button:active {
    transform: scale(0.99);
  }

  .button {
    border: 0;
    border-radius: 0.25rem;
    background: #1e88e5;
    color: white;
    font-family: system-ui, sans-serif;
    font-size: 1rem;
    line-height: 1.2;
    white-space: nowrap;
    text-decoration: none;
    padding: 0.25rem 0.5rem;
    margin: 0.25rem;
    cursor: pointer;
  }

  .bottom {
    position: fixed;
    bottom: 0;
    left: 0;
    text-align: center;
    width: 100%;
    padding: 5px;
    background: #333;
    color: #eef;
  }

  .dark-link {
    color: white;
    text-decoration: none !important;
  }

  .tabs-container {
    position: fixed;
    height: 400px;
    top: 20px;
    left: 10px;
    right: 10px;
    z-index: 9;
  }

  .tabs {
    position: relative;
    min-height: 400px;
    clear: both;
  }

  .tab {
    float: left;
  }

  .tab > label {
    background: #eee;
    padding: 10px;
    border: 1px solid transparent;
    margin-left: -1px;
    position: relative;
    left: 1px;
  }

  .tabs > .tabber {
    border-top-left-radius: 5px;
  }

  .tabs > .tabber ~ .tabber {
    border-top-left-radius: none;
  }

  .tab [type="radio"] {
    display: none;
  }

  .content {
    position: absolute;
    top: 28px;
    left: 0;
    background: #bbc;
    right: 0;
    bottom: 0;
    padding: 20px;
    border: 1px solid transparent;
    border-top-right-radius: 5px;
    border-bottom-left-radius: 5px;
    border-bottom-right-radius: 5px;
  }

  [type="radio"]:checked ~ label {
    background: #bbc;
    border-bottom: 1px solid transparent;
    z-index: 2;
  }

  [type="radio"]:checked ~ label ~ .content {
    z-index: 1;
  }
</style>

<main>
  <div class="logo-container">
    <img src="icon.png" class="logo" on:click={onLogoClick} alt="logo" />
  </div>

  <div class="tabs-container">
    <div class="tabs">
      <div class="tab">
        <input type="radio" id="tab-1" name="tab-group-1" checked />
        <label class="tabber" for="tab-1">Messages</label>
        <div class="content">
          <button class="button" id="log" on:click={log}>Call Log API</button>
          <button class="button" id="request" on:click={performRequest}>
            Call Request (async) API
          </button>
          <button class="button" id="event" on:click={emitEvent}>
            Send event to Rust
          </button>
          <button class="button" id="notification" on:click={notification}>
            Send test notification
          </button>

          <div style="margin-top: 24px">
            <input id="title" value="Awesome Tauri Example!" />
            <button class="button" id="set-title">Set title</button>
          </div>
        </div>
      </div>
      <div class="tab">
        <input type="radio" id="tab-2" name="tab-group-1" />
        <label class="tabber" for="tab-2">File System</label>
        <div class="content">
          <div style="margin-top: 24px">
            <select class="button" id="dir">
              <option value="">None</option>
              {#each DirOptions as dir}
                <option value={dir[1]}>{dir[0]}</option>
              {/each}
            </select>
            <input id="path-to-read" placeholder="Type the path to read..." />
            <button class="button" id="read">Read</button>
          </div>
          <div style="margin-top: 24px">
            <input id="dialog-default-path" placeholder="Default path" />
            <input id="dialog-filter" placeholder="Extensions filter" />
            <div>
              <input type="checkbox" id="dialog-multiple" />
              <label>Multiple</label>
            </div>
            <div>
              <input type="checkbox" id="dialog-directory" />
              <label>Directory</label>
            </div>

            <button class="button" id="open-dialog">Open dialog</button>
            <button class="button" id="save-dialog">Open save dialog</button>
          </div>
        </div>
      </div>

      <div class="tab">
        <input type="radio" id="tab-3" name="tab-group-1" />
        <label class="tabber" for="tab-3">Communication</label>
        <div class="content">
          <div style="margin-top: 24px">
            <input id="url" bind:value={urlValue} />
            <button class="button" id="open-url" on:click={openUrl}>
              Open URL
            </button>
          </div>

          <div style="margin-top: 24px">
            <select class="button" id="request-method" bind:value={httpMethod}>
              <option value="GET">GET</option>
              <option value="POST">POST</option>
              <option value="PUT">PUT</option>
              <option value="PATCH">PATCH</option>
              <option value="DELETE">DELETE</option>
            </select>
            <input
              id="request-url"
              placeholder="Type the request URL..."
              bind:value={httpUrl} />
            <br />
            <textarea
              id="request-body"
              placeholder="Request body"
              rows="5"
              style="width:100%;margin-right:10px;font-size:12px"
              bind:value={httpBody} />
            <button class="button" id="make-request" on:click={makeHttpRequest}>
              Make request
            </button>
          </div>
        </div>
      </div>
      <div class="tab">
        <input type="radio" id="tab-4" name="tab-group-1" />
        <label class="tabber" for="tab-4">CLI</label>
        <div class="content">
          <div style="margin-top: 24px">
            <button class="button" id="cli-matches" on:click={cliMatches}>
              Get matches
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
  <div id="response">{response}</div>
  <div class="bottom">
    <a class="dark-link" target="_blank" href="https://tauri.studio">
      Tauri Documentation
    </a>
    &nbsp;&nbsp;&nbsp;
    <a
      class="dark-link"
      target="_blank"
      href="https://github.com/tauri-apps/tauri">
      Github Repo
    </a>
    &nbsp;&nbsp;&nbsp;
    <a
      class="dark-link"
      target="_blank"
      href="https://github.com/tauri-apps/tauri/tree/dev/tauri/examples/communication">
      Source for this App
    </a>
  </div>
</main>
