<script lang="ts">
	import { execute } from 'tauri-plugin-{{ plugin_name }}-api'

	let response = ''

	function updateResponse(returnValue) {
		response += (typeof returnValue === 'string' ? returnValue : JSON.stringify(returnValue)) + '<br>'
	}

	function _execute() {
		execute().then(updateResponse).catch(updateResponse)
	}
</script>

<div>
	<button on:click="{_execute}">Execute</button>
</div>
