(function () {
  var observer = new MutationObserver(mutation => {
    mutation.forEach(function (mutationRecord) {
      var addedNodes = mutationRecord.addedNodes
      addedNodes.forEach(function (node) {
        if (node.nodeType === 1 && node.tagName === 'SCRIPT') {
          const src = node.src
          if (src) {
            node.onload = node.onerror = null
            if (window.tauri !== void 0) {
              window.tauri.loadAsset(src)
            } else {
              if (window.__TAURI_INIT_HOOKS === void 0) {
                window.__TAURI_INIT_HOOKS = []
              }
              window.__TAURI_INIT_HOOKS.push(function () {
                window.tauri.loadAsset(src)
              })
            }
          }
        }
      })
    })
  })

  <% if (target === 'body') { %>
    var target = document.documentElement
  <% } else { %>
    var target = document.head
  <% } %>

  observer.observe(target, {
    childList: true,
    subtree: true
  })
})()
