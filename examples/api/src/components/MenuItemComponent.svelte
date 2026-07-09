<script module lang="ts">
  import type { PredefinedMenuItemOptions } from '@tauri-apps/api/menu'

  type PredefinedItem = PredefinedMenuItemOptions['item']
  export type MenuItemComponentKind =
    | PredefinedItem
    | 'Normal'
    | 'Icon'
    | 'Check'
</script>

<script lang="ts">
  let {
    kind,
    text = $bindable(),
    iconPath = $bindable(),
    checked = $bindable()
  }: {
    kind: MenuItemComponentKind
    text?: string
    iconPath?: string
    checked?: boolean
  } = $props()
</script>

<div
  class="flex gap-2 border border-solid border-neutral-200 bg-neutral-100 p-inline-2 p-block-1 rounded-md menu-item"
  data-kind={kind}
  data-text={text}
  data-icon-path={iconPath}
  data-checked={checked}
>
  {kind}
  {#if text !== undefined}
    <input
      type="text"
      title="text"
      placeholder="text"
      class="border border-solid border-neutral-200 focus-visible:border-accent outline-none p-inline-2 p-block-1 rounded-sm"
      bind:value={text}
    />
  {/if}
  {#if iconPath !== undefined}
    <input
      type="text"
      title="icon path"
      placeholder="icon path"
      class="border border-solid border-neutral-200 focus-visible:border-accent outline-none p-inline-2 p-block-1 rounded-sm"
      bind:value={iconPath}
    />
  {/if}
  {#if checked !== undefined}
    <input type="checkbox" title="checked" bind:checked />
  {/if}
</div>
