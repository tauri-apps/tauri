/* eslint-disable */

/**
 *  * THIS FILE IS GENERATED AUTOMATICALLY.
 * DO NOT EDIT.
 *
 * Please whitelist these API functions in tauri.conf.json
 *
 **/

/**
 * @module tauri
 * @description This API interface makes powerful interactions available
 * to be run on client side applications. They are opt-in features, and
 * must be enabled in tauri.conf.json
 *
 * Each binding MUST provide these interfaces in order to be compliant,
 * and also whitelist them based upon the developer's settings.
 */

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


function s4() {
  return Math.floor((1 + Math.random()) * 0x10000)
    .toString(16)
    .substring(1)
}

var uid = function () {
  return s4() + s4() + '-' + s4() + '-' + s4() + '-' +
    s4() + '-' + s4() + s4() + s4()
}

function ownKeys(object, enumerableOnly) { var keys = Object.keys(object); if (Object.getOwnPropertySymbols) { var symbols = Object.getOwnPropertySymbols(object); if (enumerableOnly) symbols = symbols.filter(function (sym) { return Object.getOwnPropertyDescriptor(object, sym).enumerable; }); keys.push.apply(keys, symbols); } return keys; }

function _objectSpread(target) { for (var i = 1; i < arguments.length; i++) { var source = arguments[i] != null ? arguments[i] : {}; if (i % 2) { ownKeys(source, true).forEach(function (key) { _defineProperty(target, key, source[key]); }); } else if (Object.getOwnPropertyDescriptors) { Object.defineProperties(target, Object.getOwnPropertyDescriptors(source)); } else { ownKeys(source).forEach(function (key) { Object.defineProperty(target, key, Object.getOwnPropertyDescriptor(source, key)); }); } } return target; }

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }


function _typeof(obj) { if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof = function _typeof(obj) { return typeof obj; }; } else { _typeof = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof(obj); }

/**
 * @typedef {number} BaseDirectory
 */
/**
 * @enum {BaseDirectory}
 */
var Dir = {
  Audio: 1,
  Cache: 2,
  Config: 3, 
  Data: 4,
  LocalData: 5,
  Desktop: 6,
  Document: 7,
  Download: 8,
  Executable: 9,
  Font: 10,
  Home: 11,
  Picture: 12,
  Public: 13,
  Runtime: 14,
  Template: 15,
  Video: 16,
  Resource: 17,
  App: 18
}




var __reject = function () {
  return new Promise(function (_, reject) {
    reject();
  });
}

window.tauri = {
  Dir: Dir,
  
  invoke: function invoke(args) {
    window.external.invoke(JSON.stringify(args));
  },

  
  listen: function listen(event, handler) {
    
    var once = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : false;
      this.invoke({
        cmd: 'listen',
        event: event,
        handler: window.tauri.transformCallback(handler, once),
        once: once
      });
    
  },

  
  emit: function emit(evt, payload) {
    
      this.invoke({
        cmd: 'emit',
        event: evt,
        payload: payload
      });
    
  },

  
  transformCallback: function transformCallback(callback) {
    var once = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : false;
    var identifier = uid();

    window[identifier] = function (result) {
      if (once) {
        delete window[identifier];
      }

      return callback && callback(result);
    };

    return identifier;
  },

  
  promisified: function promisified(args) {
    var _this = this;

    return new Promise(function (resolve, reject) {
      _this.invoke(_objectSpread({
        callback: _this.transformCallback(resolve),
        error: _this.transformCallback(reject)
      }, args));
    });
  },

  
  readTextFile: function readTextFile(path, options) {
    
      return this.promisified({
        cmd: 'readTextFile',
        path: path,
        options: options
      });
    
  },

  
  readBinaryFile: function readBinaryFile(path, options) {
    
      return this.promisified({
        cmd: 'readBinaryFile',
        path: path,
        options: options
      });
    
  },

  
  writeFile: function writeFile(cfg, options) {
    
      if (_typeof(cfg) === 'object') {
        Object.freeze(cfg);
      }
      return this.promisified({
        cmd: 'writeFile',
        file: cfg.file,
        contents: cfg.contents,
        options: options
      });
    
  },

  
  readDir: function readDir(path, options) {
    
      return this.promisified({
        cmd: 'readDir',
        path: path,
        options: options
      });
    
  },

  
  createDir: function createDir(path, options) {
    
      return this.promisified({
        cmd: 'createDir',
        path: path,
        options: options
      });
    
  },

  
  removeDir: function removeDir(path, options) {
    
      return this.promisified({
        cmd: 'removeDir',
        path: path,
        options: options
      });
    
  },

  
  copyFile: function copyFile(source, destination, options) {
    
      return this.promisified({
        cmd: 'copyFile',
        source: source,
        destination: destination,
        options: options
      });
    
  },

  
  removeFile: function removeFile(path, options) {
    
      return this.promisified({
        cmd: 'removeFile',
        path: path,
        options: options
      });
    
  },

  
  renameFile: function renameFile(oldPath, newPath, options) {
    
      return this.promisified({
        cmd: 'renameFile',
        old_path: oldPath,
        new_path: newPath,
        options: options
      });
    
  },

  
  setTitle: function setTitle(title) {
    
      this.invoke({
        cmd: 'setTitle',
        title: title
      });
    
  },

  
  open: function open(uri) {
    
      this.invoke({
        cmd: 'open',
        uri: uri
      });
    
  },

  
  execute: function execute(command, args) {
    

      if (_typeof(args) === 'object') {
        Object.freeze(args);
      }

      return this.promisified({
        cmd: 'execute',
        command: command,
        args: typeof args === 'string' ? [args] : args
      });
    
  },

  
  openDialog: function openDialog(options) {
    
      var opts = options || {}
      if (_typeof(options) === 'object') {
        opts.default_path = opts.defaultPath
        Object.freeze(options);
      }
      return this.promisified({
        cmd: 'openDialog',
        options: opts
      });
    
  },

  
  saveDialog: function saveDialog(options) {
    
      var opts = options || {}
      if (_typeof(options) === 'object') {
        opts.default_path = opts.defaultPath
        Object.freeze(options);
      }
      return this.promisified({
        cmd: 'saveDialog',
        options: opts
      });
    
  },

loadAsset: function loadAsset(assetName, assetType) {
  return this.promisified({
    cmd: 'loadAsset',
    asset: assetName,
    asset_type: assetType || 'unknown'
  })
}
};

// init tauri API
try {
  window.tauri.invoke({
    cmd: 'init'
  })
} catch (e) {
  window.addEventListener('DOMContentLoaded', function () {
    window.tauri.invoke({
      cmd: 'init'
    })
  }, true)
}

document.addEventListener('error', function (e) {
  var target = e.target
  while (target != null) {
    if (target.matches ? target.matches('img') : target.msMatchesSelector('img')) {
      window.tauri.loadAsset(target.src, 'image')
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
          window.tauri.open(target.href)
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
