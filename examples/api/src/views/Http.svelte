<script>
  import { getClient, Body, ResponseType } from '@tauri-apps/api/http'
  import { JsonView } from '@zerodevx/svelte-json-view'

  let httpMethod = 'GET'
  let httpBody = ''

  export let onMessage

  async function makeHttpRequest() {
    const client = await getClient().catch((e) => {
      onMessage(e)
      throw e
    })
    let method = httpMethod || 'GET'

    const options = {
      url: 'http://localhost:3003',
      method: method || 'GET'
    }

    if (
      (httpBody.startsWith('{') && httpBody.endsWith('}')) ||
      (httpBody.startsWith('[') && httpBody.endsWith(']'))
    ) {
      options.body = Body.json(JSON.parse(httpBody))
    } else if (httpBody !== '') {
      options.body = Body.text(httpBody)
    }

    client.request(options).then(onMessage).catch(onMessage)
  }

  /// http form
  let foo = 'baz'
  let bar = 'qux'
  let result = null
  let multipart = true

  async function doPost() {
    const client = await getClient().catch((e) => {
      onMessage(e)
      throw e
    })

    result = await client.request({
      url: 'http://localhost:3003',
      method: 'POST',
      body: Body.form({
        foo,
        bar
      }),
      headers: multipart
        ? { 'Content-Type': 'multipart/form-data' }
        : undefined,
      responseType: ResponseType.Text
    })
  }
</script>

<form on:submit|preventDefault={makeHttpRequest}>
  <select class="input" id="request-method" bind:value={httpMethod}>
    <option value="GET">GET</option>
    <option value="POST">POST</option>
    <option value="PUT">PUT</option>
    <option value="PATCH">PATCH</option>
    <option value="DELETE">DELETE</option>
  </select>
  <br />
  <textarea
    class="input h-auto w-100%"
    id="request-body"
    placeholder="Request body"
    rows="5"
    bind:value={httpBody}
  />
  <br />
  <button class="btn" id="make-request"> Make request </button>
</form>

<br />

<h3>HTTP Form</h3>

<div class="flex gap-2 children:grow">
  <input class="input" bind:value={foo} />
  <input class="input" bind:value={bar} />
</div>
<br />
<label>
  <input type="checkbox" bind:checked={multipart} />
  Multipart
</label>
<br />
<br />
<button class="btn" type="button" on:click={doPost}> Post it</button>
<br />
<br />
<JsonView json={result} />
