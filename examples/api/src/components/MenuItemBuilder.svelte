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
  let checked = true

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

    const t = text

    switch (kind) {
      case 'Normal':
        options = {
          text,
          action: (id) => dispatch('itemClick', { id, text: t })
        }
        item = await MenuItem.new(options)
        break
      case 'Icon':
        options = {
          text,
          icon,
          action: (id) => dispatch('itemClick', { id, text: t })
        }
        item = await IconMenuItem.new(options)
        break
      case 'Check':
        options = {
          text,
          checked,
          action: (id) => dispatch('itemClick', { id, text: t })
        }
        item = await CheckMenuItem.new(options)
        break
      case 'Predefined':
        options = {
          item: predefinedItem
        }
        item = await PredefinedMenuItem.new(options)
        break
    }
    dispatch('new', { item, options })

    text = ''
    predefinedItem = ''
  }
</script>

<div class="flex flex-row gap-2 flex-grow-0">
  <div class="flex flex-col">
    {#each itemKinds as itemKind}
      <div class="flex gap-1">
        <input
            id="{itemKind}Input"
            checked={kind === itemKind}
            on:change={onKindChange}
            type="radio"
            name="kind"
            bind:value={itemKind}
          />
        <label for="{itemKind}Input">{itemKind}</label>
      </div>
    {/each}
  </div>

  <div class="bg-gray/30 dark:bg-white/5 w-1px flex-shrink-0" />

  <div class="flex flex-col gap-2">
    {#if kind == 'Normal' || kind == 'Icon' || kind == 'Check'}
      <input
        class="input"
        type="text"
        placeholder="Text"
        bind:value={text}
      />
    {/if}
    {#if kind == 'Icon'}
      <input
        class="input"
        type="icon"
        placeholder="Icon"
        bind:value={icon}
      />
    {:else if kind == 'Check'}
      <div class="flex gap-1">
        <input
            id="checkItemCheckedInput"
            type="checkbox"
            class="checkbox"
            bind:checked={checked}
          />
        <label for="checkItemCheckedInput">Enabled</label>
      </div>
    {:else if kind == 'Predefined'}
      <div class="flex gap-2 flex-wrap">
        {#each predefinedOptions as predefinedOption}
          <div class="flex gap-1">
            <input
                id="{predefinedOption}Input"
                checked={kind === predefinedOption}
                on:change={onPredefinedChange}
                type="radio"
                name="predefinedKind"
                bind:value={predefinedOption}
              />
            <label for="{predefinedOption}Input">{predefinedOption}</label>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <div class="grow"></div>

  <div class="flex flex-col">
    <button class="btn" on:click={create}>Create</button>
  </div>
</div>
