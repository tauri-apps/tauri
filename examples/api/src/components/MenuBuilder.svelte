<script module lang="ts">
  import type {
    CheckMenuItem,
    CheckMenuItemOptions,
    IconMenuItem,
    IconMenuItemOptions,
    MenuItem,
    MenuItemOptions,
    PredefinedMenuItem,
    PredefinedMenuItemOptions
  } from '@tauri-apps/api/menu'

  export type BuiltMenuItem =
    | {
        kind: 'Normal'
        item: MenuItem
        options: MenuItemOptions
      }
    | {
        kind: 'Icon'
        item: IconMenuItem
        options: IconMenuItemOptions
      }
    | {
        kind: 'Check'
        item: CheckMenuItem
        options: CheckMenuItemOptions
      }
    | {
        kind: 'Predefined'
        item: PredefinedMenuItem
        options: PredefinedMenuItemOptions
      }

  export type MenuItemClickDetail = {
    id: string
    text: string
  }
  export type MenuItemClickHandler = (detail: MenuItemClickDetail) => void
</script>

<script lang="ts">
  import MenuItemBuilder from './MenuItemBuilder.svelte'

  let {
    items = $bindable<BuiltMenuItem[]>([]),
    itemClick
  }: { items?: BuiltMenuItem[]; itemClick: MenuItemClickHandler } = $props()

  function addItem(newItem: BuiltMenuItem) {
    items = [...items, newItem]
  }

  function itemIcon(item: BuiltMenuItem) {
    if (item.kind === 'Icon' && item.options.icon) {
      return 'i-ph-images-square'
    }
    if (item.kind === 'Check') {
      return item.options.checked ? 'i-ph-check-duotone' : 'i-ph-square-duotone'
    }
    if (item.kind === 'Predefined' && item.options.item) {
      return 'i-ph-globe-stand'
    }
    return 'i-ph-chat-teardrop-text'
  }

  function itemToString(item: BuiltMenuItem) {
    return (
      // icon
      ('icon' in item.options && item.options.icon)
      // check|normal
      || ('text' in item.options && item.options.text)
      // predefined
      || ('item' in item.options && item.options.item)
    )
  }
</script>

<div class="flex flex-col children:grow gap-2">
  <MenuItemBuilder newItem={addItem} {itemClick} />

  <div>
    {#each items as item}
      <div class="flex flex-row gap-1 items-center">
        <div class={itemIcon(item)}></div>
        <p>{itemToString(item)}</p>
      </div>
    {/each}
  </div>
</div>
