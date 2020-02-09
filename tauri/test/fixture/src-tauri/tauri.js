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




var __reject = function () {
  return new Promise(function (_, reject) {
    reject();
  });
}

window.tauri = {
  
  invoke: function invoke(args) {
    window.external.invoke(JSON.stringify(args));
  },

  
  listen: function listen(event, handler) {
    
      
        return __reject()
    
  },

  
  emit: function emit(evt, payload) {
    
      
        return __reject()
    
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
    
      
        return __reject()
    
  },

  
  readBinaryFile: function readBinaryFile(path) {
    
      
        return __reject()
    
  },

  
  writeFile: function writeFile(cfg) {
    
      
        return __reject()
    
  },

  
  listFiles: function listFiles(path) {
    
      
        return __reject()
    
  },

  
  listDirs: function listDirs(path) {
    
      
        return __reject()
    
  },

  
  setTitle: function setTitle(title) {
    
      
    return __reject()
    
  },

  
  open: function open(uri) {
    
    
    return __reject()
      
  },

  
  execute: function execute(command, args) {
    
      
        return __reject()
    
  },

  bridge: function bridge(command, payload) {
    
      
            return __reject()
    
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
        .then(img => {
          target.src = img
        })
      break
    }
    target = target.parentElement
  }
}, true)

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
