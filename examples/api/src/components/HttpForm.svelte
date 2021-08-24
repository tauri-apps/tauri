<script>
	let foo = 'baz'
	let bar = 'qux'
	let result = null
	
	async function doPost () {
    let url = navigator.userAgent.includes("Windows") ? "https://customprotocol.test/example.html" : "customprotocol://test/example.html";
		const res = await fetch(url, {
			method: 'POST',
			body: JSON.stringify({
				foo,
				bar
			})
		})
		
		const json = await res.json()
		result = JSON.stringify(json)
	}
</script>


<input bind:value={foo} />
<input bind:value={bar} />
<button type="button" on:click={doPost}>
	Post it.
</button>
<p>
	Result:
</p>
<pre>
{result}
</pre>