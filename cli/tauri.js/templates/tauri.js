// polyfills
if (!String.prototype.startsWith) {
  String.prototype.startsWith = function (searchString, position) {
    position = position || 0;
    return this.substr(position, searchString.length) === searchString;
  };
}

(function () {
  function s4() {
    return Math.floor((1 + Math.random()) * 0x10000)
      .toString(16)
      .substring(1);
  }

  var uid = function () {
    return (
      s4() +
      s4() +
      '-' +
      s4() +
      '-' +
      s4() +
      '-' +
      s4() +
      '-' +
      s4() +
      s4() +
      s4()
    );
  };

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
        writable: true
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

  window.__TAURI__.promisified = function promisified(args) {
    var _this = this;

    return new Promise(function (resolve, reject) {
      var callback = _this.transformCallback(function (r) {
        resolve(r);
        delete window[error];
      }, true);
      var error = _this.transformCallback(function (e) {
        reject(e);
        delete window[callback];
      }, true);

      window.__TAURI_INVOKE_HANDLER__(
        _objectSpread(
          {
            callback: callback,
            error: error
          },
          args
        )
      );
    });
  };

  window.__TAURI__.loadAsset = function loadAsset(assetName, assetType) {
    return this.promisified({
      cmd: 'loadAsset',
      asset: assetName,
      assetType: assetType || 'unknown'
    });
  };

  document.addEventListener(
    'error',
    function (e) {
      var target = e.target;
      while (target != null) {
        if (
          target.matches
            ? target.matches('img')
            : target.msMatchesSelector('img')
        ) {
          window.__TAURI__.loadAsset(target.src, 'image').then(function (img) {
            target.src = img;
          });
          break;
        }
        target = target.parentElement;
      }
    },
    true
  );

  // open <a href="..."> links with the Tauri API
  function __openLinks() {
    document.querySelector('body').addEventListener(
      'click',
      function (e) {
        var target = e.target;
        while (target != null) {
          if (
            target.matches ? target.matches('a') : target.msMatchesSelector('a')
          ) {
            if (
              target.href &&
              target.href.startsWith('http') &&
              target.target === '_blank'
            ) {
              window.__TAURI_INVOKE_HANDLER__({
                cmd: 'open',
                uri: target.href
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
    document.readyState === 'complete' ||
    document.readyState === 'interactive'
  ) {
    __openLinks();
  } else {
    window.addEventListener(
      'DOMContentLoaded',
      function () {
        __openLinks();
      },
      true
    );
  }

  let permissionSettable = false;
  let permissionValue = 'default';

  function isPermissionGranted() {
    if (window.Notification.permission !== 'default') {
      return Promise.resolve(window.Notification.permission === 'granted');
    }
    return window.__TAURI__.promisified({
      cmd: 'isNotificationPermissionGranted'
    });
  }

  function setNotificationPermission(value) {
    permissionSettable = true;
    window.Notification.permission = value;
    permissionSettable = false;
  }

  function requestPermission() {
    return window.__TAURI__
      .promisified({
        cmd: 'requestNotificationPermission'
      })
      .then(function (permission) {
        setNotificationPermission(permission);
        return permission;
      });
  }

  function sendNotification(options) {
    if (typeof options === 'object') {
      Object.freeze(options);
    }

    isPermissionGranted().then(function (permission) {
      if (permission) {
        return window.__TAURI__.promisified({
          cmd: 'notification',
          options:
            typeof options === 'string'
              ? {
                  title: options
                }
              : options
        });
      }
    });
  }

  window.Notification = function (title, options) {
    var opts = options || {};
    sendNotification(
      Object.assign(opts, {
        title: title
      })
    );
  };

  window.Notification.requestPermission = requestPermission;

  Object.defineProperty(window.Notification, 'permission', {
    enumerable: true,
    get: function () {
      return permissionValue;
    },
    set: function (v) {
      if (!permissionSettable) {
        throw new Error('Readonly property');
      }
      permissionValue = v;
    }
  });

  isPermissionGranted().then(function (response) {
    if (response === null) {
      setNotificationPermission('default');
    } else {
      setNotificationPermission(response ? 'granted' : 'denied');
    }
  });

  window.alert = function (message) {
    window.__TAURI_INVOKE_HANDLER__({
      cmd: 'messageDialog',
      message: message
    });
  };

  window.confirm = function (message) {
    return window.__TAURI__.promisified({
      cmd: 'askDialog',
      message: message
    });
  };
})();
