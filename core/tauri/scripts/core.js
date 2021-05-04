// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// polyfills
if (!String.prototype.startsWith) {
  String.prototype.startsWith = function (searchString, position) {
    position = position || 0;
    return this.substr(position, searchString.length) === searchString;
  };
}

(function () {
  function uid() {
    const length = new Int8Array(1)
    window.crypto.getRandomValues(length)
    const array = new Uint8Array(Math.max(16, Math.abs(length[0])))
    window.crypto.getRandomValues(array)
    return array.join('')
  }

  function ownKeys(object, enumerableOnly) {
    var keys = Object.keys(object);
    if (Object.getOwnPropertySymbols) {
      var symbols = Object.getOwnPropertySymbols(object);
      if (enumerableOnly)
        symbols = symbols.filter(function (sym) {
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
        Object.defineProperties(
          target,
          Object.getOwnPropertyDescriptors(source)
        );
      } else {
        ownKeys(source).forEach(function (key) {
          Object.defineProperty(
            target,
            key,
            Object.getOwnPropertyDescriptor(source, key)
          );
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
        writable: true,
      });
    } else {
      obj[key] = value;
    }
    return obj;
  }

  if (!window.__TAURI__) {
    window.__TAURI__ = {};
  }

  window.__TAURI__.transformCallback = function transformCallback(
    callback,
    once
  ) {
    var identifier = uid();

    window[identifier] = function (result) {
      if (once) {
        delete window[identifier];
      }

      return callback && callback(result);
    };

    return identifier;
  };

  window.__TAURI__.invoke = function invoke(cmd, args = {}) {
    return new Promise(function (resolve, reject) {
      var callback = window.__TAURI__.transformCallback(function (r) {
        resolve(r);
        delete window[error];
      }, true);
      var error = window.__TAURI__.transformCallback(function (e) {
        reject(e);
        delete window[callback];
      }, true);

      if (typeof cmd === "string") {
        args.cmd = cmd;
      } else if (typeof cmd === "object") {
        args = cmd;
      } else {
        return reject(new Error("Invalid argument type."));
      }

      if (window.rpc) {
        window.rpc.notify(
          cmd,
          _objectSpread(
            {
              callback: callback,
              error: error,
            },
            args
          )
        );
      } else {
        window.addEventListener("DOMContentLoaded", function () {
          window.rpc.notify(
            cmd,
            _objectSpread(
              {
                callback: callback,
                error: error,
              },
              args
            )
          );
        });
      }
    });
  };

  // open <a href="..."> links with the Tauri API
  function __openLinks() {
    document.querySelector("body").addEventListener(
      "click",
      function (e) {
        var target = e.target;
        while (target != null) {
          if (
            target.matches ? target.matches("a") : target.msMatchesSelector("a")
          ) {
            if (
              target.href &&
              target.href.startsWith("http") &&
              target.target === "_blank"
            ) {
              window.__TAURI__.invoke('tauri', {
                __tauriModule: "Shell",
                message: {
                  cmd: "open",
                  path: target.href,
                },
              });
              e.preventDefault();
            }
            break;
          }
          target = target.parentElement;
        }
      },
      true
    );
  }

  if (
    document.readyState === "complete" ||
    document.readyState === "interactive"
  ) {
    __openLinks();
  } else {
    window.addEventListener(
      "DOMContentLoaded",
      function () {
        __openLinks();
      },
      true
    );
  }

  // drag region
  document.addEventListener('mousedown', (e) => {
    // start dragging if the element has a `tauri-drag-region` data attribute
    if (e.target.dataset.tauriDragRegion === '' && e.buttons === 1) {
      window.__TAURI__.invoke('tauri', {
        __tauriModule: "Window",
        message: {
          cmd: "startDragging",
        }
      })
    }
  })

  window.__TAURI__.invoke('tauri', {
    __tauriModule: "Event",
    message: {
      cmd: "listen",
      event: "tauri://window-created",
      handler: window.__TAURI__.transformCallback(function (event) {
        if (event.payload) {
          var windowLabel = event.payload.label;
          window.__TAURI__.__windows.push({ label: windowLabel });
        }
      }),
    },
  });

  let permissionSettable = false;
  let permissionValue = "default";

  function isPermissionGranted() {
    if (window.Notification.permission !== "default") {
      return Promise.resolve(window.Notification.permission === "granted");
    }
    return window.__TAURI__.invoke('tauri', {
      __tauriModule: "Notification",
      message: {
        cmd: "isNotificationPermissionGranted",
      },
    });
  }

  function setNotificationPermission(value) {
    permissionSettable = true;
    window.Notification.permission = value;
    permissionSettable = false;
  }

  function requestPermission() {
    return window.__TAURI__
      .invoke('tauri', {
        __tauriModule: "Notification",
        mainThread: true,
        message: {
          cmd: "requestNotificationPermission",
        },
      })
      .then(function (permission) {
        setNotificationPermission(permission);
        return permission;
      });
  }

  function sendNotification(options) {
    if (typeof options === "object") {
      Object.freeze(options);
    }

    isPermissionGranted().then(function (permission) {
      if (permission) {
        return window.__TAURI__.invoke('tauri', {
          __tauriModule: "Notification",
          message: {
            cmd: "notification",
            options:
              typeof options === "string"
                ? {
                    title: options,
                  }
                : options,
          },
        });
      }
    });
  }

  window.Notification = function (title, options) {
    var opts = options || {};
    sendNotification(
      Object.assign(opts, {
        title: title,
      })
    );
  };

  window.Notification.requestPermission = requestPermission;

  Object.defineProperty(window.Notification, "permission", {
    enumerable: true,
    get: function () {
      return permissionValue;
    },
    set: function (v) {
      if (!permissionSettable) {
        throw new Error("Readonly property");
      }
      permissionValue = v;
    },
  });

  isPermissionGranted().then(function (response) {
    if (response === null) {
      setNotificationPermission("default");
    } else {
      setNotificationPermission(response ? "granted" : "denied");
    }
  });

  window.alert = function (message) {
    window.__TAURI__.invoke('tauri', {
      __tauriModule: "Dialog",
      mainThread: true,
      message: {
        cmd: "messageDialog",
        message: message,
      },
    });
  };

  window.confirm = function (message) {
    return window.__TAURI__.invoke('tauri', {
      __tauriModule: "Dialog",
      mainThread: true,
      message: {
        cmd: "askDialog",
        message: message,
      },
    });
  };
})();
