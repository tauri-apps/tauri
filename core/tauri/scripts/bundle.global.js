var __TAURI__ = (() => {
  var __defProp = Object.defineProperty;
  var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
  var __getOwnPropNames = Object.getOwnPropertyNames;
  var __getOwnPropSymbols = Object.getOwnPropertySymbols;
  var __hasOwnProp = Object.prototype.hasOwnProperty;
  var __propIsEnum = Object.prototype.propertyIsEnumerable;
  var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
  var __spreadValues = (a, b) => {
    for (var prop in b || (b = {}))
      if (__hasOwnProp.call(b, prop))
        __defNormalProp(a, prop, b[prop]);
    if (__getOwnPropSymbols)
      for (var prop of __getOwnPropSymbols(b)) {
        if (__propIsEnum.call(b, prop))
          __defNormalProp(a, prop, b[prop]);
      }
    return a;
  };
  var __markAsModule = (target) => __defProp(target, "__esModule", { value: true });
  var __export = (target, all) => {
    for (var name in all)
      __defProp(target, name, { get: all[name], enumerable: true });
  };
  var __reExport = (target, module, copyDefault, desc) => {
    if (module && typeof module === "object" || typeof module === "function") {
      for (let key of __getOwnPropNames(module))
        if (!__hasOwnProp.call(target, key) && (copyDefault || key !== "default"))
          __defProp(target, key, { get: () => module[key], enumerable: !(desc = __getOwnPropDesc(module, key)) || desc.enumerable });
    }
    return target;
  };
  var __toCommonJS = /* @__PURE__ */ ((cache) => {
    return (module, temp) => {
      return cache && cache.get(module) || (temp = __reExport(__markAsModule({}), module, 1), cache && cache.set(module, temp), temp);
    };
  })(typeof WeakMap !== "undefined" ? /* @__PURE__ */ new WeakMap() : 0);

  // src/index.ts
  var src_exports = {};
  __export(src_exports, {
    app: () => app_exports,
    cli: () => cli_exports,
    clipboard: () => clipboard_exports,
    dialog: () => dialog_exports,
    event: () => event_exports,
    fs: () => fs_exports,
    globalShortcut: () => globalShortcut_exports,
    http: () => http_exports,
    notification: () => notification_exports,
    os: () => os_exports,
    path: () => path_exports,
    process: () => process_exports,
    shell: () => shell_exports,
    tauri: () => tauri_exports,
    updater: () => updater_exports,
    window: () => window_exports
  });

  // src/app.ts
  var app_exports = {};
  __export(app_exports, {
    getName: () => getName,
    getTauriVersion: () => getTauriVersion,
    getVersion: () => getVersion
  });

  // src/tauri.ts
  var tauri_exports = {};
  __export(tauri_exports, {
    convertFileSrc: () => convertFileSrc,
    invoke: () => invoke,
    transformCallback: () => transformCallback
  });
  function uid() {
    return window.crypto.getRandomValues(new Uint32Array(1))[0];
  }
  function transformCallback(callback, once3 = false) {
    const identifier = uid();
    const prop = `_${identifier}`;
    Object.defineProperty(window, prop, {
      value: (result) => {
        if (once3) {
          Reflect.deleteProperty(window, prop);
        }
        return callback == null ? void 0 : callback(result);
      },
      writable: false,
      configurable: true
    });
    return identifier;
  }
  async function invoke(cmd, args = {}) {
    return new Promise((resolve2, reject) => {
      const callback = transformCallback((e) => {
        resolve2(e);
        Reflect.deleteProperty(window, error);
      }, true);
      const error = transformCallback((e) => {
        reject(e);
        Reflect.deleteProperty(window, callback);
      }, true);
      window.__TAURI_IPC__(__spreadValues({
        cmd,
        callback,
        error
      }, args));
    });
  }
  function convertFileSrc(filePath) {
    return navigator.userAgent.includes("Windows") ? `https://asset.localhost/${filePath}` : `asset://${filePath}`;
  }

  // src/helpers/tauri.ts
  async function invokeTauriCommand(command) {
    return invoke("tauri", command);
  }

  // src/app.ts
  async function getVersion() {
    return invokeTauriCommand({
      __tauriModule: "App",
      message: {
        cmd: "getAppVersion"
      }
    });
  }
  async function getName() {
    return invokeTauriCommand({
      __tauriModule: "App",
      message: {
        cmd: "getAppName"
      }
    });
  }
  async function getTauriVersion() {
    return invokeTauriCommand({
      __tauriModule: "App",
      message: {
        cmd: "getTauriVersion"
      }
    });
  }

  // src/cli.ts
  var cli_exports = {};
  __export(cli_exports, {
    getMatches: () => getMatches
  });
  async function getMatches() {
    return invokeTauriCommand({
      __tauriModule: "Cli",
      message: {
        cmd: "cliMatches"
      }
    });
  }

  // src/clipboard.ts
  var clipboard_exports = {};
  __export(clipboard_exports, {
    readText: () => readText,
    writeText: () => writeText
  });
  async function writeText(text) {
    return invokeTauriCommand({
      __tauriModule: "Clipboard",
      message: {
        cmd: "writeText",
        data: text
      }
    });
  }
  async function readText() {
    return invokeTauriCommand({
      __tauriModule: "Clipboard",
      message: {
        cmd: "readText"
      }
    });
  }

  // src/dialog.ts
  var dialog_exports = {};
  __export(dialog_exports, {
    ask: () => ask,
    confirm: () => confirm,
    message: () => message,
    open: () => open,
    save: () => save
  });
  async function open(options = {}) {
    if (typeof options === "object") {
      Object.freeze(options);
    }
    return invokeTauriCommand({
      __tauriModule: "Dialog",
      message: {
        cmd: "openDialog",
        options
      }
    });
  }
  async function save(options = {}) {
    if (typeof options === "object") {
      Object.freeze(options);
    }
    return invokeTauriCommand({
      __tauriModule: "Dialog",
      message: {
        cmd: "saveDialog",
        options
      }
    });
  }
  async function message(message2) {
    return invokeTauriCommand({
      __tauriModule: "Dialog",
      message: {
        cmd: "messageDialog",
        message: message2
      }
    });
  }
  async function ask(message2, title) {
    return invokeTauriCommand({
      __tauriModule: "Dialog",
      message: {
        cmd: "askDialog",
        title,
        message: message2
      }
    });
  }
  async function confirm(message2, title) {
    return invokeTauriCommand({
      __tauriModule: "Dialog",
      message: {
        cmd: "confirmDialog",
        title,
        message: message2
      }
    });
  }

  // src/event.ts
  var event_exports = {};
  __export(event_exports, {
    emit: () => emit2,
    listen: () => listen2,
    once: () => once2
  });

  // src/helpers/event.ts
  async function _unlisten(eventId) {
    return invokeTauriCommand({
      __tauriModule: "Event",
      message: {
        cmd: "unlisten",
        eventId
      }
    });
  }
  async function emit(event, windowLabel, payload) {
    await invokeTauriCommand({
      __tauriModule: "Event",
      message: {
        cmd: "emit",
        event,
        windowLabel,
        payload: typeof payload === "string" ? payload : JSON.stringify(payload)
      }
    });
  }
  async function listen(event, windowLabel, handler) {
    return invokeTauriCommand({
      __tauriModule: "Event",
      message: {
        cmd: "listen",
        event,
        windowLabel,
        handler: transformCallback(handler)
      }
    }).then((eventId) => {
      return async () => _unlisten(eventId);
    });
  }
  async function once(event, windowLabel, handler) {
    return listen(event, windowLabel, (eventData) => {
      handler(eventData);
      _unlisten(eventData.id).catch(() => {
      });
    });
  }

  // src/event.ts
  async function listen2(event, handler) {
    return listen(event, null, handler);
  }
  async function once2(event, handler) {
    return once(event, null, handler);
  }
  async function emit2(event, payload) {
    return emit(event, void 0, payload);
  }

  // src/fs.ts
  var fs_exports = {};
  __export(fs_exports, {
    BaseDirectory: () => BaseDirectory,
    Dir: () => BaseDirectory,
    copyFile: () => copyFile,
    createDir: () => createDir,
    readBinaryFile: () => readBinaryFile,
    readDir: () => readDir,
    readTextFile: () => readTextFile,
    removeDir: () => removeDir,
    removeFile: () => removeFile,
    renameFile: () => renameFile,
    writeBinaryFile: () => writeBinaryFile,
    writeFile: () => writeFile
  });
  var BaseDirectory = /* @__PURE__ */ ((BaseDirectory2) => {
    BaseDirectory2[BaseDirectory2["Audio"] = 1] = "Audio";
    BaseDirectory2[BaseDirectory2["Cache"] = 2] = "Cache";
    BaseDirectory2[BaseDirectory2["Config"] = 3] = "Config";
    BaseDirectory2[BaseDirectory2["Data"] = 4] = "Data";
    BaseDirectory2[BaseDirectory2["LocalData"] = 5] = "LocalData";
    BaseDirectory2[BaseDirectory2["Desktop"] = 6] = "Desktop";
    BaseDirectory2[BaseDirectory2["Document"] = 7] = "Document";
    BaseDirectory2[BaseDirectory2["Download"] = 8] = "Download";
    BaseDirectory2[BaseDirectory2["Executable"] = 9] = "Executable";
    BaseDirectory2[BaseDirectory2["Font"] = 10] = "Font";
    BaseDirectory2[BaseDirectory2["Home"] = 11] = "Home";
    BaseDirectory2[BaseDirectory2["Picture"] = 12] = "Picture";
    BaseDirectory2[BaseDirectory2["Public"] = 13] = "Public";
    BaseDirectory2[BaseDirectory2["Runtime"] = 14] = "Runtime";
    BaseDirectory2[BaseDirectory2["Template"] = 15] = "Template";
    BaseDirectory2[BaseDirectory2["Video"] = 16] = "Video";
    BaseDirectory2[BaseDirectory2["Resource"] = 17] = "Resource";
    BaseDirectory2[BaseDirectory2["App"] = 18] = "App";
    BaseDirectory2[BaseDirectory2["Log"] = 19] = "Log";
    return BaseDirectory2;
  })(BaseDirectory || {});
  async function readTextFile(filePath, options = {}) {
    return invokeTauriCommand({
      __tauriModule: "Fs",
      message: {
        cmd: "readFile",
        path: filePath,
        options
      }
    }).then((data) => new TextDecoder().decode(new Uint8Array(data)));
  }
  async function readBinaryFile(filePath, options = {}) {
    const arr = await invokeTauriCommand({
      __tauriModule: "Fs",
      message: {
        cmd: "readFile",
        path: filePath,
        options
      }
    });
    return Uint8Array.from(arr);
  }
  async function writeFile(file, options = {}) {
    if (typeof options === "object") {
      Object.freeze(options);
    }
    if (typeof file === "object") {
      Object.freeze(file);
    }
    return invokeTauriCommand({
      __tauriModule: "Fs",
      message: {
        cmd: "writeFile",
        path: file.path,
        contents: Array.from(new TextEncoder().encode(file.contents)),
        options
      }
    });
  }
  async function writeBinaryFile(file, options = {}) {
    if (typeof options === "object") {
      Object.freeze(options);
    }
    if (typeof file === "object") {
      Object.freeze(file);
    }
    return invokeTauriCommand({
      __tauriModule: "Fs",
      message: {
        cmd: "writeFile",
        path: file.path,
        contents: Array.from(file.contents),
        options
      }
    });
  }
  async function readDir(dir, options = {}) {
    return invokeTauriCommand({
      __tauriModule: "Fs",
      message: {
        cmd: "readDir",
        path: dir,
        options
      }
    });
  }
  async function createDir(dir, options = {}) {
    return invokeTauriCommand({
      __tauriModule: "Fs",
      message: {
        cmd: "createDir",
        path: dir,
        options
      }
    });
  }
  async function removeDir(dir, options = {}) {
    return invokeTauriCommand({
      __tauriModule: "Fs",
      message: {
        cmd: "removeDir",
        path: dir,
        options
      }
    });
  }
  async function copyFile(source, destination, options = {}) {
    return invokeTauriCommand({
      __tauriModule: "Fs",
      message: {
        cmd: "copyFile",
        source,
        destination,
        options
      }
    });
  }
  async function removeFile(file, options = {}) {
    return invokeTauriCommand({
      __tauriModule: "Fs",
      message: {
        cmd: "removeFile",
        path: file,
        options
      }
    });
  }
  async function renameFile(oldPath, newPath, options = {}) {
    return invokeTauriCommand({
      __tauriModule: "Fs",
      message: {
        cmd: "renameFile",
        oldPath,
        newPath,
        options
      }
    });
  }

  // src/globalShortcut.ts
  var globalShortcut_exports = {};
  __export(globalShortcut_exports, {
    isRegistered: () => isRegistered,
    register: () => register,
    registerAll: () => registerAll,
    unregister: () => unregister,
    unregisterAll: () => unregisterAll
  });
  async function register(shortcut, handler) {
    return invokeTauriCommand({
      __tauriModule: "GlobalShortcut",
      message: {
        cmd: "register",
        shortcut,
        handler: transformCallback(handler)
      }
    });
  }
  async function registerAll(shortcuts, handler) {
    return invokeTauriCommand({
      __tauriModule: "GlobalShortcut",
      message: {
        cmd: "registerAll",
        shortcuts,
        handler: transformCallback(handler)
      }
    });
  }
  async function isRegistered(shortcut) {
    return invokeTauriCommand({
      __tauriModule: "GlobalShortcut",
      message: {
        cmd: "isRegistered",
        shortcut
      }
    });
  }
  async function unregister(shortcut) {
    return invokeTauriCommand({
      __tauriModule: "GlobalShortcut",
      message: {
        cmd: "unregister",
        shortcut
      }
    });
  }
  async function unregisterAll() {
    return invokeTauriCommand({
      __tauriModule: "GlobalShortcut",
      message: {
        cmd: "unregisterAll"
      }
    });
  }

  // src/http.ts
  var http_exports = {};
  __export(http_exports, {
    Body: () => Body,
    Client: () => Client,
    Response: () => Response,
    ResponseType: () => ResponseType,
    fetch: () => fetch,
    getClient: () => getClient
  });
  var ResponseType = /* @__PURE__ */ ((ResponseType2) => {
    ResponseType2[ResponseType2["JSON"] = 1] = "JSON";
    ResponseType2[ResponseType2["Text"] = 2] = "Text";
    ResponseType2[ResponseType2["Binary"] = 3] = "Binary";
    return ResponseType2;
  })(ResponseType || {});
  var Body = class {
    constructor(type2, payload) {
      this.type = type2;
      this.payload = payload;
    }
    static form(data) {
      const form = {};
      for (const key in data) {
        const v = data[key];
        form[key] = typeof v === "string" ? v : Array.from(v);
      }
      return new Body("Form", form);
    }
    static json(data) {
      return new Body("Json", data);
    }
    static text(value) {
      return new Body("Text", value);
    }
    static bytes(bytes) {
      return new Body("Bytes", Array.from(bytes));
    }
  };
  var Response = class {
    constructor(response) {
      this.url = response.url;
      this.status = response.status;
      this.ok = this.status >= 200 && this.status < 300;
      this.headers = response.headers;
      this.rawHeaders = response.rawHeaders;
      this.data = response.data;
    }
  };
  var Client = class {
    constructor(id) {
      this.id = id;
    }
    async drop() {
      return invokeTauriCommand({
        __tauriModule: "Http",
        message: {
          cmd: "dropClient",
          client: this.id
        }
      });
    }
    async request(options) {
      const jsonResponse = !options.responseType || options.responseType === 1 /* JSON */;
      if (jsonResponse) {
        options.responseType = 2 /* Text */;
      }
      return invokeTauriCommand({
        __tauriModule: "Http",
        message: {
          cmd: "httpRequest",
          client: this.id,
          options
        }
      }).then((res) => {
        const response = new Response(res);
        if (jsonResponse) {
          try {
            response.data = JSON.parse(response.data);
          } catch (e) {
            if (response.ok && response.data === "") {
              response.data = {};
            } else if (response.ok) {
              throw Error(`Failed to parse response \`${response.data}\` as JSON: ${e};
              try setting the \`responseType\` option to \`ResponseType.Text\` or \`ResponseType.Binary\` if the API does not return a JSON response.`);
            }
          }
          return response;
        }
        return response;
      });
    }
    async get(url, options) {
      return this.request(__spreadValues({
        method: "GET",
        url
      }, options));
    }
    async post(url, body, options) {
      return this.request(__spreadValues({
        method: "POST",
        url,
        body
      }, options));
    }
    async put(url, body, options) {
      return this.request(__spreadValues({
        method: "PUT",
        url,
        body
      }, options));
    }
    async patch(url, options) {
      return this.request(__spreadValues({
        method: "PATCH",
        url
      }, options));
    }
    async delete(url, options) {
      return this.request(__spreadValues({
        method: "DELETE",
        url
      }, options));
    }
  };
  async function getClient(options) {
    return invokeTauriCommand({
      __tauriModule: "Http",
      message: {
        cmd: "createClient",
        options
      }
    }).then((id) => new Client(id));
  }
  var defaultClient = null;
  async function fetch(url, options) {
    var _a;
    if (defaultClient === null) {
      defaultClient = await getClient();
    }
    return defaultClient.request(__spreadValues({
      url,
      method: (_a = options == null ? void 0 : options.method) != null ? _a : "GET"
    }, options));
  }

  // src/notification.ts
  var notification_exports = {};
  __export(notification_exports, {
    isPermissionGranted: () => isPermissionGranted,
    requestPermission: () => requestPermission,
    sendNotification: () => sendNotification
  });
  async function isPermissionGranted() {
    if (window.Notification.permission !== "default") {
      return Promise.resolve(window.Notification.permission === "granted");
    }
    return invokeTauriCommand({
      __tauriModule: "Notification",
      message: {
        cmd: "isNotificationPermissionGranted"
      }
    });
  }
  async function requestPermission() {
    return window.Notification.requestPermission();
  }
  function sendNotification(options) {
    if (typeof options === "string") {
      new window.Notification(options);
    } else {
      new window.Notification(options.title, options);
    }
  }

  // src/path.ts
  var path_exports = {};
  __export(path_exports, {
    BaseDirectory: () => BaseDirectory,
    appDir: () => appDir,
    audioDir: () => audioDir,
    basename: () => basename,
    cacheDir: () => cacheDir,
    configDir: () => configDir,
    dataDir: () => dataDir,
    delimiter: () => delimiter,
    desktopDir: () => desktopDir,
    dirname: () => dirname,
    documentDir: () => documentDir,
    downloadDir: () => downloadDir,
    executableDir: () => executableDir,
    extname: () => extname,
    fontDir: () => fontDir,
    homeDir: () => homeDir,
    isAbsolute: () => isAbsolute,
    join: () => join,
    localDataDir: () => localDataDir,
    logDir: () => logDir,
    normalize: () => normalize,
    pictureDir: () => pictureDir,
    publicDir: () => publicDir,
    resolve: () => resolve,
    resourceDir: () => resourceDir,
    runtimeDir: () => runtimeDir,
    sep: () => sep,
    templateDir: () => templateDir,
    videoDir: () => videoDir
  });

  // src/helpers/os-check.ts
  function isWindows() {
    return navigator.appVersion.includes("Win");
  }

  // src/path.ts
  async function appDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 18 /* App */
      }
    });
  }
  async function audioDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 1 /* Audio */
      }
    });
  }
  async function cacheDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 2 /* Cache */
      }
    });
  }
  async function configDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 3 /* Config */
      }
    });
  }
  async function dataDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 4 /* Data */
      }
    });
  }
  async function desktopDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 6 /* Desktop */
      }
    });
  }
  async function documentDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 7 /* Document */
      }
    });
  }
  async function downloadDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 8 /* Download */
      }
    });
  }
  async function executableDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 9 /* Executable */
      }
    });
  }
  async function fontDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 10 /* Font */
      }
    });
  }
  async function homeDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 11 /* Home */
      }
    });
  }
  async function localDataDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 5 /* LocalData */
      }
    });
  }
  async function pictureDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 12 /* Picture */
      }
    });
  }
  async function publicDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 13 /* Public */
      }
    });
  }
  async function resourceDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 17 /* Resource */
      }
    });
  }
  async function runtimeDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 14 /* Runtime */
      }
    });
  }
  async function templateDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 15 /* Template */
      }
    });
  }
  async function videoDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 16 /* Video */
      }
    });
  }
  async function logDir() {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolvePath",
        path: "",
        directory: 19 /* Log */
      }
    });
  }
  var sep = isWindows() ? "\\" : "/";
  var delimiter = isWindows() ? ";" : ":";
  async function resolve(...paths) {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "resolve",
        paths
      }
    });
  }
  async function normalize(path) {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "normalize",
        path
      }
    });
  }
  async function join(...paths) {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "join",
        paths
      }
    });
  }
  async function dirname(path) {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "dirname",
        path
      }
    });
  }
  async function extname(path) {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "extname",
        path
      }
    });
  }
  async function basename(path, ext) {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "basename",
        path,
        ext
      }
    });
  }
  async function isAbsolute(path) {
    return invokeTauriCommand({
      __tauriModule: "Path",
      message: {
        cmd: "isAbsolute",
        path
      }
    });
  }

  // src/process.ts
  var process_exports = {};
  __export(process_exports, {
    exit: () => exit,
    relaunch: () => relaunch
  });
  async function exit(exitCode = 0) {
    return invokeTauriCommand({
      __tauriModule: "Process",
      message: {
        cmd: "exit",
        exitCode
      }
    });
  }
  async function relaunch() {
    return invokeTauriCommand({
      __tauriModule: "Process",
      message: {
        cmd: "relaunch"
      }
    });
  }

  // src/shell.ts
  var shell_exports = {};
  __export(shell_exports, {
    Child: () => Child,
    Command: () => Command,
    open: () => open2
  });
  async function execute(onEvent, program, args, options) {
    if (typeof args === "object") {
      Object.freeze(args);
    }
    return invokeTauriCommand({
      __tauriModule: "Shell",
      message: {
        cmd: "execute",
        program,
        args,
        options,
        onEventFn: transformCallback(onEvent)
      }
    });
  }
  var EventEmitter = class {
    constructor() {
      this.eventListeners = /* @__PURE__ */ Object.create(null);
    }
    addEventListener(event, handler) {
      if (event in this.eventListeners) {
        this.eventListeners[event].push(handler);
      } else {
        this.eventListeners[event] = [handler];
      }
    }
    _emit(event, payload) {
      if (event in this.eventListeners) {
        const listeners = this.eventListeners[event];
        for (const listener of listeners) {
          listener(payload);
        }
      }
    }
    on(event, handler) {
      this.addEventListener(event, handler);
      return this;
    }
  };
  var Child = class {
    constructor(pid) {
      this.pid = pid;
    }
    async write(data) {
      return invokeTauriCommand({
        __tauriModule: "Shell",
        message: {
          cmd: "stdinWrite",
          pid: this.pid,
          buffer: typeof data === "string" ? data : Array.from(data)
        }
      });
    }
    async kill() {
      return invokeTauriCommand({
        __tauriModule: "Shell",
        message: {
          cmd: "killChild",
          pid: this.pid
        }
      });
    }
  };
  var Command = class extends EventEmitter {
    constructor(program, args = [], options) {
      super();
      this.stdout = new EventEmitter();
      this.stderr = new EventEmitter();
      this.program = program;
      this.args = typeof args === "string" ? [args] : args;
      this.options = options != null ? options : {};
    }
    static sidecar(program, args = [], options) {
      const instance = new Command(program, args, options);
      instance.options.sidecar = true;
      return instance;
    }
    async spawn() {
      return execute((event) => {
        switch (event.event) {
          case "Error":
            this._emit("error", event.payload);
            break;
          case "Terminated":
            this._emit("close", event.payload);
            break;
          case "Stdout":
            this.stdout._emit("data", event.payload);
            break;
          case "Stderr":
            this.stderr._emit("data", event.payload);
            break;
        }
      }, this.program, this.args, this.options).then((pid) => new Child(pid));
    }
    async execute() {
      return new Promise((resolve2, reject) => {
        this.on("error", reject);
        const stdout = [];
        const stderr = [];
        this.stdout.on("data", (line) => {
          stdout.push(line);
        });
        this.stderr.on("data", (line) => {
          stderr.push(line);
        });
        this.on("close", (payload) => {
          resolve2({
            code: payload.code,
            signal: payload.signal,
            stdout: stdout.join("\n"),
            stderr: stderr.join("\n")
          });
        });
        this.spawn().catch(reject);
      });
    }
  };
  async function open2(path, openWith) {
    return invokeTauriCommand({
      __tauriModule: "Shell",
      message: {
        cmd: "open",
        path,
        with: openWith
      }
    });
  }

  // src/updater.ts
  var updater_exports = {};
  __export(updater_exports, {
    checkUpdate: () => checkUpdate,
    installUpdate: () => installUpdate
  });
  async function installUpdate() {
    let unlistenerFn;
    function cleanListener() {
      if (unlistenerFn) {
        unlistenerFn();
      }
      unlistenerFn = void 0;
    }
    return new Promise((resolve2, reject) => {
      function onStatusChange(statusResult) {
        if (statusResult.error) {
          cleanListener();
          return reject(statusResult.error);
        }
        if (statusResult.status === "DONE") {
          cleanListener();
          return resolve2();
        }
      }
      listen2("tauri://update-status", (data) => {
        onStatusChange(data == null ? void 0 : data.payload);
      }).then((fn) => {
        unlistenerFn = fn;
      }).catch((e) => {
        cleanListener();
        throw e;
      });
      emit2("tauri://update-install").catch((e) => {
        cleanListener();
        throw e;
      });
    });
  }
  async function checkUpdate() {
    let unlistenerFn;
    function cleanListener() {
      if (unlistenerFn) {
        unlistenerFn();
      }
      unlistenerFn = void 0;
    }
    return new Promise((resolve2, reject) => {
      function onUpdateAvailable(manifest) {
        cleanListener();
        return resolve2({
          manifest,
          shouldUpdate: true
        });
      }
      function onStatusChange(statusResult) {
        if (statusResult.error) {
          cleanListener();
          return reject(statusResult.error);
        }
        if (statusResult.status === "UPTODATE") {
          cleanListener();
          return resolve2({
            shouldUpdate: false
          });
        }
      }
      once2("tauri://update-available", (data) => {
        onUpdateAvailable(data == null ? void 0 : data.payload);
      }).catch((e) => {
        cleanListener();
        throw e;
      });
      listen2("tauri://update-status", (data) => {
        onStatusChange(data == null ? void 0 : data.payload);
      }).then((fn) => {
        unlistenerFn = fn;
      }).catch((e) => {
        cleanListener();
        throw e;
      });
      emit2("tauri://update").catch((e) => {
        cleanListener();
        throw e;
      });
    });
  }

  // src/window.ts
  var window_exports = {};
  __export(window_exports, {
    LogicalPosition: () => LogicalPosition,
    LogicalSize: () => LogicalSize,
    PhysicalPosition: () => PhysicalPosition,
    PhysicalSize: () => PhysicalSize,
    UserAttentionType: () => UserAttentionType,
    WebviewWindow: () => WebviewWindow,
    WebviewWindowHandle: () => WebviewWindowHandle,
    WindowManager: () => WindowManager,
    appWindow: () => appWindow,
    availableMonitors: () => availableMonitors,
    currentMonitor: () => currentMonitor,
    getAll: () => getAll,
    getCurrent: () => getCurrent,
    primaryMonitor: () => primaryMonitor
  });
  var LogicalSize = class {
    constructor(width, height) {
      this.type = "Logical";
      this.width = width;
      this.height = height;
    }
  };
  var PhysicalSize = class {
    constructor(width, height) {
      this.type = "Physical";
      this.width = width;
      this.height = height;
    }
    toLogical(scaleFactor) {
      return new LogicalSize(this.width / scaleFactor, this.height / scaleFactor);
    }
  };
  var LogicalPosition = class {
    constructor(x, y) {
      this.type = "Logical";
      this.x = x;
      this.y = y;
    }
  };
  var PhysicalPosition = class {
    constructor(x, y) {
      this.type = "Physical";
      this.x = x;
      this.y = y;
    }
    toLogical(scaleFactor) {
      return new LogicalPosition(this.x / scaleFactor, this.y / scaleFactor);
    }
  };
  var UserAttentionType = /* @__PURE__ */ ((UserAttentionType2) => {
    UserAttentionType2[UserAttentionType2["Critical"] = 1] = "Critical";
    UserAttentionType2[UserAttentionType2["Informational"] = 2] = "Informational";
    return UserAttentionType2;
  })(UserAttentionType || {});
  function getCurrent() {
    return new WebviewWindow(window.__TAURI_METADATA__.__currentWindow.label, {
      skip: true
    });
  }
  function getAll() {
    return window.__TAURI_METADATA__.__windows.map((w) => new WebviewWindow(w.label, {
      skip: true
    }));
  }
  var localTauriEvents = ["tauri://created", "tauri://error"];
  var WebviewWindowHandle = class {
    constructor(label) {
      this.label = label;
      this.listeners = /* @__PURE__ */ Object.create(null);
    }
    async listen(event, handler) {
      if (this._handleTauriEvent(event, handler)) {
        return Promise.resolve(() => {
          const listeners = this.listeners[event];
          listeners.splice(listeners.indexOf(handler), 1);
        });
      }
      return listen(event, this.label, handler);
    }
    async once(event, handler) {
      if (this._handleTauriEvent(event, handler)) {
        return Promise.resolve(() => {
          const listeners = this.listeners[event];
          listeners.splice(listeners.indexOf(handler), 1);
        });
      }
      return once(event, this.label, handler);
    }
    async emit(event, payload) {
      if (localTauriEvents.includes(event)) {
        for (const handler of this.listeners[event] || []) {
          handler({ event, id: -1, windowLabel: this.label, payload });
        }
        return Promise.resolve();
      }
      return emit(event, this.label, payload);
    }
    _handleTauriEvent(event, handler) {
      if (localTauriEvents.includes(event)) {
        if (!(event in this.listeners)) {
          this.listeners[event] = [handler];
        } else {
          this.listeners[event].push(handler);
        }
        return true;
      }
      return false;
    }
  };
  var WindowManager = class extends WebviewWindowHandle {
    async scaleFactor() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "scaleFactor"
            }
          }
        }
      });
    }
    async innerPosition() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "innerPosition"
            }
          }
        }
      }).then(({ x, y }) => new PhysicalPosition(x, y));
    }
    async outerPosition() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "outerPosition"
            }
          }
        }
      }).then(({ x, y }) => new PhysicalPosition(x, y));
    }
    async innerSize() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "innerSize"
            }
          }
        }
      }).then(({ width, height }) => new PhysicalSize(width, height));
    }
    async outerSize() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "outerSize"
            }
          }
        }
      }).then(({ width, height }) => new PhysicalSize(width, height));
    }
    async isFullscreen() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "isFullscreen"
            }
          }
        }
      });
    }
    async isMaximized() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "isMaximized"
            }
          }
        }
      });
    }
    async isDecorated() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "isDecorated"
            }
          }
        }
      });
    }
    async isResizable() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "isResizable"
            }
          }
        }
      });
    }
    async isVisible() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "isVisible"
            }
          }
        }
      });
    }
    async center() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "center"
            }
          }
        }
      });
    }
    async requestUserAttention(requestType) {
      let requestType_ = null;
      if (requestType) {
        if (requestType === 1 /* Critical */) {
          requestType_ = { type: "Critical" };
        } else {
          requestType_ = { type: "Informational" };
        }
      }
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "requestUserAttention",
              payload: requestType_
            }
          }
        }
      });
    }
    async setResizable(resizable) {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "setResizable",
              payload: resizable
            }
          }
        }
      });
    }
    async setTitle(title) {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "setTitle",
              payload: title
            }
          }
        }
      });
    }
    async maximize() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "maximize"
            }
          }
        }
      });
    }
    async unmaximize() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "unmaximize"
            }
          }
        }
      });
    }
    async toggleMaximize() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "toggleMaximize"
            }
          }
        }
      });
    }
    async minimize() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "minimize"
            }
          }
        }
      });
    }
    async unminimize() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "unminimize"
            }
          }
        }
      });
    }
    async show() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "show"
            }
          }
        }
      });
    }
    async hide() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "hide"
            }
          }
        }
      });
    }
    async close() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "close"
            }
          }
        }
      });
    }
    async setDecorations(decorations) {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "setDecorations",
              payload: decorations
            }
          }
        }
      });
    }
    async setAlwaysOnTop(alwaysOnTop) {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "setAlwaysOnTop",
              payload: alwaysOnTop
            }
          }
        }
      });
    }
    async setSize(size) {
      if (!size || size.type !== "Logical" && size.type !== "Physical") {
        throw new Error("the `size` argument must be either a LogicalSize or a PhysicalSize instance");
      }
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "setSize",
              payload: {
                type: size.type,
                data: {
                  width: size.width,
                  height: size.height
                }
              }
            }
          }
        }
      });
    }
    async setMinSize(size) {
      if (size && size.type !== "Logical" && size.type !== "Physical") {
        throw new Error("the `size` argument must be either a LogicalSize or a PhysicalSize instance");
      }
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "setMinSize",
              payload: size ? {
                type: size.type,
                data: {
                  width: size.width,
                  height: size.height
                }
              } : null
            }
          }
        }
      });
    }
    async setMaxSize(size) {
      if (size && size.type !== "Logical" && size.type !== "Physical") {
        throw new Error("the `size` argument must be either a LogicalSize or a PhysicalSize instance");
      }
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "setMaxSize",
              payload: size ? {
                type: size.type,
                data: {
                  width: size.width,
                  height: size.height
                }
              } : null
            }
          }
        }
      });
    }
    async setPosition(position) {
      if (!position || position.type !== "Logical" && position.type !== "Physical") {
        throw new Error("the `position` argument must be either a LogicalPosition or a PhysicalPosition instance");
      }
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "setPosition",
              payload: {
                type: position.type,
                data: {
                  x: position.x,
                  y: position.y
                }
              }
            }
          }
        }
      });
    }
    async setFullscreen(fullscreen) {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "setFullscreen",
              payload: fullscreen
            }
          }
        }
      });
    }
    async setFocus() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "setFocus"
            }
          }
        }
      });
    }
    async setIcon(icon) {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "setIcon",
              payload: {
                icon: typeof icon === "string" ? icon : Array.from(icon)
              }
            }
          }
        }
      });
    }
    async setSkipTaskbar(skip) {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "setSkipTaskbar",
              payload: skip
            }
          }
        }
      });
    }
    async startDragging() {
      return invokeTauriCommand({
        __tauriModule: "Window",
        message: {
          cmd: "manage",
          data: {
            label: this.label,
            cmd: {
              type: "startDragging"
            }
          }
        }
      });
    }
  };
  var WebviewWindow = class extends WindowManager {
    constructor(label, options = {}) {
      super(label);
      if (!(options == null ? void 0 : options.skip)) {
        invokeTauriCommand({
          __tauriModule: "Window",
          message: {
            cmd: "createWebview",
            data: {
              options: __spreadValues({
                label
              }, options)
            }
          }
        }).then(async () => this.emit("tauri://created")).catch(async (e) => this.emit("tauri://error", e));
      }
    }
    static getByLabel(label) {
      if (getAll().some((w) => w.label === label)) {
        return new WebviewWindow(label, { skip: true });
      }
      return null;
    }
  };
  var appWindow = new WebviewWindow(window.__TAURI_METADATA__.__currentWindow.label, {
    skip: true
  });
  async function currentMonitor() {
    return invokeTauriCommand({
      __tauriModule: "Window",
      message: {
        cmd: "manage",
        data: {
          cmd: {
            type: "currentMonitor"
          }
        }
      }
    });
  }
  async function primaryMonitor() {
    return invokeTauriCommand({
      __tauriModule: "Window",
      message: {
        cmd: "manage",
        data: {
          cmd: {
            type: "primaryMonitor"
          }
        }
      }
    });
  }
  async function availableMonitors() {
    return invokeTauriCommand({
      __tauriModule: "Window",
      message: {
        cmd: "manage",
        data: {
          cmd: {
            type: "availableMonitors"
          }
        }
      }
    });
  }

  // src/os.ts
  var os_exports = {};
  __export(os_exports, {
    EOL: () => EOL,
    arch: () => arch,
    platform: () => platform,
    tempdir: () => tempdir,
    type: () => type,
    version: () => version
  });
  var EOL = isWindows() ? "\r\n" : "\n";
  async function platform() {
    return invokeTauriCommand({
      __tauriModule: "Os",
      message: {
        cmd: "platform"
      }
    });
  }
  async function version() {
    return invokeTauriCommand({
      __tauriModule: "Os",
      message: {
        cmd: "version"
      }
    });
  }
  async function type() {
    return invokeTauriCommand({
      __tauriModule: "Os",
      message: {
        cmd: "osType"
      }
    });
  }
  async function arch() {
    return invokeTauriCommand({
      __tauriModule: "Os",
      message: {
        cmd: "arch"
      }
    });
  }
  async function tempdir() {
    return invokeTauriCommand({
      __tauriModule: "Os",
      message: {
        cmd: "tempdir"
      }
    });
  }
  return __toCommonJS(src_exports);
})();
