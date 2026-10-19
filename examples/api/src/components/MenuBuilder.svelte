<script module lang="ts">
  import {
    CheckMenuItem,
    IconMenuItem,
    Menu,
    MenuItem,
    PredefinedMenuItem,
    Submenu,
    type PredefinedMenuItemOptions
  } from '@tauri-apps/api/menu'

  export type MenuOptions = {
    id: number
    kind: MenuItemComponentKind
    text: string | undefined
    iconPath: string | undefined
    checked: boolean | undefined
  }

  export type Item = MenuOptions & {
    menu?: MenuItems
  }

  export type MenuItems =
    | MenuItem
    | IconMenuItem
    | CheckMenuItem
    | PredefinedMenuItem

  export type MenuItemClickDetail = {
    id: string
    text: string
  }
  export type MenuItemClickHandler = (detail: MenuItemClickDetail) => void

  export async function reorderMenuItems(
    menu: Menu | Submenu,
    reorderItem: Item,
    toIndex: number
  ) {
    if (!reorderItem.menu) {
      return
    }
    await menu.remove(reorderItem.menu)
    menu.insert(reorderItem.menu, toIndex)
  }
</script>

<script lang="ts">
  import Sortable from 'sortablejs'
  import { onDestroy, onMount } from 'svelte'
  import MenuItemComponent, {
    type MenuItemComponentKind
  } from './MenuItemComponent.svelte'

  type PredefinedItem = PredefinedMenuItemOptions['item']

  let {
    items = $bindable(),
    itemClick,
    onItemAdded,
    onItemRemoved,
    onItemMoved
  }: {
    items: Item[]
    itemClick: MenuItemClickHandler
    onItemAdded?: (item: Item) => Promise<void>
    onItemRemoved?: (item: Item) => Promise<void>
    onItemMoved?: (item: Item, toIndex: number) => void
  } = $props()

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

  async function create(options: MenuOptions) {
    let { kind, text, iconPath, checked } = options
    text ??= ''

    switch (kind) {
      case 'Normal': {
        return await MenuItem.new({
          text,
          action: (id) => {
            const item = items.find((item) => item.id === options.id)
            itemClick({ id, text: item?.text ?? text })
          }
        })
      }
      case 'Icon': {
        return await IconMenuItem.new({
          text,
          icon: iconPath,
          action: (id) => {
            const item = items.find((item) => item.id === options.id)
            itemClick({ id, text: item?.text ?? text })
          }
        })
      }
      case 'Check': {
        const checkItem = await CheckMenuItem.new({
          text,
          checked,
          action: async (id) => {
            const item = items.find((item) => item.id === options.id)
            itemClick({ id, text: item?.text ?? text })
            if (item) {
              item.checked = await checkItem.isChecked()
            }
          }
        })
        return checkItem
      }
      default: {
        return await PredefinedMenuItem.new({
          item: kind
        })
      }
    }
  }

  let currentId = 0

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
      async onAdd(event) {
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
        } as Item
        currentId += 1

        // HACK: We can't track the element created by Sortable,
        // just make a new one and delete the one from Sortable
        item.remove()

        if (event.newIndex !== undefined) {
          items.splice(event.newIndex, 0, newItem)
        } else {
          items.push(newItem)
        }

        newItem.menu = await create(newItem)

        await onItemAdded?.(newItem)
      },
      onUpdate(event) {
        const [item] = items.splice(event.oldIndex!, 1)
        // Sortable's DOM manipulation doesn't work with Svelte,
        // so we recreate all the nodes on order changes...
        // This must be `toSpliced` to trigger the `#key` block re-render
        items = items.toSpliced(event.newIndex!, 0, item!)

        onItemMoved?.(item, event.newIndex!)
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

  onDestroy(() => {
    for (const item of items) {
      item.menu?.close()
    }
    items = []
  })
</script>

<div class="grid gap-4 max-w-5xl">
  <div class="grid gap-2">
    <div>
      <h3>Custom</h3>
      <div class="flex flex-wrap gap-2" bind:this={sourceSortableEl1}>
        <MenuItemComponent kind="Normal" text="A menu item" />
        <MenuItemComponent
          kind="Icon"
          text="Menu item with icon"
          iconPath="../../.icons/icon.png"
        />
        <MenuItemComponent kind="Check" text="A check menu" checked={false} />
      </div>
    </div>
    <div>
      <h3>Predefined</h3>
      <div class="flex flex-wrap gap-2" bind:this={sourceSortableEl2}>
        {#each predefinedOptions as predefinedOption}
          <MenuItemComponent kind={predefinedOption} />
        {/each}
      </div>
    </div>
  </div>
  <div class="h-1px bg-neutral/20 m-inline-4"></div>
  <div
    bind:this={targetSortableEl}
    class="p-2 border border-solid border-neutral-300 dark:border-neutral-700 rounded-md min-h-24 grid items-start gap-1"
  >
    {#key items}
      {#each items as item, i (item.id)}
        <MenuItemComponent
          kind={item.kind}
          id={item.id}
          bind:text={item.text}
          bind:iconPath={item.iconPath}
          bind:checked={item.checked}
          onTextChange={item.text !== undefined
            ? () => {
                item.menu?.setText(item.text ?? '')
              }
            : undefined}
          onIconPathChange={item.iconPath !== undefined
            ? () => {
                ;(item.menu as IconMenuItem | undefined)?.setIcon(
                  item.iconPath!
                )
              }
            : undefined}
          onCheckedChange={item.checked !== undefined
            ? () => {
                ;(item.menu as CheckMenuItem | undefined)?.setChecked(
                  item.checked!
                )
              }
            : undefined}
          onRemove={async () => {
            items.splice(i, 1)
            await onItemRemoved?.(item).finally(() => {
              item.menu?.close()
            })
          }}
        />
      {:else}
        <div class="place-self-center color-secondaryText">
          Drag and drop the menu items above to start building
        </div>
      {/each}
    {/key}
  </div>
</div>
