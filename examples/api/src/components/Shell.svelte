<script>
  import { Command } from "@tauri-apps/api/shell"
  const windows = navigator.userAgent.includes('Windows')
  let cmd = windows ? 'cmd' : 'sh'
  let args = windows ? ['/C'] : ['-c']

  export let onMessage;

  let script = 'echo "hello world"'
  let cwd = null
  let env = 'SOMETHING=value ANOTHER=2'
  let stdin = ''
  let child

  function _getEnv() {
    return env.split(' ').reduce((env, clause) => {
      let [key, value] = clause.split('=')
      return {
        ...env,
        [key]: value
      }
    }, {})
  }

  function spawn() {
    child = null
    const command = new Command(cmd, [...args, script], { cwd: cwd || null, env: _getEnv() })

    command.on('close', data => {
      onMessage(`command finished with code ${data.code} and signal ${data.signal}`)
      child = null
    })
    command.on('error', error => onMessage(`command error: "${error}"`))

    command.stdout.on('data', line => onMessage(`command stdout: "${line}"`))
    command.stderr.on('data', line => onMessage(`command stderr: "${line}"`))
    
    command.spawn()
      .then(c => {
        child = c
      })
      .catch(onMessage)
  }

  function kill() {
    child.kill().then(() => onMessage('killed child process')).error(onMessage)
  }

  function writeToStdin() {
    child.write(stdin).catch(onMessage)
  }
</script>

<div>
  <div>
    <input bind:value={script}>
    <button class="button" on:click={spawn}>Run</button>
    <button class="button" on:click={kill}>Kill</button>
    {#if child}
      <input placeholder="write to stdin" bind:value={stdin}>
      <button class="button" on:click={writeToStdin}>Write</button>
    {/if}
  </div>
  <div>
    <input bind:value={cwd} placeholder="Working directory">
    <input bind:value={env} placeholder="Environment variables" style="width: 300px">
  </div>
</div>
