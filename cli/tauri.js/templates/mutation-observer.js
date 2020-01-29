(function () {
  function loadAsset(path, type) {
    if (path) {
      if (window.tauri !== void 0) {
        window.tauri.loadAsset(path, type)
      } else {
        if (window.__TAURI_INIT_HOOKS === void 0) {
          window.__TAURI_INIT_HOOKS = []
        }
        window.__TAURI_INIT_HOOKS.push(function () {
          window.tauri.loadAsset(path, type)
        })
      }
    }
  }

  var observer = new MutationObserver(mutation => {
    mutation.forEach(function (mutationRecord) {
      var addedNodes = mutationRecord.addedNodes
      addedNodes.forEach(function (node) {
        if (node.nodeType === 1) {
          if (node.tagName === 'SCRIPT') {
            node.onload = node.onerror = null
            loadAsset(node.src)
          } else if (node.tagName === 'LINK') {
            if (node.type === 'text/css' || (node.href && node.href.endsWith('.css'))) {
              loadAsset(node.href, 'stylesheet')
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
