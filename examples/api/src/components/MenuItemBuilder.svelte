<script>
  import {
    IconMenuItem,
    CheckMenuItem,
    PredefinedMenuItem,
    MenuItem
  } from '@tauri-apps/api/menu'
  import { createEventDispatcher } from 'svelte'

  const dispatch = createEventDispatcher()

  let kind = 'Normal'
  let text = ''
  let icon = ''
  let predefinedItem = ''

  const itemKinds = ['Normal', 'Icon', 'Check', 'Predefined']
  const predefinedOptions = [
    'Separator',
    'Copy',
    'Cut',
    'Paste',
    'SelectAll',
    'Undo',
    'Redo',
    'Minimize',
    'Maximize',
    'Fullscreen',
    'Hide',
    'HideOthers',
    'ShowAll',
    'CloseWindow',
    'Quit',
    'Services'
  ]

  function onKindChange(event) {
    kind = event.currentTarget.value
  }

  function onPredefinedChange(event) {
    predefinedItem = event.currentTarget.value
  }

  async function create() {
    let options = null
    let item = null
    switch (kind) {
      case 'Normal':
        options = { text }
        item = await MenuItem.new(options)
        break
      case 'Icon':
        options = { text, icon }
        item = await IconMenuItem.new(options)
        break
      case 'Check':
        options = { text }
        item = await CheckMenuItem.new(options)
        break
      case 'Predefined':
        options = { item: predefinedItem }
        item = await PredefinedMenuItem.new(options)
        break
    }
    dispatch('new', { item, options })

    text = ''
    predefinedItem = ''
  }
</script>

<div class="flex flex-row children:grow gap-2 items-center justify-between">
  <div class="flex flex-col" style="max-width: 160px">
    {#each itemKinds as itemKind}
      <label>
        <input
          checked={kind === itemKind}
          on:change={onKindChange}
          type="radio"
          name="kind"
          bind:value={itemKind}
        />
        {itemKind}
      </label>
    {/each}
  </div>

  <div class="flex flex-col" style="max-height: 40px">
    {#if kind == 'Normal' || kind == 'Icon' || kind == 'Check'}
      <input
        class="input grow"
        type="text"
        placeholder="Text"
        bind:value={text}
      />
    {/if}
    {#if kind == 'Icon'}
      <input
        class="input grow"
        type="icon"
        placeholder="Icon"
        bind:value={icon}
      />
    {:else if kind == 'Predefined'}
      <div class="flex flex-col flex-wrap" style="max-height: 40px">
        {#each predefinedOptions as predefinedOption}
          <label>
            <input
              checked={kind === predefinedOption}
              on:change={onPredefinedChange}
              type="radio"
              name="predefinedKind"
              bind:value={predefinedOption}
            />
            {predefinedOption}
          </label>
        {/each}
      </div>
    {/if}
  </div>

  <div class="flex flex-col items-end">
    <button class="btn" on:click={create}>Create</button>
  </div>
</div>
