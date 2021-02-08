function __tauri_mutation_observer (target) {
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

  observer.observe(target, {
    childList: true,
    subtree: true
  })
}

__tauri_mutation_observer(document.documentElement)
if (
  document.readyState === 'complete' ||
  document.readyState === 'interactive'
) {
  __tauri_mutation_observer(document.head)
} else {
  window.addEventListener(
    'DOMContentLoaded',
    function () {
      __tauri_mutation_observer(document.head)
    },
    true
  )
}

