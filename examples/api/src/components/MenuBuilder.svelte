<script lang="ts">
  import { CheckMenuItem } from '@tauri-apps/api/menu'
  import MenuItemBuilder from './MenuItemBuilder.svelte'
  import type {
    BuiltMenuItem,
    BuiltMenuItemOptions,
    MenuItemClickDetail,
    MenuItemClickHandler
  } from '../types'

  let {
    items = $bindable<BuiltMenuItem[]>([]),
    itemClick
  }: { items?: BuiltMenuItem[]; itemClick: MenuItemClickHandler } = $props()

  function addItem({ item, options }: BuiltMenuItem) {
    items = [...items, { item, options }]
  }

  function onItemClick(detail: MenuItemClickDetail) {
    itemClick(detail)
  }

  function hasOption<K extends keyof BuiltMenuItemOptions>(
    options: BuiltMenuItemOptions,
    key: K
  ): options is BuiltMenuItemOptions & Record<K, unknown> {
    return key in options
  }

  function itemIcon(item: BuiltMenuItem) {
    if (hasOption(item.options, 'icon') && item.options.icon) {
      return 'i-ph-images-square'
    }
    if (item.item instanceof CheckMenuItem) {
      return hasOption(item.options, 'checked') && item.options.checked
        ? 'i-ph-check-duotone'
        : 'i-ph-square-duotone'
    }
    if (hasOption(item.options, 'item') && item.options.item) {
      return 'i-ph-globe-stand'
    }
    return 'i-ph-chat-teardrop-text'
  }

  function itemToString(item: BuiltMenuItem) {
    // icon || check|normal || predefined
    if (hasOption(item.options, 'icon') && item.options.icon) {
      return String(item.options.icon)
    }
    if (hasOption(item.options, 'text') && item.options.text) {
      return item.options.text
    }
    if (hasOption(item.options, 'item') && item.options.item) {
      return String(item.options.item)
    }
    return ''
  }
</script>

<div class="flex flex-col children:grow gap-2">
  <MenuItemBuilder newItem={addItem} itemClick={onItemClick} />

  <div>
    {#each items as item}
      <div class="flex flex-row gap-1 items-center">
        <div class={itemIcon(item)}></div>
        <p>{itemToString(item)}</p>
      </div>
    {/each}
  </div>
</div>
