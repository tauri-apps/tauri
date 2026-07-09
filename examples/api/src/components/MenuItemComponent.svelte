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
    id,
    kind,
    text = $bindable(),
    iconPath = $bindable(),
    checked = $bindable(),
    onTextChange,
    onIconPathChange,
    onCheckedChange,
    onRemove
  }: {
    id?: number
    kind: MenuItemComponentKind
    text?: string
    iconPath?: string
    checked?: boolean
    onTextChange?: () => void
    onIconPathChange?: () => void
    onCheckedChange?: () => void
    onRemove?: () => void
  } = $props()
</script>

<div
  class="flex items-center gap-2 border border-solid border-neutral-200 bg-neutral-100 p-inline-2 p-block-1 rounded-md menu-item"
  data-kind={kind}
  data-text={text}
  data-icon-path={iconPath}
  data-checked={checked}
  data-id={id}
>
  {kind}
  {#if text !== undefined}
    <input
      type="text"
      title="text"
      placeholder="text"
      class="border border-solid border-neutral-200 focus-visible:border-accent outline-none p-inline-2 p-block-1 rounded-sm"
      bind:value={text}
      onchange={onTextChange}
    />
  {/if}
  {#if iconPath !== undefined}
    <input
      type="text"
      title="icon path"
      placeholder="icon path"
      class="border border-solid border-neutral-200 focus-visible:border-accent outline-none p-inline-2 p-block-1 rounded-sm"
      bind:value={iconPath}
      onchange={onIconPathChange}
    />
  {/if}
  {#if checked !== undefined}
    <input
      type="checkbox"
      title="checked"
      bind:checked
      onchange={onCheckedChange}
    />
  {/if}

  {#if onRemove !== undefined}
    <button onclick={onRemove} title="remove" class="btn m-l-auto">
      <div class="i-ph-trash"></div>
    </button>
  {/if}
</div>
