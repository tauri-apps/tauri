// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// qtbrowser://downloads: the list comes through a channel (the `watch_downloads` command sends it
// now and after every change). Images get a thumbnail through the asset protocol.
const { invoke, Channel, convertFileSrc } = window.__TAURI__.core
const list = document.getElementById('list')
const images = /\.(png|jpe?g|gif|webp|svg|avif|bmp|ico)$/i

invoke('get_settings').then(({ settings }) => {
  if (settings.theme === 'light' || settings.theme === 'dark')
    document.documentElement.dataset.theme = settings.theme
})

const button = (text, action, id) => {
  const b = document.createElement('button')
  b.className = 'button'
  b.textContent = text
  b.addEventListener('click', () => invoke('download_action', { id, action }))
  return b
}

function render(items) {
  document.getElementById('empty').hidden = items.length > 0
  list.replaceChildren(
    ...items.map((d) => {
      const li = document.createElement('li')
      const name = d.path.split('/').pop()
      const thumb = document.createElement('div')
      thumb.className = 'thumb'
      if (d.state === 'completed' && images.test(name)) {
        const img = new Image()
        img.src = convertFileSrc(d.path) // asset protocol
        img.alt = ''
        thumb.append(img)
      } else {
        thumb.textContent = (name.split('.').pop() || '?')
          .slice(0, 4)
          .toUpperCase()
      }
      const info = document.createElement('div')
      info.className = 'info'
      const title = document.createElement('strong')
      title.textContent = name
      const meta = document.createElement('small')
      const when = new Date(d.started).toLocaleString()
      meta.textContent = `${{ downloading: 'Downloading…', completed: 'Done', failed: 'Failed' }[d.state]} · ${when} · ${d.url}`
      info.append(title, meta)
      const actions = document.createElement('div')
      actions.className = 'actions'
      if (d.state === 'completed')
        actions.append(
          button('Open', 'open', d.id),
          button('Show in folder', 'show', d.id)
        )
      actions.append(button('Remove', 'remove', d.id))
      li.append(thumb, info, actions)
      return li
    })
  )
}

const channel = new Channel()
channel.onmessage = render
invoke('watch_downloads', { channel })
document
  .getElementById('clear')
  .addEventListener('click', () =>
    invoke('download_action', { action: 'clear' })
  )
