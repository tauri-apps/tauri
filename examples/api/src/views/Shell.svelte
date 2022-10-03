<script>
  import { Command } from '@tauri-apps/api/shell'

  const windows = navigator.userAgent.includes('Windows')
  let cmd = windows ? 'cmd' : 'sh'
  let args = windows ? ['/C'] : ['-c']

  export let onMessage

  let script = 'echo "hello world"'
  let cwd = null
  let env = 'SOMETHING=value ANOTHER=2'
  let encoding = ''
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
    const command = new Command(cmd, [...args, script], {
      cwd: cwd || null,
      env: _getEnv(),
      encoding,
    })

    command.on('close', (data) => {
      onMessage(
        `command finished with code ${data.code} and signal ${data.signal}`
      )
      child = null
    })
    command.on('error', (error) => onMessage(`command error: "${error}"`))

    command.stdout.on('data', (line) => onMessage(`command stdout: "${line}"`))
    command.stderr.on('data', (line) => onMessage(`command stderr: "${line}"`))

    command
      .spawn()
      .then((c) => {
        child = c
      })
      .catch(onMessage)
  }

  function kill() {
    child
      .kill()
      .then(() => onMessage('killed child process'))
      .catch(onMessage)
  }

  function writeToStdin() {
    child.write(stdin).catch(onMessage)
  }
</script>

<div class="flex flex-col childre:grow gap-1">
  <div class="flex items-center gap-1">
    Script:
    <input class="grow input" bind:value={script} />
  </div>
  <div class="flex items-center gap-1">
    Encoding:
    <input class="grow input" bind:value={encoding} />
  </div>
  <div class="flex items-center gap-1">
    Working directory:
    <input
      class="grow input"
      bind:value={cwd}
      placeholder="Working directory"
    />
  </div>
  <div class="flex items-center gap-1">
    Arguments:
    <input
      class="grow input"
      bind:value={env}
      placeholder="Environment variables"
    />
  </div>
  <div class="flex children:grow gap-1">
    <button class="btn" on:click={spawn}>Run</button>
    <button class="btn" on:click={kill}>Kill</button>
  </div>
  {#if child}
    <br />
    <input class="input" placeholder="write to stdin" bind:value={stdin} />
    <button class="btn" on:click={writeToStdin}>Write</button>
  {/if}
</div>
