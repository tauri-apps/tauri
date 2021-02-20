const methodSelect = document.getElementById("request-method");
const requestUrlInput = document.getElementById("request-url");
const requestBodyInput = document.getElementById("request-body");

let client
window.__TAURI__.http.getClient().then(function (c) {
  client = c
})

document.getElementById("make-request").addEventListener("click", function () {
  const method = methodSelect.value || "GET";
  const url = requestUrlInput.value || "";

  const options = {
    url: url,
    method: method,
  };

  let httpBody = requestBodyInput.value || "";
  if (
    (httpBody.startsWith("{") && httpBody.endsWith("}")) ||
    (httpBody.startsWith("[") && httpBody.endsWith("]"))
  ) {
    options.body = window.__TAURI__.http.Body.json(JSON.parse(httpBody));
  } else if (httpBody !== '') {
    options.body = window.__TAURI__.http.Body.text(httpBody)
  }

  client
    .request(options)
    .then(registerResponse)
    .catch(registerResponse);
});
