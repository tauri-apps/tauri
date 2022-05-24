<script>
  import { getClient, Body } from "@tauri-apps/api/http";
  let httpMethod = "GET";
  let httpBody = "";

  export let onMessage;

  async function makeHttpRequest() {
    const client = await getClient().catch(e => {
      onMessage(e)
      throw e
    });
    let method = httpMethod || "GET";

    const options = {
      url: "http://localhost:3003",
      method: method || "GET",
    };

    if (
      (httpBody.startsWith("{") && httpBody.endsWith("}")) ||
      (httpBody.startsWith("[") && httpBody.endsWith("]"))
    ) {
      options.body = Body.json(JSON.parse(httpBody));
    } else if (httpBody !== "") {
      options.body = Body.text(httpBody);
    }

    client.request(options).then(onMessage).catch(onMessage);
  }
</script>

<form on:submit|preventDefault={makeHttpRequest}>
  <select class="button" id="request-method" bind:value={httpMethod}>
    <option value="GET">GET</option>
    <option value="POST">POST</option>
    <option value="PUT">PUT</option>
    <option value="PATCH">PATCH</option>
    <option value="DELETE">DELETE</option>
  </select>
  <br />
  <textarea
    id="request-body"
    placeholder="Request body"
    rows="5"
    bind:value={httpBody}
  />
  <button class="button" id="make-request"> Make request </button>
</form>

<style>
#request-body {
  width:100%;
  margin-right:10px;
  font-size:12px;
}
</style>
