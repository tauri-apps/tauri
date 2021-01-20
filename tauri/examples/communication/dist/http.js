const methodSelect = document.getElementById("request-method");
const requestUrlInput = document.getElementById("request-url");
const requestBodyInput = document.getElementById("request-body");

document.getElementById("make-request").addEventListener("click", function () {
  const method = methodSelect.value || "GET";
  const url = requestUrlInput.value || "";

  const options = {
    url: url,
    method: method,
  };

  let body = requestBodyInput.value || "";
  if (
    (body.startsWith("{") && body.endsWith("}")) ||
    (body.startsWith("[") && body.endsWith("]"))
  ) {
    body = JSON.parse(body);
  } else if (body.startsWith("/") || body.match(/\S:\//g)) {
    options.bodyAsFile = true;
  }
  options.body = body;

  window.__TAURI__.http
    .request(options)
    .then(registerResponse)
    .catch(registerResponse);
});
