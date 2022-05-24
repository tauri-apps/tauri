<script>
	import { getClient, Body, ResponseType } from "@tauri-apps/api/http"
	import { JsonView } from '@zerodevx/svelte-json-view'
	let foo = 'baz'
	let bar = 'qux'
	let result = null
	let multipart = true
	
	async function doPost () {
		const client = await getClient().catch(e => {
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
			headers: multipart ? { 'Content-Type': 'multipart/form-data' } : undefined,
			responseType: ResponseType.Text
		})
	}
</script>

<div>
	<input bind:value={foo} />
	<input bind:value={bar} />
	<label>
		<input type="checkbox" bind:checked={multipart} />
		Multipart
	</label>
	<button type="button" on:click={doPost}>
		Post it.
	</button>
	<JsonView json={result} />
</div>
