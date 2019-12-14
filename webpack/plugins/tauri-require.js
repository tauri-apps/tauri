module.exports.plugin = class TauriRequirePlugin {
  apply (compiler) {
    compiler.plugin('compilation', function (compilation) {
      compilation.mainTemplate.hooks.requireEnsure.tap('JsonpMainTemplatePlugin load', (source, chunk, hash) => {
        return `
          // Tauri chunk loading

          var installedChunkData = installedChunks[chunkId];
          if(installedChunkData !== 0) { // 0 means "already installed".

            // a Promise means "currently loading".
            if(installedChunkData) {
              promises.push(installedChunkData[2]);
            } else {
                // setup Promise in chunk cache
                var promise = new Promise(function(resolve, reject) {
                  installedChunkData = installedChunks[chunkId] = [resolve, reject];
                });
                promises.push(installedChunkData[2] = promise);

                var onScriptComplete;
                // create error before stack unwound to get useful stacktrace later
                var error = new Error();

                onScriptComplete = function (event) {
                  clearTimeout(timeout);
                  var chunk = installedChunks[chunkId];
                  if(chunk !== 0) {
                    if(chunk) {
                      var errorType = event && (event.type === 'load' ? 'missing' : event.type);
                      var realSrc = event && event.target && event.target.src;
                      error.message = 'Loading chunk ' + chunkId + ' failed.';
                      error.name = 'ChunkLoadError';
                      error.type = errorType;
                      error.request = realSrc;
                      chunk[1](error);
                    }
                    installedChunks[chunkId] = undefined;
                  }
                };
                var timeout = setTimeout(function(){
                  onScriptComplete({ type: 'timeout' });
                }, 120000);

                // start chunk loading
                window.tauri.loadAsset(jsonpScriptSrc(chunkId)).then(function () {
                  setTimeout(function () {
                    // onScriptComplete({ type: 'load' })
                  }, 2000)
                }).catch(function () {
                  onScriptComplete({type: 'error' })
                });
              }
            }
        `
      })
    })
  }
}
