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

<% if (ctx.dev) { %>
function camelToKebab (string) {
  return string.replace(/([a-z0-9]|(?=[A-Z]))([A-Z])/g, '$1-$2').toLowerCase()
}
/**
 * @name return __whitelistWarning
 * @description Present a stylish warning to the developer that their API
 * call has not been whitelisted in tauri.conf.json
 * @param {String} func - function name to warn
 * @private
 */
var __whitelistWarning = function (func) {
    console.warn('%c[Tauri] Danger \ntauri.' + func + ' not whitelisted 💣\n%c\nAdd to tauri.conf.json: \n\ntauri: \n  whitelist: { \n    ' + camelToKebab(func) + ': true \n\nReference: https://github.com/tauri-apps/tauri/wiki' + func, 'background: red; color: white; font-weight: 800; padding: 2px; font-size:1.5em', ' ')
    return __reject()
  }
    <% } %>

<% if (ctx.dev) { %>
  /**
   * @name __reject
   * @description generates a promise used to deflect un-whitelisted tauri API calls
   * Its only purpose is to maintain thenable structure in client code without
   * breaking the application
   *  * @type {Promise<any>}
   * @private
   */
<% } %>
var __reject = function () {
  return new Promise(function (_, reject) {
    reject();
  });
}

window.tauri = {
  Dir: Dir,
  <% if (ctx.dev) { %>
    /**
     * @name invoke
     * @description Calls a Tauri Core feature, such as setTitle
     * @param {Object} args
     */
  <% } %>
  invoke: function invoke(args) {
    window.external.invoke(JSON.stringify(args));
  },

  <% if (ctx.dev) { %>
    /**
     * @name listen
     * @description Add an event listener to Tauri backend
     * @param {String} event
     * @param {Function} handler
     * @param {Boolean} once
     */
  <% } %>
  listen: function listen(event, handler) {
    <% if (tauri.whitelist.event === true || tauri.whitelist.all === true) { %>
    var once = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : false;
      this.invoke({
        cmd: 'listen',
        event: event,
        handler: window.tauri.transformCallback(handler, once),
        once: once
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('event')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name emit
     * @description Emits an evt to the Tauri back end
     * @param {String} evt
     * @param {Object} payload
     */
  <% } %>
  emit: function emit(evt, payload) {
    <% if (tauri.whitelist.event === true || tauri.whitelist.all === true) { %>
      this.invoke({
        cmd: 'emit',
        event: evt,
        payload: payload
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('event')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name transformCallback
     * @description Registers a callback with a uid
     * @param {Function} callback
     * @param {Boolean} once
     * @returns {*}
     */
  <% } %>
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

  <% if (ctx.dev) { %>
    /**
     * @name promisified
     * @description Turns a request into a chainable promise
     * @param {Object} args
     * @returns {Promise<any>}
     */
  <% } %>
  promisified: function promisified(args) {
    var _this = this;

    return new Promise(function (resolve, reject) {
      _this.invoke(_objectSpread({
        callback: _this.transformCallback(resolve),
        error: _this.transformCallback(reject)
      }, args));
    });
  },

  <% if (ctx.dev) { %>
    /**
     * @name readTextFile
     * @description Accesses a non-binary file on the user's filesystem
     * and returns the content. Permissions based on the app's PID owner
     * @param {String} path
     * @param {Object} [options]
     * @param {BaseDirectory} [options.dir]
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  readTextFile: function readTextFile(path, options) {
    <% if (tauri.whitelist.readTextFile === true || tauri.whitelist.all === true) { %>
      return this.promisified({
        cmd: 'readTextFile',
        path: path,
        options: options
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('readTextFile')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name readBinaryFile
     * @description Accesses a binary file on the user's filesystem
     * and returns the content. Permissions based on the app's PID owner
     * @param {String} path
     * @param {Object} [options]
     * @param {BaseDirectory} [options.dir]
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  readBinaryFile: function readBinaryFile(path, options) {
    <% if (tauri.whitelist.readBinaryFile === true || tauri.whitelist.all === true) { %>
      return this.promisified({
        cmd: 'readBinaryFile',
        path: path,
        options: options
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('readBinaryFile')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name writeFile
     * @description Write a file to the Local Filesystem.
     * Permissions based on the app's PID owner
     * @param {Object} cfg
     * @param {String} cfg.file
     * @param {String|Binary} cfg.contents
     * @param {Object} [options]
     * @param {BaseDirectory} [options.dir]
     */
  <% } %>
  writeFile: function writeFile(cfg, options) {
    <% if (tauri.whitelist.writeFile === true || tauri.whitelist.all === true) { %>
      if (_typeof(cfg) === 'object') {
        Object.freeze(cfg);
      }
      return this.promisified({
        cmd: 'writeFile',
        file: cfg.file,
        contents: cfg.contents,
        options: options
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('writeFile')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name readDir
     * @description Reads a directory
     * Permissions based on the app's PID owner
     * @param {String} path
     * @param {Object} [options]
     * @param {Boolean} [options.recursive]
     * @param {BaseDirectory} [options.dir]
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  readDir: function readDir(path, options) {
    <% if (tauri.whitelist.readDir === true || tauri.whitelist.all === true) { %>
      return this.promisified({
        cmd: 'readDir',
        path: path,
        options: options
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('readDir')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name createDir
     * @description Creates a directory
     * Permissions based on the app's PID owner
     * @param {String} path
     * @param {Object} [options]
     * @param {Boolean} [options.recursive]
     * @param {BaseDirectory} [options.dir]
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  createDir: function createDir(path, options) {
    <% if (tauri.whitelist.createDir === true || tauri.whitelist.all === true) { %>
      return this.promisified({
        cmd: 'createDir',
        path: path,
        options: options
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('createDir')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name removeDir
     * @description Removes a directory
     * Permissions based on the app's PID owner
     * @param {String} path
     * @param {Object} [options]
     * @param {Boolean} [options.recursive]
     * @param {BaseDirectory} [options.dir]
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  removeDir: function removeDir(path, options) {
    <% if (tauri.whitelist.removeDir === true || tauri.whitelist.all === true) { %>
      return this.promisified({
        cmd: 'removeDir',
        path: path,
        options: options
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('removeDir')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name copyFile
     * @description Copy file
     * Permissions based on the app's PID owner
     * @param {String} source
     * @param {String} destination
     * @param {Object} [options]
     * @param {BaseDirectory} [options.dir]
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  copyFile: function copyFile(source, destination, options) {
    <% if (tauri.whitelist.copyFile === true || tauri.whitelist.all === true) { %>
      return this.promisified({
        cmd: 'copyFile',
        source: source,
        destination: destination,
        options: options
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('copyFile')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name removeFile
     * @description Removes a file
     * Permissions based on the app's PID owner
     * @param {String} path
     * @param {Object} [options]
     * @param {BaseDirectory} [options.dir]
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  removeFile: function removeFile(path, options) {
    <% if (tauri.whitelist.removeFile === true || tauri.whitelist.all === true) { %>
      return this.promisified({
        cmd: 'removeFile',
        path: path,
        options: options
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('removeFile')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name renameFile
     * @description Renames a file
     * Permissions based on the app's PID owner
     * @param {String} path
     * @param {Object} [options]
     * @param {BaseDirectory} [options.dir]
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  renameFile: function renameFile(oldPath, newPath, options) {
    <% if (tauri.whitelist.renameFile === true || tauri.whitelist.all === true) { %>
      return this.promisified({
        cmd: 'renameFile',
        oldPath: oldPath,
        newPath: newPath,
        options: options
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('renameFile')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name setTitle
     * @description Set the application's title
     * @param {String} title
     */
  <% } %>
  setTitle: function setTitle(title) {
    <% if (tauri.whitelist.setTitle === true || tauri.whitelist.all === true) { %>
      this.invoke({
        cmd: 'setTitle',
        title: title
      });
    <% } else { %>
      <% if (ctx.dev) { %>
    return __whitelistWarning('setTitle')
          <% } %>
    return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name open
     * @description Open an URI
     * @param {String} uri
     */
  <% } %>
  open: function open(uri) {
    <% if (tauri.whitelist.open === true || tauri.whitelist.all === true) { %>
      this.invoke({
        cmd: 'open',
        uri: uri
      });
    <% } else { %>
    <% if (ctx.dev) { %>
      return __whitelistWarning('open')
          <% } %>
    return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name execute
     * @description Execute a program with arguments.
     * Permissions based on the app's PID owner
     * @param {String} command
     * @param {String|Array} args
     * @returns {*|Promise<any>|Promise}
     */
  <% } %>
  execute: function execute(command, args) {
    <% if (tauri.whitelist.execute === true || tauri.whitelist.all === true) { %>

      if (_typeof(args) === 'object') {
        Object.freeze(args);
      }

      return this.promisified({
        cmd: 'execute',
        command: command,
        args: typeof args === 'string' ? [args] : args
      });
    <% } else { %>
      <% if (ctx.dev) { %>
        return __whitelistWarning('execute')
          <% } %>
        return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name openDialog
     * @description Open a file/directory selection dialog
     * @param {String} [options]
     * @param {String} [options.filter]
     * @param {String} [options.defaultPath]
     * @param {Boolean} [options.multiple=false]
     * @param {Boolean} [options.directory=false]
     * @returns {Promise<String|String[]>} promise resolving to the select path(s)
     */
  <% } %>
  openDialog: function openDialog(options) {
    <% if (tauri.whitelist.openDialog === true || tauri.whitelist.all === true) { %>
      var opts = options || {}
      if (_typeof(options) === 'object') {
        opts.default_path = opts.defaultPath
        Object.freeze(options);
      }
      return this.promisified({
        cmd: 'openDialog',
        options: opts
      });
    <% } else { %>
    <% if (ctx.dev) { %>
      return __whitelistWarning('openDialog')
          <% } %>
    return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name saveDialog
     * @description Open a file/directory save dialog
     * @param {String} [options]
     * @param {String} [options.filter]
     * @param {String} [options.defaultPath]
     * @returns {Promise<String>} promise resolving to the select path
     */
  <% } %>
  saveDialog: function saveDialog(options) {
    <% if (tauri.whitelist.saveDialog === true || tauri.whitelist.all === true) { %>
      var opts = options || {}
      if (_typeof(options) === 'object') {
        opts.default_path = opts.defaultPath
        Object.freeze(options);
      }
      return this.promisified({
        cmd: 'saveDialog',
        options: opts
      });
    <% } else { %>
    <% if (ctx.dev) { %>
      return __whitelistWarning('saveDialog')
          <% } %>
    return __reject()
        <% } %>
  },

  <% if (ctx.dev) { %>
    /**
     * @name httpRequest
     * @description Makes an HTTP request
     * @param {Object} options
     * @param {String} options.method GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT or TRACE
     * @param {String} options.url the request URL
     * @param {Object} [options.headers] the request headers
     * @param {Object} [options.params] the request query params
     * @param {Object|String|Binary} [options.body] the request body
     * @param {Boolean} followRedirects whether to follow redirects or not
     * @param {Number} maxRedirections max number of redirections
     * @param {Number} connectTimeout request connect timeout
     * @param {Number} readTimeout request read timeout
     * @param {Number} timeout request timeout
     * @param {Boolean} allowCompression
     * @param {Number} [responseType=1] 1 - JSON, 2 - Text, 3 - Binary
     * @param {Number} [bodyType=3] 1 - Form, 2 - File, 3 - Auto
     * @returns {Promise<any>}
     */
  <% } %>
  httpRequest: function httpRequest(options) {
    <% if (tauri.whitelist.readBinaryFile === true || tauri.whitelist.all === true) { %>
      return this.promisified({
        cmd: 'httpRequest',
        options: options
      });
    <% } else { %>
      <% if (ctx.dev) { %>
          return __whitelistWarning('httpRequest')
          <% } %>
        return __reject()
        <% } %>
  },

  loadAsset: function loadAsset(assetName, assetType) {
    return this.promisified({
      cmd: 'loadAsset',
      asset: assetName,
      assetType: assetType || 'unknown'
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
