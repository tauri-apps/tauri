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

<% if (ctx.dev) { %>
/**
 * @name __whitelistWarning
 * @description Present a stylish warning to the developer that their API
 * call has not been whitelisted in tauri.conf.js
 * @param {String} func - function name to warn
 * @private
 */
var __whitelistWarning = function (func) {
  console.warn('%c[Tauri] Danger \ntauri.' + func + ' not whitelisted 💣\n%c\nAdd to tauri.conf.js: \n\ntauri: \n  whitelist: { \n    ' + func + ': true \n\nReference: https://tauri-apps.org/docs/api#' + func , 'background: red; color: white; font-weight: 800; padding: 2px; font-size:1.5em', ' ')
}
<% } %>

/**
 * @name __reject
 * @description is a private promise used to deflect un-whitelisted tauri API calls
 * Its only purpose is to maintain thenable structure in client code without
 * breaking the application
 *  * @type {Promise<any>}
 * @private
 */
var __reject = new Promise((reject) => { reject })

var tauri = {
<% if (ctx.dev) { %>
  /**
   * @name invoke
   * @description Calls a Tauri Core feature, such as setTitle
   * @param {Object} args
   */
<% } %>
  invoke: function (args) {
    Object.freeze(args)
    window.external.invoke(JSON.stringify(args))
  },

<% if (ctx.dev) { %>
  /**
   * @name addEventListener
   * @description Add an evt listener to Tauri back end
   * @param {String} evt
   * @param {Function} handler
   * @param {Boolean} once
   */
<% } %>
  addEventListener: function (evt, handler, once = false) {
    this.invoke({
      cmd: 'addEventListener',
      evt,
      handler: this.transformCallback(handler, once),
      once
    })
  },

<% if (ctx.dev) { %>
  /**
   * @name emit
   * @description Emits an evt to the Tauri back end
   * @param {String} evt
   * @param {Object} payload
   */
<% } %>
  emit: function (evt, payload) {
    this.invoke({
      cmd: 'emit',
      event: evt,
      payload
    })
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
  transformCallback: function (callback, once = true) {
    var identifier = Object.freeze(uid())
    window[identifier] = (result) => {
      if (once) {
        delete window[identifier]
      }
      return callback && callback(result)
    }
    return identifier
  },

<% if (ctx.dev) { %>
  /**
   * @name promisified
   * @description Turns a request into a chainable promise
   * @param {Object} args
   * @returns {Promise<any>}
   */
<% } %>
  promisified: function (args) {
    return new Promise((resolve, reject) => {
      this.invoke({
        callback: this.transformCallback(resolve),
        error: this.transformCallback(reject),
        ...args
      })
    })
  },

<% if (ctx.dev) { %>
  /**
   * @name readTextFile
   * @description Accesses a non-binary file on the user's filesystem
   * and returns the content. Permissions based on the app's PID owner
   * @param {String} path
   * @returns {*|Promise<any>|Promise}
   */
<% } %>
  readTextFile: function (path) {
  <% if (tauri.whitelist.readTextFile === true || tauri.whitelist.all === true) { %>
    Object.freeze(path)
    return this.promisified({ cmd: 'readTextFile', path })
      <% } else { %>
  <% if (ctx.dev) { %>
      __whitelistWarning('readTextFile')
      <% } %>
    return __reject
      <% } %>
  },

<% if (ctx.dev) { %>
  /**
   * @name readBinaryFile
   * @description Accesses a binary file on the user's filesystem
   * and returns the content. Permissions based on the app's PID owner
   * @param {String} path
   * @returns {*|Promise<any>|Promise}
   */
<% } %>
  readBinaryFile: function (path) {
  <% if (tauri.whitelist.readBinaryFile === true || tauri.whitelist.all === true) { %>
    Object.freeze(path)
    return this.promisified({ cmd: 'readBinaryFile', path })
      <% } else { %>
  <% if (ctx.dev) { %>
      __whitelistWarning('readBinaryFile')
      <% } %>
    return __reject
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
   */
<% } %>
  writeFile: function (cfg) {
    Object.freeze(cfg)
  <% if (tauri.whitelist.writeFile === true || tauri.whitelist.all === true) { %>
    this.invoke({ cmd: 'writeFile', file: cfg.file, contents: cfg.contents })
    <% } else { %>
  <% if (ctx.dev) { %>
      __whitelistWarning('writeFile')
      <% } %>
    return __reject
      <% } %>
  },

<% if (ctx.dev) { %>
  /**
   * @name listFiles
   * @description Get the files in a path.
   * Permissions based on the app's PID owner
   * @param {String} path
   * @returns {*|Promise<any>|Promise}
   */
<% } %>
  listFiles: function (path) {
  <% if (tauri.whitelist.listFiles === true || tauri.whitelist.all === true) { %>
    Object.freeze(path)
    return this.promisified({ cmd: 'listFiles', path })
      <% } else { %>
  <% if (ctx.dev) { %>
      __whitelistWarning('listFiles')
      <% } %>
    return __reject
      <% } %>
  },

<% if (ctx.dev) { %>
  /**
   * @name listDirs
   * @description Get the directories in a path.
   * Permissions based on the app's PID owner
   * @param {String} path
   * @returns {*|Promise<any>|Promise}
   */
<% } %>
  listDirs: function (path) {
  <% if (tauri.whitelist.listDirs === true || tauri.whitelist.all === true) { %>
    Object.freeze(path)
    return this.promisified({ cmd: 'listDirs', path })
      <% } else { %>
  <% if (ctx.dev) { %>
      __whitelistWarning('listDirs')
      <% } %>
    return __reject
      <% } %>
  },

<% if (ctx.dev) { %>
  /**
   * @name setTitle
   * @description Set the application's title
   * @param {String} title
   */
<% } %>
  setTitle: function (title) {
    <% if (tauri.whitelist.setTitle === true || tauri.whitelist.all === true) { %>
    Object.freeze(title)
    this.invoke({ cmd: 'setTitle', title })
      <% } else { %>
  <% if (ctx.dev) { %>
      __whitelistWarning('setTitle')
      <% } %>
    return __reject
      <% } %>
  },

  <% if (ctx.dev) { %>
  /**
   * @name open
   * @description Open an URI
   * @param {String} uri
   */
<% } %>
  open: function (uri) {
    <% if (tauri.whitelist.open === true || tauri.whitelist.all === true) { %>
    Object.freeze(uri)
    this.invoke({ cmd: 'open', uri })
      <% } else { %>
  <% if (ctx.dev) { %>
      __whitelistWarning('open')
      <% } %>
    return __reject
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
  execute: function (command, args) {
    <% if (tauri.whitelist.execute === true || tauri.whitelist.all === true) { %>
    Object.freeze(command)
    if (typeof args === 'string' || typeof args === 'object') {
      Object.freeze(args)
    }
    return this.promisified({ cmd: 'execute', command, args: typeof (args) === 'string' ? [args] : args })
  <% } else { %>
  <% if (ctx.dev) { %>
    __whitelistWarning('execute')
    <% } %>
    return __reject
      <% } %>
  },

<% if (ctx.dev) { %>
  /**
   * @name bridge
   * @description Securely pass a message to the backend.
   * @example
   *  this.$q.tauri.bridge('QBP/1/ping/client-1', 'pingback')
   * @param {String} command - a compressed, slash-delimited and
   * versioned API call to the backend.
   * @param {String|Object}payload
   * @returns {*|Promise<any>|Promise}
   */
<% } %>
  bridge: function (command, payload) {
<% if (tauri.whitelist.bridge === true || tauri.whitelist.all === true) { %>
    Object.freeze(command)
    if (typeof payload === 'string' || typeof payload === 'object') {
      Object.freeze(payload)
    }
    return this.promisified({ cmd: 'bridge', command, payload: typeof (payload) === 'object' ? [payload] : payload })
<% } else { %>
<% if (ctx.dev) { %>
    __whitelistWarning('bridge')
<% } %>
      return __reject
<% } %>
  }
}

document.addEventListener('DOMContentLoaded', function () {
  document.getElementById('__TAURI_IFRAME').contentWindow.a = 5
  window.alert('bb' + window.frames.length)
  // open <a href="..."> links with the Tauri API
  document.querySelector('body').addEventListener('click', function (e) {
    var target = e.target
    while (target != null) {
      if (target.matches ? target.matches('a') : target.msMatchesSelector('a')) {
        tauri.open(target.href)
        break
      }
      target = target.parentElement
    }
  }, true)
  // init tauri API
  tauri.invoke({
    cmd: 'init'
  })
})
