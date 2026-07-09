<script lang="ts">
  import type { PredefinedMenuItemOptions } from '@tauri-apps/api/menu'

  type PredefinedItem = PredefinedMenuItemOptions['item']

  const {
    kind,
    text,
    icon,
    check
  }: {
    kind: PredefinedItem | 'Normal' | 'Icon' | 'Check'
    text?: boolean
    icon?: boolean
    check?: boolean
  } = $props()
</script>

<div
  class="flex gap-2 border border-solid border-neutral-200 bg-neutral-100 p-inline-2 p-block-1 rounded-md menu-item"
>
  {kind}
  {#if text}
    <input
      type="text"
      title="text"
      placeholder="text"
      class="border border-solid border-neutral-200 focus-visible:border-accent outline-none p-inline-2 p-block-1 rounded-sm"
    />
  {/if}
  {#if icon}
    <input
      type="text"
      title="icon path"
      placeholder="icon path"
      class="border border-solid border-neutral-200 focus-visible:border-accent outline-none p-inline-2 p-block-1 rounded-sm"
    />
  {/if}
  {#if check}
    <input type="checkbox" title="checked" />
  {/if}
</div>
