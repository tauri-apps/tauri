/* eslint-disable */

/**
 *  * THIS FILE IS GENERATED AUTOMATICALLY.
 * DO NOT EDIT.
 *
 * Please whitelist these API functions in tauri.conf.js
 *
 **/

/**
 * @module tauri
 * @description This API interface makes powerful interactions available
 * to be run on client side applications. They are opt-in features, and
 * must be enabled in tauri.conf.js
 *
 * Each binding MUST provide these interfaces in order to be compliant,
 * and also whitelist them based upon the developer's settings.
 */

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


var __reject = new Promise(function (reject) {
  reject;
});

window.tauri = {
  
  invoke: function invoke(args) {
    
      window.parent.postMessage({
        type: 'tauri-invoke',
        payload: JSON.stringify(args)
      }, '*')
      
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
    var identifier = Object.freeze(uid());

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

  
  readTextFile: function readTextFile(path) {
    
    Object.freeze(path);
    return this.promisified({
      cmd: 'readTextFile',
      path: path
    });
    
  },

  
  readBinaryFile: function readBinaryFile(path) {
    
    Object.freeze(path);
    return this.promisified({
      cmd: 'readBinaryFile',
      path: path
    });
    
  },

  
  writeFile: function writeFile(cfg) {
    
    Object.freeze(cfg);
    this.invoke({
      cmd: 'writeFile',
      file: cfg.file,
      contents: cfg.contents
    });
    
  },

  
  listFiles: function listFiles(path) {
    

    Object.freeze(path);
    return this.promisified({
      cmd: 'listFiles',
      path: path
    });
    
  },

  
  listDirs: function listDirs(path) {
    
    Object.freeze(path);
    return this.promisified({
      cmd: 'listDirs',
      path: path
    });
    
  },

  
  setTitle: function setTitle(title) {
    
    Object.freeze(title);
    this.invoke({
      cmd: 'setTitle',
      title: title
    });
    
  },

  
  open: function open(uri) {
    
    Object.freeze(uri);
    this.invoke({
      cmd: 'open',
      uri: uri
    });
    
  },

  
  execute: function execute(command, args) {
    

    Object.freeze(command);

    if (typeof args === 'string' || _typeof(args) === 'object') {
      Object.freeze(args);
    }

    return this.promisified({
      cmd: 'execute',
      command: command,
      args: typeof args === 'string' ? [args] : args
    });
    
  },

  bridge: function bridge(command, payload) {
    

    Object.freeze(command);

    if (typeof payload === 'string' || _typeof(payload) === 'object') {
      Object.freeze(payload);
    }

    return this.promisified({
      cmd: 'bridge',
      command: command,
      payload: _typeof(payload) === 'object' ? [payload] : payload
    });
    
  }
};

window.addEventListener('message', function (event) {
  event.data.type === 'tauri-callback' && window[event.data.callback](event.data.payload)
})

// init tauri API

window.onTauriInit = function () {
  alert('mounted')
  console.log(window.tauri)
}

function __initTauri () {
  if (window.onTauriInit !== void 0) {
    window.onTauriInit()
  }
  alert('INIT')
}

if (window.top === window.self) {
  // detect if we are in an iframe
  // technically "dev mode"
  window.addEventListener('message', function (event) {
    alert(JSON.stringify(event.data.payload))
    if (event.data.type === 'tauri-invoke') {
      window.external.invoke(event.data.payload)
    }
  }, true)
} else {
  // this is an iframe
  window.addEventListener('message', function (event) {
    alert(JSON.stringify(event))
    alert(JSON.stringify(event.data))
    alert(JSON.stringify(event.data.payload))
  }, true)
}

window.addEventListener('DOMContentLoaded', function () {
  // open <a href="..."> links with the Tauri API
  document.querySelector('body').addEventListener('click', function (e) {
    var target = e.target
    while (target != null) {
      if (target.matches ? target.matches('a') : target.msMatchesSelector('a')) {
        window.tauri.open(target.href)
        break
      }
      target = target.parentElement
    }
  }, true)
}, true)
