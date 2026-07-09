<script lang="ts">
  import {
    IconMenuItem,
    CheckMenuItem,
    PredefinedMenuItem,
    MenuItem
  } from '@tauri-apps/api/menu'
  import type {
    CheckMenuItemOptions,
    IconMenuItemOptions,
    MenuItemOptions,
    PredefinedMenuItemOptions
  } from '@tauri-apps/api/menu'
  import type {
    BuiltMenuItem,
    MenuItemClickHandler
  } from './MenuBuilder.svelte'
  import Sortable from 'sortablejs'
  import { onMount } from 'svelte'
  import MenuItemComponent from './MenuItemComponent.svelte'

  type PredefinedItem = PredefinedMenuItemOptions['item']

  let {
    newItem,
    itemClick
  }: {
    newItem: (item: BuiltMenuItem) => void
    itemClick: MenuItemClickHandler
  } = $props()

  let kind = $state('Normal')
  let text = $state('')
  let icon = $state('')
  let predefinedItem = $state<PredefinedItem>()
  let checked = $state(true)

  const itemKinds = ['Normal', 'Icon', 'Check', 'Predefined']
  const predefinedOptions: PredefinedItem[] = [
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
    'Services',
    'BringAllToFront'
  ]

  async function create() {
    const t = text

    switch (kind) {
      case 'Normal': {
        const options: MenuItemOptions = {
          text,
          action: (id) => itemClick({ id, text: t })
        }
        const item = await MenuItem.new(options)
        newItem({ kind, item, options })
        break
      }
      case 'Icon': {
        const options: IconMenuItemOptions = {
          text,
          icon,
          action: (id) => itemClick({ id, text: t })
        }
        const item = await IconMenuItem.new(options)
        newItem({ kind, item, options })
        break
      }
      case 'Check': {
        const options: CheckMenuItemOptions = {
          text,
          checked,
          action: (id) => itemClick({ id, text: t })
        }
        const item = await CheckMenuItem.new(options)
        newItem({ kind, item, options })
        break
      }
      case 'Predefined': {
        const options: PredefinedMenuItemOptions = {
          item: predefinedItem!
        }
        const item = await PredefinedMenuItem.new(options)
        newItem({ kind, item, options })
        break
      }
    }

    text = ''
    predefinedItem = undefined
  }

  // function reorder<T>(
  //   array: T[],
  //   evt: Sortable.SortableEvent
  // ): $state.Snapshot<T>[] {
  //   // should have no effect on stores or regular array
  //   const workArray = $state.snapshot(array)

  //   // get changes
  //   const { oldIndex, newIndex } = evt

  //   if (oldIndex === undefined || newIndex === undefined) {
  //     return workArray
  //   }
  //   if (newIndex === oldIndex) {
  //     return workArray
  //   }

  //   // move elements
  //   const target = workArray[oldIndex]
  //   const increment = newIndex < oldIndex ? -1 : 1

  //   for (let k = oldIndex; k !== newIndex; k += increment) {
  //     workArray[k] = workArray[k + increment]
  //   }
  //   workArray[newIndex] = target
  //   return workArray
  // }

  let sourceSortableEl1: HTMLElement
  let sourceSortableEl2: HTMLElement
  let targetSortableEl: HTMLElement

  function makeSourceSortable(sourceSortableEl: HTMLElement) {
    return new Sortable(sourceSortableEl, {
      draggable: 'div.menu-item',
      group: {
        name: 'shared',
        pull: 'clone',
        put: false
      },
      sort: false,
      delayOnTouchOnly: true,
      delay: 500
    })
  }

  onMount(() => {
    const sourceSortable1 = makeSourceSortable(sourceSortableEl1)
    const sourceSortable2 = makeSourceSortable(sourceSortableEl2)
    const targetSortable = new Sortable(targetSortableEl, {
      group: {
        name: 'shared'
      },
      delayOnTouchOnly: true,
      delay: 500,
      animation: 200
    })
    return () => {
      targetSortable.destroy()
      sourceSortable1.destroy()
      sourceSortable2.destroy()
    }
  })
</script>

<div class="grid gap-4">
  <div class="grid gap-2">
    <div>
      <h3>Custom</h3>
      <div class="flex flex-wrap gap-2" bind:this={sourceSortableEl1}>
        <MenuItemComponent kind="Normal" text />
        <MenuItemComponent kind="Icon" text icon />
        <MenuItemComponent kind="Check" text check />
      </div>
    </div>
    <div>
      <h3>Predefined</h3>
      <div class="flex flex-wrap gap-2 max-w-5xl" bind:this={sourceSortableEl2}>
        {#each predefinedOptions as predefinedOption}
          <MenuItemComponent kind={predefinedOption} />
        {/each}
      </div>
    </div>
  </div>
  <div class="h-1px bg-neutral/20"></div>
  <div
    bind:this={targetSortableEl}
    class="p-2 border border-solid border-neutral-300 rounded-md max-w-lg min-h-24 grid items-start gap-1"
  ></div>
  <div class="h-1px bg-neutral/20"></div>
</div>

<!-- <div class="flex flex-row gap-2 flex-grow-0">
  <div class="flex flex-col">
    {#each itemKinds as itemKind, i}
      <div class="flex gap-1">
        <input
          id="{itemKind}Input"
          checked={kind === itemKind}
          onchange={() => (kind = itemKind)}
          type="radio"
          name="kind"
          value={itemKinds[i]}
        />
        <label for="{itemKind}Input">{itemKind}</label>
      </div>
    {/each}
  </div>

  <div class="bg-gray/30 dark:bg-white/5 w-1px flex-shrink-0"></div>

  <div class="flex flex-col gap-2">
    {#if kind == 'Normal' || kind == 'Icon' || kind == 'Check'}
      <input class="input" type="text" placeholder="Text" bind:value={text} />
    {/if}
    {#if kind == 'Icon'}
      <input class="input" type="icon" placeholder="Icon" bind:value={icon} />
    {:else if kind == 'Check'}
      <div class="flex gap-1">
        <input
          id="checkItemCheckedInput"
          type="checkbox"
          class="checkbox"
          bind:checked
        />
        <label for="checkItemCheckedInput">Enabled</label>
      </div>
    {:else if kind == 'Predefined'}
      <div class="flex gap-2 flex-wrap">
        {#each predefinedOptions as predefinedOption, i}
          <div class="flex gap-1">
            <input
              id="{predefinedOption}Input"
              checked={predefinedItem === predefinedOption}
              onchange={() => (predefinedItem = predefinedOption)}
              type="radio"
              name="predefinedKind"
              value={predefinedOptions[i]}
            />
            <label for="{predefinedOption}Input">{predefinedOption}</label>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <div class="grow"></div>

  <div class="flex flex-col">
    <button class="btn" onclick={create}>Create</button>
  </div>
</div> -->
