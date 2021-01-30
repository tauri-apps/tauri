(function () {
  function loadAsset(path, type) {
    if (path) {
      window.__TAURI__.loadAsset(path, type)
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

  {{#if (eq target "body")}}
    var target = document.documentElement
  {{ else }}
    var target = document.head
  {{/if}}

  observer.observe(target, {
    childList: true,
    subtree: true
  })
})()
