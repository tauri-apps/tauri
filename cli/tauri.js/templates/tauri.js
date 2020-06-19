// polyfills
if (!String.prototype.startsWith) {
  String.prototype.startsWith = function (searchString, position) {
    position = position || 0
    return this.substr(position, searchString.length) === searchString
  }
}

// makes the window.external.invoke API available after window.location.href changes
switch (navigator.platform) {
  case "Macintosh":
  case "MacPPC":
  case "MacIntel":
  case "Mac68K":
    window.external = this
    invoke = function (x) {
      webkit.messageHandlers.invoke.postMessage(x);
    }
    break;
  case "Windows":
  case "WinCE":
  case "Win32":
  case "Win64":
    break;
  default:
    window.external = this
    invoke = function (x) {
      window.webkit.messageHandlers.external.postMessage(x);
    }
    break;
}

(function () {
  function s4() {
    return Math.floor((1 + Math.random()) * 0x10000)
      .toString(16)
      .substring(1)
  }

  var uid = function () {
    return s4() + s4() + '-' + s4() + '-' + s4() + '-' +
      s4() + '-' + s4() + s4() + s4()
  }

  function ownKeys(object, enumerableOnly) {
    var keys = Object.keys(object);
    if (Object.getOwnPropertySymbols) {
      var symbols = Object.getOwnPropertySymbols(object);
      if (enumerableOnly) symbols = symbols.filter(function (sym) {
        return Object.getOwnPropertyDescriptor(object, sym).enumerable;
      });
      keys.push.apply(keys, symbols);
    }
    return keys;
  }

  function _objectSpread(target) {
    for (var i = 1; i < arguments.length; i++) {
      var source = arguments[i] != null ? arguments[i] : {};
      if (i % 2) {
        ownKeys(source, true).forEach(function (key) {
          _defineProperty(target, key, source[key]);
        });
      } else if (Object.getOwnPropertyDescriptors) {
        Object.defineProperties(target, Object.getOwnPropertyDescriptors(source));
      } else {
        ownKeys(source).forEach(function (key) {
          Object.defineProperty(target, key, Object.getOwnPropertyDescriptor(source, key));
        });
      }
    }
    return target;
  }

  function _defineProperty(obj, key, value) {
    if (key in obj) {
      Object.defineProperty(obj, key, {
        value: value,
        enumerable: true,
        configurable: true,
        writable: true
      });
    } else {
      obj[key] = value;
    }
    return obj;
  }

  if (!window.__TAURI__) {
    window.__TAURI__ = {}
  }
  window.__TAURI__.invoke = function invoke(args) {
    window.external.invoke(JSON.stringify(args))
  }

  window.__TAURI__.transformCallback = function transformCallback(callback) {
    var once = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : false
    var identifier = uid()

    window[identifier] = function (result) {
      if (once) {
        delete window[identifier]
      }

      return callback && callback(result)
    }

    return identifier;
  }

  window.__TAURI__.promisified = function promisified(args) {
    var _this = this;

    return new Promise(function (resolve, reject) {
      _this.invoke(_objectSpread({
        callback: _this.transformCallback(resolve),
        error: _this.transformCallback(reject)
      }, args))
    })
  }

  window.__TAURI__.loadAsset = function loadAsset(assetName, assetType) {
    return this.promisified({
      cmd: 'loadAsset',
      asset: assetName,
      assetType: assetType || 'unknown'
    })
  }

  // init tauri API
  try {
    window.__TAURI__.invoke({
      cmd: 'init'
    })
  } catch (e) {
    window.addEventListener('DOMContentLoaded', function () {
      window.__TAURI__.invoke({
        cmd: 'init'
      })
    }, true)
  }

  document.addEventListener('error', function (e) {
    var target = e.target
    while (target != null) {
      if (target.matches ? target.matches('img') : target.msMatchesSelector('img')) {
        window.__TAURI__.loadAsset(target.src, 'image')
          .then(function (img) {
            target.src = img
          })
        break
      }
      target = target.parentElement
    }
  }, true)

  // open <a href="..."> links with the Tauri API
  function __openLinks() {
    document.querySelector('body').addEventListener('click', function (e) {
      var target = e.target
      while (target != null) {
        if (target.matches ? target.matches('a') : target.msMatchesSelector('a')) {
          if (target.href && target.href.startsWith('http') && target.target === '_blank') {
            window.__TAURI__.invoke({
              cmd: 'open',
              uri: target.href
            })
            e.preventDefault()
          }
          break
        }
        target = target.parentElement
      }
    }, true)
  }

  if (document.readyState === 'complete' || document.readyState === 'interactive') {
    __openLinks()
  } else {
    window.addEventListener('DOMContentLoaded', function () {
      __openLinks()
    }, true)
  }
})()
