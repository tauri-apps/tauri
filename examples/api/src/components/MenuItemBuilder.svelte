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
  import MenuItemComponent, {
    type MenuItemComponentKind
  } from './MenuItemComponent.svelte'

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

  let currentId = 0

  const targetList: {
    id: number
    kind: MenuItemComponentKind
    text: string | undefined
    iconPath: string | undefined
    checked: boolean | undefined
  }[] = $state([])

  $effect(() => {
    $inspect(targetList)
  })

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
      dataIdAttr: 'data-id',
      group: {
        name: 'shared'
      },
      onAdd(event) {
        const item = event.item
        const kind = item.dataset.kind as MenuItemComponentKind
        const text = item.dataset.text
        const iconPath = item.dataset.iconPath
        const checked = item.dataset.checked
        const newItem = {
          id: currentId,
          kind,
          text,
          iconPath,
          checked: checked !== undefined ? checked === 'true' : checked
        }
        currentId += 1
        if (event.newIndex !== undefined) {
          targetList.splice(event.newIndex, 0, newItem)
        } else {
          targetList.push(newItem)
        }
        // HACK: We can't track the element created by Sortable,
        // just make a new one and delete the one from Sortable
        item.remove()
      },
      onUpdate(event) {
        // HACK: Svelte `#each` can't track external changes,
        // and will revert changes made by Sortable,
        // so we store the order and restore it after updating the svelte state
        const order = targetSortable.toArray()
        const [item] = targetList.splice(event.oldIndex!, 1)
        targetList.splice(event.newIndex!, 0, item!)
        targetSortable.sort(order)
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
        <MenuItemComponent kind="Normal" text="" />
        <MenuItemComponent kind="Icon" text="" iconPath="" />
        <MenuItemComponent kind="Check" text="" checked={false} />
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
  >
    {#each targetList as item (item.id)}
      <MenuItemComponent
        kind={item.kind}
        id={item.id}
        bind:text={item.text}
        bind:iconPath={item.iconPath}
        bind:checked={item.checked}
      />
    {/each}
  </div>
  <div class="h-1px bg-neutral/20"></div>
</div>
