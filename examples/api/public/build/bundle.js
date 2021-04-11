(function (l, r) {
  if (l.getElementById("livereloadscript")) return;
  r = l.createElement("script");
  r.async = 1;
  r.src =
    "//" +
    (window.location.host || "localhost").split(":")[0] +
    ":35729/livereload.js?snipver=1";
  r.id = "livereloadscript";
  l.getElementsByTagName("head")[0].appendChild(r);
})(window.document);
var app = (function () {
  "use strict";

  function noop() {}
  function add_location(element, file, line, column, char) {
    element.__svelte_meta = {
      loc: { file, line, column, char },
    };
  }
  function run(fn) {
    return fn();
  }
  function blank_object() {
    return Object.create(null);
  }
  function run_all(fns) {
    fns.forEach(run);
  }
  function is_function(thing) {
    return typeof thing === "function";
  }
  function safe_not_equal(a, b) {
    return a != a
      ? b == b
      : a !== b || (a && typeof a === "object") || typeof a === "function";
  }
  function is_empty(obj) {
    return Object.keys(obj).length === 0;
  }
  function validate_store(store, name) {
    if (store != null && typeof store.subscribe !== "function") {
      throw new Error(`'${name}' is not a store with a 'subscribe' method`);
    }
  }
  function subscribe(store, ...callbacks) {
    if (store == null) {
      return noop;
    }
    const unsub = store.subscribe(...callbacks);
    return unsub.unsubscribe ? () => unsub.unsubscribe() : unsub;
  }
  function component_subscribe(component, store, callback) {
    component.$$.on_destroy.push(subscribe(store, callback));
  }

  function append(target, node) {
    target.appendChild(node);
  }
  function insert(target, node, anchor) {
    target.insertBefore(node, anchor || null);
  }
  function detach(node) {
    node.parentNode.removeChild(node);
  }
  function destroy_each(iterations, detaching) {
    for (let i = 0; i < iterations.length; i += 1) {
      if (iterations[i]) iterations[i].d(detaching);
    }
  }
  function element(name) {
    return document.createElement(name);
  }
  function text(data) {
    return document.createTextNode(data);
  }
  function space() {
    return text(" ");
  }
  function listen(node, event, handler, options) {
    node.addEventListener(event, handler, options);
    return () => node.removeEventListener(event, handler, options);
  }
  function prevent_default(fn) {
    return function (event) {
      event.preventDefault();
      // @ts-ignore
      return fn.call(this, event);
    };
  }
  function attr(node, attribute, value) {
    if (value == null) node.removeAttribute(attribute);
    else if (node.getAttribute(attribute) !== value)
      node.setAttribute(attribute, value);
  }
  function to_number(value) {
    return value === "" ? null : +value;
  }
  function children(element) {
    return Array.from(element.childNodes);
  }
  function set_input_value(input, value) {
    input.value = value == null ? "" : value;
  }
  function set_style(node, key, value, important) {
    node.style.setProperty(key, value, important ? "important" : "");
  }
  function select_option(select, value) {
    for (let i = 0; i < select.options.length; i += 1) {
      const option = select.options[i];
      if (option.__value === value) {
        option.selected = true;
        return;
      }
    }
  }
  function select_value(select) {
    const selected_option =
      select.querySelector(":checked") || select.options[0];
    return selected_option && selected_option.__value;
  }
  function custom_event(type, detail) {
    const e = document.createEvent("CustomEvent");
    e.initCustomEvent(type, false, false, detail);
    return e;
  }

  let current_component;
  function set_current_component(component) {
    current_component = component;
  }
  function get_current_component() {
    if (!current_component)
      throw new Error("Function called outside component initialization");
    return current_component;
  }
  function onMount(fn) {
    get_current_component().$$.on_mount.push(fn);
  }
  function onDestroy(fn) {
    get_current_component().$$.on_destroy.push(fn);
  }

  const dirty_components = [];
  const binding_callbacks = [];
  const render_callbacks = [];
  const flush_callbacks = [];
  const resolved_promise = Promise.resolve();
  let update_scheduled = false;
  function schedule_update() {
    if (!update_scheduled) {
      update_scheduled = true;
      resolved_promise.then(flush);
    }
  }
  function add_render_callback(fn) {
    render_callbacks.push(fn);
  }
  let flushing = false;
  const seen_callbacks = new Set();
  function flush() {
    if (flushing) return;
    flushing = true;
    do {
      // first, call beforeUpdate functions
      // and update components
      for (let i = 0; i < dirty_components.length; i += 1) {
        const component = dirty_components[i];
        set_current_component(component);
        update(component.$$);
      }
      set_current_component(null);
      dirty_components.length = 0;
      while (binding_callbacks.length) binding_callbacks.pop()();
      // then, once components are updated, call
      // afterUpdate functions. This may cause
      // subsequent updates...
      for (let i = 0; i < render_callbacks.length; i += 1) {
        const callback = render_callbacks[i];
        if (!seen_callbacks.has(callback)) {
          // ...so guard against infinite loops
          seen_callbacks.add(callback);
          callback();
        }
      }
      render_callbacks.length = 0;
    } while (dirty_components.length);
    while (flush_callbacks.length) {
      flush_callbacks.pop()();
    }
    update_scheduled = false;
    flushing = false;
    seen_callbacks.clear();
  }
  function update($$) {
    if ($$.fragment !== null) {
      $$.update();
      run_all($$.before_update);
      const dirty = $$.dirty;
      $$.dirty = [-1];
      $$.fragment && $$.fragment.p($$.ctx, dirty);
      $$.after_update.forEach(add_render_callback);
    }
  }
  const outroing = new Set();
  let outros;
  function group_outros() {
    outros = {
      r: 0,
      c: [],
      p: outros, // parent group
    };
  }
  function check_outros() {
    if (!outros.r) {
      run_all(outros.c);
    }
    outros = outros.p;
  }
  function transition_in(block, local) {
    if (block && block.i) {
      outroing.delete(block);
      block.i(local);
    }
  }
  function transition_out(block, local, detach, callback) {
    if (block && block.o) {
      if (outroing.has(block)) return;
      outroing.add(block);
      outros.c.push(() => {
        outroing.delete(block);
        if (callback) {
          if (detach) block.d(1);
          callback();
        }
      });
      block.o(local);
    }
  }

  const globals =
    typeof window !== "undefined"
      ? window
      : typeof globalThis !== "undefined"
      ? globalThis
      : global;
  function create_component(block) {
    block && block.c();
  }
  function mount_component(component, target, anchor, customElement) {
    const { fragment, on_mount, on_destroy, after_update } = component.$$;
    fragment && fragment.m(target, anchor);
    if (!customElement) {
      // onMount happens before the initial afterUpdate
      add_render_callback(() => {
        const new_on_destroy = on_mount.map(run).filter(is_function);
        if (on_destroy) {
          on_destroy.push(...new_on_destroy);
        } else {
          // Edge case - component was destroyed immediately,
          // most likely as a result of a binding initialising
          run_all(new_on_destroy);
        }
        component.$$.on_mount = [];
      });
    }
    after_update.forEach(add_render_callback);
  }
  function destroy_component(component, detaching) {
    const $$ = component.$$;
    if ($$.fragment !== null) {
      run_all($$.on_destroy);
      $$.fragment && $$.fragment.d(detaching);
      // TODO null out other refs, including component.$$ (but need to
      // preserve final state?)
      $$.on_destroy = $$.fragment = null;
      $$.ctx = [];
    }
  }
  function make_dirty(component, i) {
    if (component.$$.dirty[0] === -1) {
      dirty_components.push(component);
      schedule_update();
      component.$$.dirty.fill(0);
    }
    component.$$.dirty[(i / 31) | 0] |= 1 << i % 31;
  }
  function init(
    component,
    options,
    instance,
    create_fragment,
    not_equal,
    props,
    dirty = [-1]
  ) {
    const parent_component = current_component;
    set_current_component(component);
    const $$ = (component.$$ = {
      fragment: null,
      ctx: null,
      // state
      props,
      update: noop,
      not_equal,
      bound: blank_object(),
      // lifecycle
      on_mount: [],
      on_destroy: [],
      on_disconnect: [],
      before_update: [],
      after_update: [],
      context: new Map(parent_component ? parent_component.$$.context : []),
      // everything else
      callbacks: blank_object(),
      dirty,
      skip_bound: false,
    });
    let ready = false;
    $$.ctx = instance
      ? instance(component, options.props || {}, (i, ret, ...rest) => {
          const value = rest.length ? rest[0] : ret;
          if ($$.ctx && not_equal($$.ctx[i], ($$.ctx[i] = value))) {
            if (!$$.skip_bound && $$.bound[i]) $$.bound[i](value);
            if (ready) make_dirty(component, i);
          }
          return ret;
        })
      : [];
    $$.update();
    ready = true;
    run_all($$.before_update);
    // `false` as a special case of no DOM component
    $$.fragment = create_fragment ? create_fragment($$.ctx) : false;
    if (options.target) {
      if (options.hydrate) {
        const nodes = children(options.target);
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        $$.fragment && $$.fragment.l(nodes);
        nodes.forEach(detach);
      } else {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        $$.fragment && $$.fragment.c();
      }
      if (options.intro) transition_in(component.$$.fragment);
      mount_component(
        component,
        options.target,
        options.anchor,
        options.customElement
      );
      flush();
    }
    set_current_component(parent_component);
  }
  /**
   * Base class for Svelte components. Used when dev=false.
   */
  class SvelteComponent {
    $destroy() {
      destroy_component(this, 1);
      this.$destroy = noop;
    }
    $on(type, callback) {
      const callbacks =
        this.$$.callbacks[type] || (this.$$.callbacks[type] = []);
      callbacks.push(callback);
      return () => {
        const index = callbacks.indexOf(callback);
        if (index !== -1) callbacks.splice(index, 1);
      };
    }
    $set($$props) {
      if (this.$$set && !is_empty($$props)) {
        this.$$.skip_bound = true;
        this.$$set($$props);
        this.$$.skip_bound = false;
      }
    }
  }

  function dispatch_dev(type, detail) {
    document.dispatchEvent(
      custom_event(type, Object.assign({ version: "3.35.0" }, detail))
    );
  }
  function append_dev(target, node) {
    dispatch_dev("SvelteDOMInsert", { target, node });
    append(target, node);
  }
  function insert_dev(target, node, anchor) {
    dispatch_dev("SvelteDOMInsert", { target, node, anchor });
    insert(target, node, anchor);
  }
  function detach_dev(node) {
    dispatch_dev("SvelteDOMRemove", { node });
    detach(node);
  }
  function listen_dev(
    node,
    event,
    handler,
    options,
    has_prevent_default,
    has_stop_propagation
  ) {
    const modifiers =
      options === true
        ? ["capture"]
        : options
        ? Array.from(Object.keys(options))
        : [];
    if (has_prevent_default) modifiers.push("preventDefault");
    if (has_stop_propagation) modifiers.push("stopPropagation");
    dispatch_dev("SvelteDOMAddEventListener", {
      node,
      event,
      handler,
      modifiers,
    });
    const dispose = listen(node, event, handler, options);
    return () => {
      dispatch_dev("SvelteDOMRemoveEventListener", {
        node,
        event,
        handler,
        modifiers,
      });
      dispose();
    };
  }
  function attr_dev(node, attribute, value) {
    attr(node, attribute, value);
    if (value == null)
      dispatch_dev("SvelteDOMRemoveAttribute", { node, attribute });
    else dispatch_dev("SvelteDOMSetAttribute", { node, attribute, value });
  }
  function set_data_dev(text, data) {
    data = "" + data;
    if (text.wholeText === data) return;
    dispatch_dev("SvelteDOMSetData", { node: text, data });
    text.data = data;
  }
  function validate_each_argument(arg) {
    if (
      typeof arg !== "string" &&
      !(arg && typeof arg === "object" && "length" in arg)
    ) {
      let msg = "{#each} only iterates over array-like objects.";
      if (typeof Symbol === "function" && arg && Symbol.iterator in arg) {
        msg += " You can use a spread to convert this iterable into an array.";
      }
      throw new Error(msg);
    }
  }
  function validate_slots(name, slot, keys) {
    for (const slot_key of Object.keys(slot)) {
      if (!~keys.indexOf(slot_key)) {
        console.warn(`<${name}> received an unexpected slot "${slot_key}".`);
      }
    }
  }
  /**
   * Base class for Svelte components with some minor dev-enhancements. Used when dev=true.
   */
  class SvelteComponentDev extends SvelteComponent {
    constructor(options) {
      if (!options || (!options.target && !options.$$inline)) {
        throw new Error("'target' is a required option");
      }
      super();
    }
    $destroy() {
      super.$destroy();
      this.$destroy = () => {
        console.warn("Component was already destroyed"); // eslint-disable-line no-console
      };
    }
    $capture_state() {}
    $inject_state() {}
  }

  /*! *****************************************************************************
    Copyright (c) Microsoft Corporation.

    Permission to use, copy, modify, and/or distribute this software for any
    purpose with or without fee is hereby granted.

    THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
    REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
    AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
    INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
    LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
    OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
    PERFORMANCE OF THIS SOFTWARE.
    ***************************************************************************** */
  var t = function (n, e) {
    return (t =
      Object.setPrototypeOf ||
      ({ __proto__: [] } instanceof Array &&
        function (t, n) {
          t.__proto__ = n;
        }) ||
      function (t, n) {
        for (var e in n)
          Object.prototype.hasOwnProperty.call(n, e) && (t[e] = n[e]);
      })(n, e);
  };
  function n$5(n, e) {
    if ("function" != typeof e && null !== e)
      throw new TypeError(
        "Class extends value " + String(e) + " is not a constructor or null"
      );
    function r() {
      this.constructor = n;
    }
    t(n, e),
      (n.prototype =
        null === e ? Object.create(e) : ((r.prototype = e.prototype), new r()));
  }
  var e$2 = function () {
    return (e$2 =
      Object.assign ||
      function (t) {
        for (var n, e = 1, r = arguments.length; e < r; e++)
          for (var o in (n = arguments[e]))
            Object.prototype.hasOwnProperty.call(n, o) && (t[o] = n[o]);
        return t;
      }).apply(this, arguments);
  };
  function r$3(t, n, e, r) {
    return new (e || (e = Promise))(function (o, a) {
      function i(t) {
        try {
          u(r.next(t));
        } catch (t) {
          a(t);
        }
      }
      function c(t) {
        try {
          u(r.throw(t));
        } catch (t) {
          a(t);
        }
      }
      function u(t) {
        var n;
        t.done
          ? o(t.value)
          : ((n = t.value),
            n instanceof e
              ? n
              : new e(function (t) {
                  t(n);
                })).then(i, c);
      }
      u((r = r.apply(t, n || [])).next());
    });
  }
  function o$6(t, n) {
    var e,
      r,
      o,
      a,
      i = {
        label: 0,
        sent: function () {
          if (1 & o[0]) throw o[1];
          return o[1];
        },
        trys: [],
        ops: [],
      };
    return (
      (a = { next: c(0), throw: c(1), return: c(2) }),
      "function" == typeof Symbol &&
        (a[Symbol.iterator] = function () {
          return this;
        }),
      a
    );
    function c(a) {
      return function (c) {
        return (function (a) {
          if (e) throw new TypeError("Generator is already executing.");
          for (; i; )
            try {
              if (
                ((e = 1),
                r &&
                  (o =
                    2 & a[0]
                      ? r.return
                      : a[0]
                      ? r.throw || ((o = r.return) && o.call(r), 0)
                      : r.next) &&
                  !(o = o.call(r, a[1])).done)
              )
                return o;
              switch (((r = 0), o && (a = [2 & a[0], o.value]), a[0])) {
                case 0:
                case 1:
                  o = a;
                  break;
                case 4:
                  return i.label++, { value: a[1], done: !1 };
                case 5:
                  i.label++, (r = a[1]), (a = [0]);
                  continue;
                case 7:
                  (a = i.ops.pop()), i.trys.pop();
                  continue;
                default:
                  if (
                    !((o = i.trys),
                    (o = o.length > 0 && o[o.length - 1]) ||
                      (6 !== a[0] && 2 !== a[0]))
                  ) {
                    i = 0;
                    continue;
                  }
                  if (3 === a[0] && (!o || (a[1] > o[0] && a[1] < o[3]))) {
                    i.label = a[1];
                    break;
                  }
                  if (6 === a[0] && i.label < o[1]) {
                    (i.label = o[1]), (o = a);
                    break;
                  }
                  if (o && i.label < o[2]) {
                    (i.label = o[2]), i.ops.push(a);
                    break;
                  }
                  o[2] && i.ops.pop(), i.trys.pop();
                  continue;
              }
              a = n.call(t, i);
            } catch (t) {
              (a = [6, t]), (r = 0);
            } finally {
              e = o = 0;
            }
          if (5 & a[0]) throw a[1];
          return { value: a[0] ? a[1] : void 0, done: !0 };
        })([a, c]);
      };
    }
  }
  function a$5(t, n) {
    void 0 === n && (n = !1);
    var e = (function () {
      var t = new Int8Array(1);
      window.crypto.getRandomValues(t);
      var n = new Uint8Array(Math.max(16, Math.abs(t[0])));
      return window.crypto.getRandomValues(n), n.join("");
    })();
    return (
      Object.defineProperty(window, e, {
        value: function (r) {
          return (
            n && Reflect.deleteProperty(window, e), null == t ? void 0 : t(r)
          );
        },
        writable: !1,
        configurable: !0,
      }),
      e
    );
  }
  function i$3(t, n) {
    return (
      void 0 === n && (n = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (r) {
          return [
            2,
            new Promise(function (r, o) {
              var i = a$5(function (t) {
                  r(t), Reflect.deleteProperty(window, c);
                }, !0),
                c = a$5(function (t) {
                  o(t), Reflect.deleteProperty(window, i);
                }, !0);
              window.rpc.notify(t, e$2({ callback: i, error: c }, n));
            }),
          ];
        });
      })
    );
  }
  Object.freeze({ __proto__: null, transformCallback: a$5, invoke: i$3 });

  function n$4(n) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (i) {
        return [2, i$3("tauri", n)];
      });
    });
  }

  function o$5(n, o, s, u) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return (
          "object" == typeof u && Object.freeze(u),
          [
            2,
            n$4({
              __tauriModule: "Shell",
              message: {
                cmd: "execute",
                program: n,
                sidecar: o,
                onEventFn: a$5(s),
                args: "string" == typeof u ? [u] : u,
              },
            }),
          ]
        );
      });
    });
  }
  var s$4 = (function () {
      function t() {
        this.eventListeners = Object.create(null);
      }
      return (
        (t.prototype.addEventListener = function (t, e) {
          t in this.eventListeners
            ? this.eventListeners[t].push(e)
            : (this.eventListeners[t] = [e]);
        }),
        (t.prototype._emit = function (t, e) {
          if (t in this.eventListeners)
            for (var n = 0, r = this.eventListeners[t]; n < r.length; n++) {
              (0, r[n])(e);
            }
        }),
        (t.prototype.on = function (t, e) {
          return this.addEventListener(t, e), this;
        }),
        t
      );
    })(),
    u$6 = (function () {
      function n(t) {
        this.pid = t;
      }
      return (
        (n.prototype.write = function (n) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Shell",
                  message: { cmd: "stdinWrite", pid: this.pid, buffer: n },
                }),
              ];
            });
          });
        }),
        (n.prototype.kill = function () {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Shell",
                  message: { cmd: "killChild", pid: this.pid },
                }),
              ];
            });
          });
        }),
        n
      );
    })(),
    a$4 = (function (r) {
      function i(t, e) {
        void 0 === e && (e = []);
        var n = r.call(this) || this;
        return (
          (n.sidecar = !1),
          (n.stdout = new s$4()),
          (n.stderr = new s$4()),
          (n.pid = null),
          (n.program = t),
          (n.args = "string" == typeof e ? [e] : e),
          n
        );
      }
      return (
        n$5(i, r),
        (i.sidecar = function (t, e) {
          void 0 === e && (e = []);
          var n = new i(t, e);
          return (n.sidecar = !0), n;
        }),
        (i.prototype.spawn = function () {
          return r$3(this, void 0, void 0, function () {
            var t = this;
            return o$6(this, function (e) {
              return [
                2,
                o$5(
                  this.program,
                  this.sidecar,
                  function (e) {
                    switch (e.event) {
                      case "Error":
                        t._emit("error", e.payload);
                        break;
                      case "Terminated":
                        t._emit("close", e.payload);
                        break;
                      case "Stdout":
                        t.stdout._emit("data", e.payload);
                        break;
                      case "Stderr":
                        t.stderr._emit("data", e.payload);
                    }
                  },
                  this.args
                ).then(function (t) {
                  return new u$6(t);
                }),
              ];
            });
          });
        }),
        (i.prototype.execute = function () {
          return r$3(this, void 0, void 0, function () {
            var t = this;
            return o$6(this, function (e) {
              return [
                2,
                new Promise(function (e, n) {
                  t.on("error", n);
                  var r = [],
                    i = [];
                  t.stdout.on("data", function (t) {
                    r.push(t);
                  }),
                    t.stderr.on("data", function (t) {
                      i.push(t);
                    }),
                    t.on("close", function (t) {
                      e({
                        code: t.code,
                        signal: t.signal,
                        stdout: r.join("\n"),
                        stderr: i.join("\n"),
                      });
                    }),
                    t.spawn().catch(n);
                }),
              ];
            });
          });
        }),
        i
      );
    })(s$4);
  function d$3(n, r) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [
          2,
          n$4({
            __tauriModule: "Shell",
            message: { cmd: "open", path: n, with: r },
          }),
        ];
      });
    });
  }
  Object.freeze({ __proto__: null, Command: a$4, Child: u$6, open: d$3 });

  function r$2() {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [
          2,
          n$4({
            __tauriModule: "App",
            mainThread: !0,
            message: { cmd: "getAppVersion" },
          }),
        ];
      });
    });
  }
  function n$3() {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [
          2,
          n$4({
            __tauriModule: "App",
            mainThread: !0,
            message: { cmd: "getAppName" },
          }),
        ];
      });
    });
  }
  function u$5() {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [
          2,
          n$4({
            __tauriModule: "App",
            mainThread: !0,
            message: { cmd: "getTauriVersion" },
          }),
        ];
      });
    });
  }
  function o$4(r) {
    return (
      void 0 === r && (r = 0),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (t) {
          return [
            2,
            n$4({
              __tauriModule: "App",
              mainThread: !0,
              message: { cmd: "exit", exitCode: r },
            }),
          ];
        });
      })
    );
  }
  function a$3() {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [
          2,
          n$4({
            __tauriModule: "App",
            mainThread: !0,
            message: { cmd: "relaunch" },
          }),
        ];
      });
    });
  }
  Object.freeze({
    __proto__: null,
    getName: n$3,
    getVersion: r$2,
    getTauriVersion: u$5,
    relaunch: a$3,
    exit: o$4,
  });

  /* src/components/Welcome.svelte generated by Svelte v3.35.0 */

  const file$b = "src/components/Welcome.svelte";

  function create_fragment$b(ctx) {
    let h1;
    let t1;
    let p0;
    let t3;
    let p1;
    let t4;
    let t5;
    let t6;
    let p2;
    let t7;
    let t8;
    let t9;
    let p3;
    let t10;
    let t11;
    let t12;
    let button0;
    let t14;
    let button1;
    let mounted;
    let dispose;

    const block = {
      c: function create() {
        h1 = element("h1");
        h1.textContent = "Welcome";
        t1 = space();
        p0 = element("p");
        p0.textContent =
          "Tauri's API capabilities using the ` @tauri-apps/api ` package. It's used as\n  the main validation app, serving as the testbed of our development process. In\n  the future, this app will be used on Tauri's integration tests.";
        t3 = space();
        p1 = element("p");
        t4 = text("Current App version: ");
        t5 = text(/*version*/ ctx[0]);
        t6 = space();
        p2 = element("p");
        t7 = text("Current Tauri version: ");
        t8 = text(/*tauriVersion*/ ctx[1]);
        t9 = space();
        p3 = element("p");
        t10 = text("Current App name: ");
        t11 = text(/*appName*/ ctx[2]);
        t12 = space();
        button0 = element("button");
        button0.textContent = "Close application";
        t14 = space();
        button1 = element("button");
        button1.textContent = "Relaunch application";
        add_location(h1, file$b, 18, 0, 431);
        add_location(p0, file$b, 19, 0, 448);
        add_location(p1, file$b, 25, 0, 684);
        add_location(p2, file$b, 26, 0, 722);
        add_location(p3, file$b, 27, 0, 767);
        attr_dev(button0, "class", "button");
        add_location(button0, file$b, 29, 0, 803);
        attr_dev(button1, "class", "button");
        add_location(button1, file$b, 30, 0, 873);
      },
      l: function claim(nodes) {
        throw new Error(
          "options.hydrate only works if the component was compiled with the `hydratable: true` option"
        );
      },
      m: function mount(target, anchor) {
        insert_dev(target, h1, anchor);
        insert_dev(target, t1, anchor);
        insert_dev(target, p0, anchor);
        insert_dev(target, t3, anchor);
        insert_dev(target, p1, anchor);
        append_dev(p1, t4);
        append_dev(p1, t5);
        insert_dev(target, t6, anchor);
        insert_dev(target, p2, anchor);
        append_dev(p2, t7);
        append_dev(p2, t8);
        insert_dev(target, t9, anchor);
        insert_dev(target, p3, anchor);
        append_dev(p3, t10);
        append_dev(p3, t11);
        insert_dev(target, t12, anchor);
        insert_dev(target, button0, anchor);
        insert_dev(target, t14, anchor);
        insert_dev(target, button1, anchor);

        if (!mounted) {
          dispose = [
            listen_dev(
              button0,
              "click",
              /*closeApp*/ ctx[3],
              false,
              false,
              false
            ),
            listen_dev(
              button1,
              "click",
              /*relaunchApp*/ ctx[4],
              false,
              false,
              false
            ),
          ];

          mounted = true;
        }
      },
      p: function update(ctx, [dirty]) {
        if (dirty & /*version*/ 1) set_data_dev(t5, /*version*/ ctx[0]);
        if (dirty & /*tauriVersion*/ 2)
          set_data_dev(t8, /*tauriVersion*/ ctx[1]);
        if (dirty & /*appName*/ 4) set_data_dev(t11, /*appName*/ ctx[2]);
      },
      i: noop,
      o: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(h1);
        if (detaching) detach_dev(t1);
        if (detaching) detach_dev(p0);
        if (detaching) detach_dev(t3);
        if (detaching) detach_dev(p1);
        if (detaching) detach_dev(t6);
        if (detaching) detach_dev(p2);
        if (detaching) detach_dev(t9);
        if (detaching) detach_dev(p3);
        if (detaching) detach_dev(t12);
        if (detaching) detach_dev(button0);
        if (detaching) detach_dev(t14);
        if (detaching) detach_dev(button1);
        mounted = false;
        run_all(dispose);
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_fragment$b.name,
      type: "component",
      source: "",
      ctx,
    });

    return block;
  }

  function instance$b($$self, $$props, $$invalidate) {
    let { $$slots: slots = {}, $$scope } = $$props;
    validate_slots("Welcome", slots, []);
    let version = 0;
    let tauriVersion = 0;
    let appName = "Unknown";

    n$3().then((n) => {
      $$invalidate(2, (appName = n));
    });

    r$2().then((v) => {
      $$invalidate(0, (version = v));
    });

    u$5().then((v) => {
      $$invalidate(1, (tauriVersion = v));
    });

    async function closeApp() {
      await o$4();
    }

    async function relaunchApp() {
      await a$3();
    }

    const writable_props = [];

    Object.keys($$props).forEach((key) => {
      if (!~writable_props.indexOf(key) && key.slice(0, 2) !== "$$")
        console.warn(`<Welcome> was created with unknown prop '${key}'`);
    });

    $$self.$capture_state = () => ({
      getName: n$3,
      getVersion: r$2,
      getTauriVersion: u$5,
      relaunch: a$3,
      exit: o$4,
      version,
      tauriVersion,
      appName,
      closeApp,
      relaunchApp,
    });

    $$self.$inject_state = ($$props) => {
      if ("version" in $$props) $$invalidate(0, (version = $$props.version));
      if ("tauriVersion" in $$props)
        $$invalidate(1, (tauriVersion = $$props.tauriVersion));
      if ("appName" in $$props) $$invalidate(2, (appName = $$props.appName));
    };

    if ($$props && "$$inject" in $$props) {
      $$self.$inject_state($$props.$$inject);
    }

    return [version, tauriVersion, appName, closeApp, relaunchApp];
  }

  class Welcome extends SvelteComponentDev {
    constructor(options) {
      super(options);
      init(this, options, instance$b, create_fragment$b, safe_not_equal, {});

      dispatch_dev("SvelteRegisterComponent", {
        component: this,
        tagName: "Welcome",
        options,
        id: create_fragment$b.name,
      });
    }
  }

  function e$1() {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [
          2,
          n$4({ __tauriModule: "Cli", message: { cmd: "cliMatches" } }),
        ];
      });
    });
  }
  Object.freeze({ __proto__: null, getMatches: e$1 });

  /* src/components/Cli.svelte generated by Svelte v3.35.0 */
  const file$a = "src/components/Cli.svelte";

  function create_fragment$a(ctx) {
    let div;
    let button;
    let mounted;
    let dispose;

    const block = {
      c: function create() {
        div = element("div");
        button = element("button");
        button.textContent = "Get matches";
        attr_dev(button, "class", "button");
        attr_dev(button, "id", "cli-matches");
        add_location(button, file$a, 11, 2, 187);
        add_location(div, file$a, 10, 0, 179);
      },
      l: function claim(nodes) {
        throw new Error(
          "options.hydrate only works if the component was compiled with the `hydratable: true` option"
        );
      },
      m: function mount(target, anchor) {
        insert_dev(target, div, anchor);
        append_dev(div, button);

        if (!mounted) {
          dispose = listen_dev(
            button,
            "click",
            /*cliMatches*/ ctx[0],
            false,
            false,
            false
          );
          mounted = true;
        }
      },
      p: noop,
      i: noop,
      o: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(div);
        mounted = false;
        dispose();
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_fragment$a.name,
      type: "component",
      source: "",
      ctx,
    });

    return block;
  }

  function instance$a($$self, $$props, $$invalidate) {
    let { $$slots: slots = {}, $$scope } = $$props;
    validate_slots("Cli", slots, []);
    let { onMessage } = $$props;

    function cliMatches() {
      e$1().then(onMessage).catch(onMessage);
    }

    const writable_props = ["onMessage"];

    Object.keys($$props).forEach((key) => {
      if (!~writable_props.indexOf(key) && key.slice(0, 2) !== "$$")
        console.warn(`<Cli> was created with unknown prop '${key}'`);
    });

    $$self.$$set = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(1, (onMessage = $$props.onMessage));
    };

    $$self.$capture_state = () => ({ getMatches: e$1, onMessage, cliMatches });

    $$self.$inject_state = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(1, (onMessage = $$props.onMessage));
    };

    if ($$props && "$$inject" in $$props) {
      $$self.$inject_state($$props.$$inject);
    }

    return [cliMatches, onMessage];
  }

  class Cli extends SvelteComponentDev {
    constructor(options) {
      super(options);
      init(this, options, instance$a, create_fragment$a, safe_not_equal, {
        onMessage: 1,
      });

      dispatch_dev("SvelteRegisterComponent", {
        component: this,
        tagName: "Cli",
        options,
        id: create_fragment$a.name,
      });

      const { ctx } = this.$$;
      const props = options.props || {};

      if (/*onMessage*/ ctx[1] === undefined && !("onMessage" in props)) {
        console.warn("<Cli> was created without expected prop 'onMessage'");
      }
    }

    get onMessage() {
      throw new Error(
        "<Cli>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }

    set onMessage(value) {
      throw new Error(
        "<Cli>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }
  }

  function e(e, o) {
    return r$3(this, void 0, void 0, function () {
      var s = this;
      return o$6(this, function (c) {
        return [
          2,
          n$4({
            __tauriModule: "Event",
            message: { cmd: "listen", event: e, handler: a$5(o) },
          }).then(function (i) {
            return function () {
              return r$3(s, void 0, void 0, function () {
                return o$6(this, function (n) {
                  return [2, u$4(i)];
                });
              });
            };
          }),
        ];
      });
    });
  }
  function u$4(i) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (n) {
        return [
          2,
          n$4({
            __tauriModule: "Event",
            message: { cmd: "unlisten", eventId: i },
          }),
        ];
      });
    });
  }
  function o$3(i, r) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (n) {
        return [2, e(i, r)];
      });
    });
  }
  function s$3(i, r) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (n) {
        return [
          2,
          e(i, function (n) {
            r(n), u$4(n.id).catch(function () {});
          }),
        ];
      });
    });
  }
  function c$3(i, e, u) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (n) {
        switch (n.label) {
          case 0:
            return [
              4,
              n$4({
                __tauriModule: "Event",
                message: { cmd: "emit", event: i, windowLabel: e, payload: u },
              }),
            ];
          case 1:
            return n.sent(), [2];
        }
      });
    });
  }

  function n$2(r, i) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [2, c$3(r, void 0, i)];
      });
    });
  }
  Object.freeze({ __proto__: null, emit: n$2, listen: o$3, once: s$3 });

  /* src/components/Communication.svelte generated by Svelte v3.35.0 */
  const file$9 = "src/components/Communication.svelte";

  function create_fragment$9(ctx) {
    let div;
    let button0;
    let t1;
    let button1;
    let t3;
    let button2;
    let mounted;
    let dispose;

    const block = {
      c: function create() {
        div = element("div");
        button0 = element("button");
        button0.textContent = "Call Log API";
        t1 = space();
        button1 = element("button");
        button1.textContent = "Call Request (async) API";
        t3 = space();
        button2 = element("button");
        button2.textContent = "Send event to Rust";
        attr_dev(button0, "class", "button");
        attr_dev(button0, "id", "log");
        add_location(button0, file$9, 42, 2, 839);
        attr_dev(button1, "class", "button");
        attr_dev(button1, "id", "request");
        add_location(button1, file$9, 43, 2, 910);
        attr_dev(button2, "class", "button");
        attr_dev(button2, "id", "event");
        add_location(button2, file$9, 46, 2, 1016);
        add_location(div, file$9, 41, 0, 831);
      },
      l: function claim(nodes) {
        throw new Error(
          "options.hydrate only works if the component was compiled with the `hydratable: true` option"
        );
      },
      m: function mount(target, anchor) {
        insert_dev(target, div, anchor);
        append_dev(div, button0);
        append_dev(div, t1);
        append_dev(div, button1);
        append_dev(div, t3);
        append_dev(div, button2);

        if (!mounted) {
          dispose = [
            listen_dev(button0, "click", /*log*/ ctx[0], false, false, false),
            listen_dev(
              button1,
              "click",
              /*performRequest*/ ctx[1],
              false,
              false,
              false
            ),
            listen_dev(
              button2,
              "click",
              /*emitEvent*/ ctx[2],
              false,
              false,
              false
            ),
          ];

          mounted = true;
        }
      },
      p: noop,
      i: noop,
      o: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(div);
        mounted = false;
        run_all(dispose);
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_fragment$9.name,
      type: "component",
      source: "",
      ctx,
    });

    return block;
  }

  function instance$9($$self, $$props, $$invalidate) {
    let { $$slots: slots = {}, $$scope } = $$props;
    validate_slots("Communication", slots, []);
    let { onMessage } = $$props;
    let unlisten;

    onMount(async () => {
      unlisten = await o$3("rust-event", onMessage);
    });

    onDestroy(() => {
      if (unlisten) {
        unlisten();
      }
    });

    function log() {
      i$3("log_operation", {
        event: "tauri-click",
        payload: "this payload is optional because we used Option in Rust",
      });
    }

    function performRequest() {
      i$3("perform_request", {
        endpoint: "dummy endpoint arg",
        body: { id: 5, name: "test" },
      })
        .then(onMessage)
        .catch(onMessage);
    }

    function emitEvent() {
      n$2("js-event", "this is the payload string");
    }

    const writable_props = ["onMessage"];

    Object.keys($$props).forEach((key) => {
      if (!~writable_props.indexOf(key) && key.slice(0, 2) !== "$$")
        console.warn(`<Communication> was created with unknown prop '${key}'`);
    });

    $$self.$$set = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(3, (onMessage = $$props.onMessage));
    };

    $$self.$capture_state = () => ({
      listen: o$3,
      emit: n$2,
      invoke: i$3,
      onMount,
      onDestroy,
      onMessage,
      unlisten,
      log,
      performRequest,
      emitEvent,
    });

    $$self.$inject_state = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(3, (onMessage = $$props.onMessage));
      if ("unlisten" in $$props) unlisten = $$props.unlisten;
    };

    if ($$props && "$$inject" in $$props) {
      $$self.$inject_state($$props.$$inject);
    }

    return [log, performRequest, emitEvent, onMessage];
  }

  class Communication extends SvelteComponentDev {
    constructor(options) {
      super(options);
      init(this, options, instance$9, create_fragment$9, safe_not_equal, {
        onMessage: 3,
      });

      dispatch_dev("SvelteRegisterComponent", {
        component: this,
        tagName: "Communication",
        options,
        id: create_fragment$9.name,
      });

      const { ctx } = this.$$;
      const props = options.props || {};

      if (/*onMessage*/ ctx[3] === undefined && !("onMessage" in props)) {
        console.warn(
          "<Communication> was created without expected prop 'onMessage'"
        );
      }
    }

    get onMessage() {
      throw new Error(
        "<Communication>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }

    set onMessage(value) {
      throw new Error(
        "<Communication>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }
  }

  function i$2(i) {
    return (
      void 0 === i && (i = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (o) {
          return (
            "object" == typeof i && Object.freeze(i),
            [
              2,
              n$4({
                __tauriModule: "Dialog",
                mainThread: !0,
                message: { cmd: "openDialog", options: i },
              }),
            ]
          );
        });
      })
    );
  }
  function r$1(i) {
    return (
      void 0 === i && (i = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (o) {
          return (
            "object" == typeof i && Object.freeze(i),
            [
              2,
              n$4({
                __tauriModule: "Dialog",
                mainThread: !0,
                message: { cmd: "saveDialog", options: i },
              }),
            ]
          );
        });
      })
    );
  }
  Object.freeze({ __proto__: null, open: i$2, save: r$1 });

  var r;
  function o$2(r, o) {
    return (
      void 0 === o && (o = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (t) {
          return [
            2,
            n$4({
              __tauriModule: "Fs",
              message: { cmd: "readTextFile", path: r, options: o },
            }),
          ];
        });
      })
    );
  }
  function n$1(r, o) {
    return (
      void 0 === o && (o = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (t) {
          return [
            2,
            n$4({
              __tauriModule: "Fs",
              message: { cmd: "readBinaryFile", path: r, options: o },
            }),
          ];
        });
      })
    );
  }
  function u$3(r, o) {
    return (
      void 0 === o && (o = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (t) {
          return (
            "object" == typeof o && Object.freeze(o),
            "object" == typeof r && Object.freeze(r),
            [
              2,
              n$4({
                __tauriModule: "Fs",
                message: {
                  cmd: "writeFile",
                  path: r.path,
                  contents: r.contents,
                  options: o,
                },
              }),
            ]
          );
        });
      })
    );
  }
  !(function (t) {
    (t[(t.Audio = 1)] = "Audio"),
      (t[(t.Cache = 2)] = "Cache"),
      (t[(t.Config = 3)] = "Config"),
      (t[(t.Data = 4)] = "Data"),
      (t[(t.LocalData = 5)] = "LocalData"),
      (t[(t.Desktop = 6)] = "Desktop"),
      (t[(t.Document = 7)] = "Document"),
      (t[(t.Download = 8)] = "Download"),
      (t[(t.Executable = 9)] = "Executable"),
      (t[(t.Font = 10)] = "Font"),
      (t[(t.Home = 11)] = "Home"),
      (t[(t.Picture = 12)] = "Picture"),
      (t[(t.Public = 13)] = "Public"),
      (t[(t.Runtime = 14)] = "Runtime"),
      (t[(t.Template = 15)] = "Template"),
      (t[(t.Video = 16)] = "Video"),
      (t[(t.Resource = 17)] = "Resource"),
      (t[(t.App = 18)] = "App"),
      (t[(t.Current = 19)] = "Current");
  })(r || (r = {}));
  function a$2(t) {
    var e = (function (t) {
      if (t.length < 65536)
        return String.fromCharCode.apply(null, Array.from(t));
      for (var e = "", i = t.length, r = 0; r < i; r++) {
        var o = t.subarray(65536 * r, 65536 * (r + 1));
        e += String.fromCharCode.apply(null, Array.from(o));
      }
      return e;
    })(new Uint8Array(t));
    return btoa(e);
  }
  function s$2(r, o) {
    return (
      void 0 === o && (o = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (t) {
          return (
            "object" == typeof o && Object.freeze(o),
            "object" == typeof r && Object.freeze(r),
            [
              2,
              n$4({
                __tauriModule: "Fs",
                message: {
                  cmd: "writeBinaryFile",
                  path: r.path,
                  contents: a$2(r.contents),
                  options: o,
                },
              }),
            ]
          );
        });
      })
    );
  }
  function c$2(r, o) {
    return (
      void 0 === o && (o = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (t) {
          return [
            2,
            n$4({
              __tauriModule: "Fs",
              message: { cmd: "readDir", path: r, options: o },
            }),
          ];
        });
      })
    );
  }
  function d$2(r, o) {
    return (
      void 0 === o && (o = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (t) {
          return [
            2,
            n$4({
              __tauriModule: "Fs",
              message: { cmd: "createDir", path: r, options: o },
            }),
          ];
        });
      })
    );
  }
  function f$1(r, o) {
    return (
      void 0 === o && (o = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (t) {
          return [
            2,
            n$4({
              __tauriModule: "Fs",
              message: { cmd: "removeDir", path: r, options: o },
            }),
          ];
        });
      })
    );
  }
  function l$1(r, o, n) {
    return (
      void 0 === n && (n = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (t) {
          return [
            2,
            n$4({
              __tauriModule: "Fs",
              message: {
                cmd: "copyFile",
                source: r,
                destination: o,
                options: n,
              },
            }),
          ];
        });
      })
    );
  }
  function m(r, o) {
    return (
      void 0 === o && (o = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (t) {
          return [
            2,
            n$4({
              __tauriModule: "Fs",
              message: { cmd: "removeFile", path: r, options: o },
            }),
          ];
        });
      })
    );
  }
  function p(r, o, n) {
    return (
      void 0 === n && (n = {}),
      r$3(this, void 0, void 0, function () {
        return o$6(this, function (t) {
          return [
            2,
            n$4({
              __tauriModule: "Fs",
              message: {
                cmd: "renameFile",
                oldPath: r,
                newPath: o,
                options: n,
              },
            }),
          ];
        });
      })
    );
  }
  Object.freeze({
    __proto__: null,
    get BaseDirectory() {
      return r;
    },
    get Dir() {
      return r;
    },
    readTextFile: o$2,
    readBinaryFile: n$1,
    writeFile: u$3,
    writeBinaryFile: s$2,
    readDir: c$2,
    createDir: d$2,
    removeDir: f$1,
    copyFile: l$1,
    removeFile: m,
    renameFile: p,
  });

  /* src/components/Dialog.svelte generated by Svelte v3.35.0 */
  const file$8 = "src/components/Dialog.svelte";

  function add_css$1() {
    var style = element("style");
    style.id = "svelte-1eg58yg-style";
    style.textContent =
      "#dialog-filter.svelte-1eg58yg{width:260px}\n/*# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiRGlhbG9nLnN2ZWx0ZSIsInNvdXJjZXMiOlsiRGlhbG9nLnN2ZWx0ZSJdLCJzb3VyY2VzQ29udGVudCI6WyI8c2NyaXB0PlxuICBpbXBvcnQgeyBvcGVuLCBzYXZlIH0gZnJvbSBcIkB0YXVyaS1hcHBzL2FwaS9kaWFsb2dcIjtcbiAgaW1wb3J0IHsgcmVhZEJpbmFyeUZpbGUgfSBmcm9tIFwiQHRhdXJpLWFwcHMvYXBpL2ZzXCI7XG5cbiAgZXhwb3J0IGxldCBvbk1lc3NhZ2U7XG4gIGxldCBkZWZhdWx0UGF0aCA9IG51bGw7XG4gIGxldCBmaWx0ZXIgPSBudWxsO1xuICBsZXQgbXVsdGlwbGUgPSBmYWxzZTtcbiAgbGV0IGRpcmVjdG9yeSA9IGZhbHNlO1xuXG4gIGZ1bmN0aW9uIGFycmF5QnVmZmVyVG9CYXNlNjQoYnVmZmVyLCBjYWxsYmFjaykge1xuICAgIHZhciBibG9iID0gbmV3IEJsb2IoW2J1ZmZlcl0sIHtcbiAgICAgIHR5cGU6IFwiYXBwbGljYXRpb24vb2N0ZXQtYmluYXJ5XCIsXG4gICAgfSk7XG4gICAgdmFyIHJlYWRlciA9IG5ldyBGaWxlUmVhZGVyKCk7XG4gICAgcmVhZGVyLm9ubG9hZCA9IGZ1bmN0aW9uIChldnQpIHtcbiAgICAgIHZhciBkYXRhdXJsID0gZXZ0LnRhcmdldC5yZXN1bHQ7XG4gICAgICBjYWxsYmFjayhkYXRhdXJsLnN1YnN0cihkYXRhdXJsLmluZGV4T2YoXCIsXCIpICsgMSkpO1xuICAgIH07XG4gICAgcmVhZGVyLnJlYWRBc0RhdGFVUkwoYmxvYik7XG4gIH1cblxuICBmdW5jdGlvbiBvcGVuRGlhbG9nKCkge1xuICAgIG9wZW4oe1xuICAgICAgZGVmYXVsdFBhdGgsXG4gICAgICBmaWx0ZXJzOiBmaWx0ZXJcbiAgICAgICAgPyBbXG4gICAgICAgICAgICB7XG4gICAgICAgICAgICAgIG5hbWU6IFwiVGF1cmkgRXhhbXBsZVwiLFxuICAgICAgICAgICAgICBleHRlbnNpb25zOiBmaWx0ZXIuc3BsaXQoXCIsXCIpLm1hcCgoZikgPT4gZi50cmltKCkpLFxuICAgICAgICAgICAgfSxcbiAgICAgICAgICBdXG4gICAgICAgIDogW10sXG4gICAgICBtdWx0aXBsZSxcbiAgICAgIGRpcmVjdG9yeSxcbiAgICB9KVxuICAgICAgLnRoZW4oZnVuY3Rpb24gKHJlcykge1xuICAgICAgICBpZiAoQXJyYXkuaXNBcnJheShyZXMpKSB7XG4gICAgICAgICAgb25NZXNzYWdlKHJlcyk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgdmFyIHBhdGhUb1JlYWQgPSByZXM7XG4gICAgICAgICAgdmFyIGlzRmlsZSA9IHBhdGhUb1JlYWQubWF0Y2goL1xcUytcXC5cXFMrJC9nKTtcbiAgICAgICAgICByZWFkQmluYXJ5RmlsZShwYXRoVG9SZWFkKVxuICAgICAgICAgICAgLnRoZW4oZnVuY3Rpb24gKHJlc3BvbnNlKSB7XG4gICAgICAgICAgICAgIGlmIChpc0ZpbGUpIHtcbiAgICAgICAgICAgICAgICBpZiAoXG4gICAgICAgICAgICAgICAgICBwYXRoVG9SZWFkLmluY2x1ZGVzKFwiLnBuZ1wiKSB8fFxuICAgICAgICAgICAgICAgICAgcGF0aFRvUmVhZC5pbmNsdWRlcyhcIi5qcGdcIilcbiAgICAgICAgICAgICAgICApIHtcbiAgICAgICAgICAgICAgICAgIGFycmF5QnVmZmVyVG9CYXNlNjQoXG4gICAgICAgICAgICAgICAgICAgIG5ldyBVaW50OEFycmF5KHJlc3BvbnNlKSxcbiAgICAgICAgICAgICAgICAgICAgZnVuY3Rpb24gKGJhc2U2NCkge1xuICAgICAgICAgICAgICAgICAgICAgIHZhciBzcmMgPSBcImRhdGE6aW1hZ2UvcG5nO2Jhc2U2NCxcIiArIGJhc2U2NDtcbiAgICAgICAgICAgICAgICAgICAgICBvbk1lc3NhZ2UoJzxpbWcgc3JjPVwiJyArIHNyYyArICdcIj48L2ltZz4nKTtcbiAgICAgICAgICAgICAgICAgICAgfVxuICAgICAgICAgICAgICAgICAgKTtcbiAgICAgICAgICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgICAgICAgICAgb25NZXNzYWdlKHJlcyk7XG4gICAgICAgICAgICAgICAgfVxuICAgICAgICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgICAgICAgIG9uTWVzc2FnZShyZXMpO1xuICAgICAgICAgICAgICB9XG4gICAgICAgICAgICB9KVxuICAgICAgICAgICAgLmNhdGNoKG9uTWVzc2FnZShyZXMpKTtcbiAgICAgICAgfVxuICAgICAgfSlcbiAgICAgIC5jYXRjaChvbk1lc3NhZ2UpO1xuICB9XG5cbiAgZnVuY3Rpb24gc2F2ZURpYWxvZygpIHtcbiAgICBzYXZlKHtcbiAgICAgIGRlZmF1bHRQYXRoLFxuICAgICAgZmlsdGVyczogZmlsdGVyXG4gICAgICAgID8gW1xuICAgICAgICAgICAge1xuICAgICAgICAgICAgICBuYW1lOiBcIlRhdXJpIEV4YW1wbGVcIixcbiAgICAgICAgICAgICAgZXh0ZW5zaW9uczogZmlsdGVyLnNwbGl0KFwiLFwiKS5tYXAoKGYpID0+IGYudHJpbSgpKSxcbiAgICAgICAgICAgIH0sXG4gICAgICAgICAgXVxuICAgICAgICA6IFtdLFxuICAgIH0pXG4gICAgICAudGhlbihvbk1lc3NhZ2UpXG4gICAgICAuY2F0Y2gob25NZXNzYWdlKTtcbiAgfVxuPC9zY3JpcHQ+XG5cbjxkaXY+XG4gIDxpbnB1dFxuICAgIGlkPVwiZGlhbG9nLWRlZmF1bHQtcGF0aFwiXG4gICAgcGxhY2Vob2xkZXI9XCJEZWZhdWx0IHBhdGhcIlxuICAgIGJpbmQ6dmFsdWU9e2RlZmF1bHRQYXRofVxuICAvPlxuICA8aW5wdXRcbiAgICBpZD1cImRpYWxvZy1maWx0ZXJcIlxuICAgIHBsYWNlaG9sZGVyPVwiRXh0ZW5zaW9ucyBmaWx0ZXIsIGNvbW1hLXNlcGFyYXRlZFwiXG4gICAgYmluZDp2YWx1ZT17ZmlsdGVyfVxuICAvPlxuICA8ZGl2PlxuICAgIDxpbnB1dCB0eXBlPVwiY2hlY2tib3hcIiBpZD1cImRpYWxvZy1tdWx0aXBsZVwiIGJpbmQ6Y2hlY2tlZD17bXVsdGlwbGV9IC8+XG4gICAgPGxhYmVsIGZvcj1cImRpYWxvZy1tdWx0aXBsZVwiPk11bHRpcGxlPC9sYWJlbD5cbiAgPC9kaXY+XG4gIDxkaXY+XG4gICAgPGlucHV0IHR5cGU9XCJjaGVja2JveFwiIGlkPVwiZGlhbG9nLWRpcmVjdG9yeVwiIGJpbmQ6Y2hlY2tlZD17ZGlyZWN0b3J5fSAvPlxuICAgIDxsYWJlbCBmb3I9XCJkaWFsb2ctZGlyZWN0b3J5XCI+RGlyZWN0b3J5PC9sYWJlbD5cbiAgPC9kaXY+XG5cbiAgPGJ1dHRvbiBjbGFzcz1cImJ1dHRvblwiIGlkPVwib3Blbi1kaWFsb2dcIiBvbjpjbGljaz17b3BlbkRpYWxvZ31cbiAgICA+T3BlbiBkaWFsb2c8L2J1dHRvblxuICA+XG4gIDxidXR0b24gY2xhc3M9XCJidXR0b25cIiBpZD1cInNhdmUtZGlhbG9nXCIgb246Y2xpY2s9e3NhdmVEaWFsb2d9XG4gICAgPk9wZW4gc2F2ZSBkaWFsb2c8L2J1dHRvblxuICA+XG48L2Rpdj5cblxuPHN0eWxlPlxuICAjZGlhbG9nLWZpbHRlciB7XG4gICAgd2lkdGg6IDI2MHB4O1xuICB9XG48L3N0eWxlPlxuIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQW1IRSxjQUFjLGVBQUMsQ0FBQyxBQUNkLEtBQUssQ0FBRSxLQUFLLEFBQ2QsQ0FBQyJ9 */";
    append_dev(document.head, style);
  }

  function create_fragment$8(ctx) {
    let div2;
    let input0;
    let t0;
    let input1;
    let t1;
    let div0;
    let input2;
    let t2;
    let label0;
    let t4;
    let div1;
    let input3;
    let t5;
    let label1;
    let t7;
    let button0;
    let t9;
    let button1;
    let mounted;
    let dispose;

    const block = {
      c: function create() {
        div2 = element("div");
        input0 = element("input");
        t0 = space();
        input1 = element("input");
        t1 = space();
        div0 = element("div");
        input2 = element("input");
        t2 = space();
        label0 = element("label");
        label0.textContent = "Multiple";
        t4 = space();
        div1 = element("div");
        input3 = element("input");
        t5 = space();
        label1 = element("label");
        label1.textContent = "Directory";
        t7 = space();
        button0 = element("button");
        button0.textContent = "Open dialog";
        t9 = space();
        button1 = element("button");
        button1.textContent = "Open save dialog";
        attr_dev(input0, "id", "dialog-default-path");
        attr_dev(input0, "placeholder", "Default path");
        add_location(input0, file$8, 87, 2, 2186);
        attr_dev(input1, "id", "dialog-filter");
        attr_dev(input1, "placeholder", "Extensions filter, comma-separated");
        attr_dev(input1, "class", "svelte-1eg58yg");
        add_location(input1, file$8, 92, 2, 2289);
        attr_dev(input2, "type", "checkbox");
        attr_dev(input2, "id", "dialog-multiple");
        add_location(input2, file$8, 98, 4, 2413);
        attr_dev(label0, "for", "dialog-multiple");
        add_location(label0, file$8, 99, 4, 2488);
        add_location(div0, file$8, 97, 2, 2403);
        attr_dev(input3, "type", "checkbox");
        attr_dev(input3, "id", "dialog-directory");
        add_location(input3, file$8, 102, 4, 2555);
        attr_dev(label1, "for", "dialog-directory");
        add_location(label1, file$8, 103, 4, 2632);
        add_location(div1, file$8, 101, 2, 2545);
        attr_dev(button0, "class", "button");
        attr_dev(button0, "id", "open-dialog");
        add_location(button0, file$8, 106, 2, 2692);
        attr_dev(button1, "class", "button");
        attr_dev(button1, "id", "save-dialog");
        add_location(button1, file$8, 109, 2, 2785);
        add_location(div2, file$8, 86, 0, 2178);
      },
      l: function claim(nodes) {
        throw new Error(
          "options.hydrate only works if the component was compiled with the `hydratable: true` option"
        );
      },
      m: function mount(target, anchor) {
        insert_dev(target, div2, anchor);
        append_dev(div2, input0);
        set_input_value(input0, /*defaultPath*/ ctx[0]);
        append_dev(div2, t0);
        append_dev(div2, input1);
        set_input_value(input1, /*filter*/ ctx[1]);
        append_dev(div2, t1);
        append_dev(div2, div0);
        append_dev(div0, input2);
        input2.checked = /*multiple*/ ctx[2];
        append_dev(div0, t2);
        append_dev(div0, label0);
        append_dev(div2, t4);
        append_dev(div2, div1);
        append_dev(div1, input3);
        input3.checked = /*directory*/ ctx[3];
        append_dev(div1, t5);
        append_dev(div1, label1);
        append_dev(div2, t7);
        append_dev(div2, button0);
        append_dev(div2, t9);
        append_dev(div2, button1);

        if (!mounted) {
          dispose = [
            listen_dev(input0, "input", /*input0_input_handler*/ ctx[7]),
            listen_dev(input1, "input", /*input1_input_handler*/ ctx[8]),
            listen_dev(input2, "change", /*input2_change_handler*/ ctx[9]),
            listen_dev(input3, "change", /*input3_change_handler*/ ctx[10]),
            listen_dev(
              button0,
              "click",
              /*openDialog*/ ctx[4],
              false,
              false,
              false
            ),
            listen_dev(
              button1,
              "click",
              /*saveDialog*/ ctx[5],
              false,
              false,
              false
            ),
          ];

          mounted = true;
        }
      },
      p: function update(ctx, [dirty]) {
        if (
          dirty & /*defaultPath*/ 1 &&
          input0.value !== /*defaultPath*/ ctx[0]
        ) {
          set_input_value(input0, /*defaultPath*/ ctx[0]);
        }

        if (dirty & /*filter*/ 2 && input1.value !== /*filter*/ ctx[1]) {
          set_input_value(input1, /*filter*/ ctx[1]);
        }

        if (dirty & /*multiple*/ 4) {
          input2.checked = /*multiple*/ ctx[2];
        }

        if (dirty & /*directory*/ 8) {
          input3.checked = /*directory*/ ctx[3];
        }
      },
      i: noop,
      o: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(div2);
        mounted = false;
        run_all(dispose);
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_fragment$8.name,
      type: "component",
      source: "",
      ctx,
    });

    return block;
  }

  function arrayBufferToBase64$1(buffer, callback) {
    var blob = new Blob([buffer], { type: "application/octet-binary" });
    var reader = new FileReader();

    reader.onload = function (evt) {
      var dataurl = evt.target.result;
      callback(dataurl.substr(dataurl.indexOf(",") + 1));
    };

    reader.readAsDataURL(blob);
  }

  function instance$8($$self, $$props, $$invalidate) {
    let { $$slots: slots = {}, $$scope } = $$props;
    validate_slots("Dialog", slots, []);
    let { onMessage } = $$props;
    let defaultPath = null;
    let filter = null;
    let multiple = false;
    let directory = false;

    function openDialog() {
      i$2({
        defaultPath,
        filters: filter
          ? [
              {
                name: "Tauri Example",
                extensions: filter.split(",").map((f) => f.trim()),
              },
            ]
          : [],
        multiple,
        directory,
      })
        .then(function (res) {
          if (Array.isArray(res)) {
            onMessage(res);
          } else {
            var pathToRead = res;
            var isFile = pathToRead.match(/\S+\.\S+$/g);

            n$1(pathToRead)
              .then(function (response) {
                if (isFile) {
                  if (
                    pathToRead.includes(".png") ||
                    pathToRead.includes(".jpg")
                  ) {
                    arrayBufferToBase64$1(
                      new Uint8Array(response),
                      function (base64) {
                        var src = "data:image/png;base64," + base64;
                        onMessage('<img src="' + src + '"></img>');
                      }
                    );
                  } else {
                    onMessage(res);
                  }
                } else {
                  onMessage(res);
                }
              })
              .catch(onMessage(res));
          }
        })
        .catch(onMessage);
    }

    function saveDialog() {
      r$1({
        defaultPath,
        filters: filter
          ? [
              {
                name: "Tauri Example",
                extensions: filter.split(",").map((f) => f.trim()),
              },
            ]
          : [],
      })
        .then(onMessage)
        .catch(onMessage);
    }

    const writable_props = ["onMessage"];

    Object.keys($$props).forEach((key) => {
      if (!~writable_props.indexOf(key) && key.slice(0, 2) !== "$$")
        console.warn(`<Dialog> was created with unknown prop '${key}'`);
    });

    function input0_input_handler() {
      defaultPath = this.value;
      $$invalidate(0, defaultPath);
    }

    function input1_input_handler() {
      filter = this.value;
      $$invalidate(1, filter);
    }

    function input2_change_handler() {
      multiple = this.checked;
      $$invalidate(2, multiple);
    }

    function input3_change_handler() {
      directory = this.checked;
      $$invalidate(3, directory);
    }

    $$self.$$set = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(6, (onMessage = $$props.onMessage));
    };

    $$self.$capture_state = () => ({
      open: i$2,
      save: r$1,
      readBinaryFile: n$1,
      onMessage,
      defaultPath,
      filter,
      multiple,
      directory,
      arrayBufferToBase64: arrayBufferToBase64$1,
      openDialog,
      saveDialog,
    });

    $$self.$inject_state = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(6, (onMessage = $$props.onMessage));
      if ("defaultPath" in $$props)
        $$invalidate(0, (defaultPath = $$props.defaultPath));
      if ("filter" in $$props) $$invalidate(1, (filter = $$props.filter));
      if ("multiple" in $$props) $$invalidate(2, (multiple = $$props.multiple));
      if ("directory" in $$props)
        $$invalidate(3, (directory = $$props.directory));
    };

    if ($$props && "$$inject" in $$props) {
      $$self.$inject_state($$props.$$inject);
    }

    return [
      defaultPath,
      filter,
      multiple,
      directory,
      openDialog,
      saveDialog,
      onMessage,
      input0_input_handler,
      input1_input_handler,
      input2_change_handler,
      input3_change_handler,
    ];
  }

  class Dialog extends SvelteComponentDev {
    constructor(options) {
      super(options);
      if (!document.getElementById("svelte-1eg58yg-style")) add_css$1();
      init(this, options, instance$8, create_fragment$8, safe_not_equal, {
        onMessage: 6,
      });

      dispatch_dev("SvelteRegisterComponent", {
        component: this,
        tagName: "Dialog",
        options,
        id: create_fragment$8.name,
      });

      const { ctx } = this.$$;
      const props = options.props || {};

      if (/*onMessage*/ ctx[6] === undefined && !("onMessage" in props)) {
        console.warn("<Dialog> was created without expected prop 'onMessage'");
      }
    }

    get onMessage() {
      throw new Error(
        "<Dialog>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }

    set onMessage(value) {
      throw new Error(
        "<Dialog>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }
  }

  /* src/components/FileSystem.svelte generated by Svelte v3.35.0 */

  const { Object: Object_1 } = globals;
  const file$7 = "src/components/FileSystem.svelte";

  function get_each_context$2(ctx, list, i) {
    const child_ctx = ctx.slice();
    child_ctx[5] = list[i];
    return child_ctx;
  }

  // (79:4) {#each DirOptions as dir}
  function create_each_block$2(ctx) {
    let option;
    let t_value = /*dir*/ ctx[5][0] + "";
    let t;

    const block = {
      c: function create() {
        option = element("option");
        t = text(t_value);
        option.__value = /*dir*/ ctx[5][1];
        option.value = option.__value;
        add_location(option, file$7, 79, 6, 2408);
      },
      m: function mount(target, anchor) {
        insert_dev(target, option, anchor);
        append_dev(option, t);
      },
      p: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(option);
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_each_block$2.name,
      type: "each",
      source: "(79:4) {#each DirOptions as dir}",
      ctx,
    });

    return block;
  }

  function create_fragment$7(ctx) {
    let form;
    let select;
    let option;
    let t1;
    let input;
    let t2;
    let button;
    let mounted;
    let dispose;
    let each_value = /*DirOptions*/ ctx[1];
    validate_each_argument(each_value);
    let each_blocks = [];

    for (let i = 0; i < each_value.length; i += 1) {
      each_blocks[i] = create_each_block$2(
        get_each_context$2(ctx, each_value, i)
      );
    }

    const block = {
      c: function create() {
        form = element("form");
        select = element("select");
        option = element("option");
        option.textContent = "None";

        for (let i = 0; i < each_blocks.length; i += 1) {
          each_blocks[i].c();
        }

        t1 = space();
        input = element("input");
        t2 = space();
        button = element("button");
        button.textContent = "Read";
        option.__value = "";
        option.value = option.__value;
        add_location(option, file$7, 77, 4, 2341);
        attr_dev(select, "class", "button");
        attr_dev(select, "id", "dir");
        add_location(select, file$7, 76, 2, 2304);
        attr_dev(input, "id", "path-to-read");
        attr_dev(input, "placeholder", "Type the path to read...");
        add_location(input, file$7, 82, 2, 2475);
        attr_dev(button, "class", "button");
        attr_dev(button, "id", "read");
        add_location(button, file$7, 87, 2, 2582);
        add_location(form, file$7, 75, 0, 2263);
      },
      l: function claim(nodes) {
        throw new Error(
          "options.hydrate only works if the component was compiled with the `hydratable: true` option"
        );
      },
      m: function mount(target, anchor) {
        insert_dev(target, form, anchor);
        append_dev(form, select);
        append_dev(select, option);

        for (let i = 0; i < each_blocks.length; i += 1) {
          each_blocks[i].m(select, null);
        }

        append_dev(form, t1);
        append_dev(form, input);
        set_input_value(input, /*pathToRead*/ ctx[0]);
        append_dev(form, t2);
        append_dev(form, button);

        if (!mounted) {
          dispose = [
            listen_dev(input, "input", /*input_input_handler*/ ctx[4]),
            listen_dev(
              form,
              "submit",
              prevent_default(/*read*/ ctx[2]),
              false,
              true,
              false
            ),
          ];

          mounted = true;
        }
      },
      p: function update(ctx, [dirty]) {
        if (dirty & /*DirOptions*/ 2) {
          each_value = /*DirOptions*/ ctx[1];
          validate_each_argument(each_value);
          let i;

          for (i = 0; i < each_value.length; i += 1) {
            const child_ctx = get_each_context$2(ctx, each_value, i);

            if (each_blocks[i]) {
              each_blocks[i].p(child_ctx, dirty);
            } else {
              each_blocks[i] = create_each_block$2(child_ctx);
              each_blocks[i].c();
              each_blocks[i].m(select, null);
            }
          }

          for (; i < each_blocks.length; i += 1) {
            each_blocks[i].d(1);
          }

          each_blocks.length = each_value.length;
        }

        if (dirty & /*pathToRead*/ 1 && input.value !== /*pathToRead*/ ctx[0]) {
          set_input_value(input, /*pathToRead*/ ctx[0]);
        }
      },
      i: noop,
      o: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(form);
        destroy_each(each_blocks, detaching);
        mounted = false;
        run_all(dispose);
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_fragment$7.name,
      type: "component",
      source: "",
      ctx,
    });

    return block;
  }

  function getDir() {
    const dirSelect = document.getElementById("dir");
    return dirSelect.value ? parseInt(dir.value) : null;
  }

  function arrayBufferToBase64(buffer, callback) {
    const blob = new Blob([buffer], { type: "application/octet-binary" });
    const reader = new FileReader();

    reader.onload = function (evt) {
      const dataurl = evt.target.result;
      callback(dataurl.substr(dataurl.indexOf(",") + 1));
    };

    reader.readAsDataURL(blob);
  }

  function instance$7($$self, $$props, $$invalidate) {
    let { $$slots: slots = {}, $$scope } = $$props;
    validate_slots("FileSystem", slots, []);
    let { onMessage } = $$props;
    let pathToRead = "";
    const DirOptions = Object.keys(r)
      .filter((key) => isNaN(parseInt(key)))
      .map((dir) => [dir, r[dir]]);

    function read() {
      const isFile = pathToRead.match(/\S+\.\S+$/g);
      const opts = { dir: getDir() };

      const promise = isFile ? n$1(pathToRead, opts) : c$2(pathToRead, opts);

      promise
        .then(function (response) {
          if (isFile) {
            if (pathToRead.includes(".png") || pathToRead.includes(".jpg")) {
              arrayBufferToBase64(new Uint8Array(response), function (base64) {
                const src = "data:image/png;base64," + base64;
                onMessage('<img src="' + src + '"></img>');
              });
            } else {
              const value = String.fromCharCode.apply(null, response);
              onMessage(
                '<textarea id="file-response" style="height: 400px"></textarea><button id="file-save">Save</button>'
              );

              setTimeout(() => {
                const fileInput = document.getElementById("file-response");
                fileInput.value = value;

                document
                  .getElementById("file-save")
                  .addEventListener("click", function () {
                    writeFile(
                      {
                        file: pathToRead,
                        contents: fileInput.value,
                      },
                      { dir: getDir() }
                    ).catch(onMessage);
                  });
              });
            }
          } else {
            onMessage(response);
          }
        })
        .catch(onMessage);
    }

    const writable_props = ["onMessage"];

    Object_1.keys($$props).forEach((key) => {
      if (!~writable_props.indexOf(key) && key.slice(0, 2) !== "$$")
        console.warn(`<FileSystem> was created with unknown prop '${key}'`);
    });

    function input_input_handler() {
      pathToRead = this.value;
      $$invalidate(0, pathToRead);
    }

    $$self.$$set = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(3, (onMessage = $$props.onMessage));
    };

    $$self.$capture_state = () => ({
      readBinaryFile: n$1,
      readDir: c$2,
      Dir: r,
      onMessage,
      pathToRead,
      getDir,
      arrayBufferToBase64,
      DirOptions,
      read,
    });

    $$self.$inject_state = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(3, (onMessage = $$props.onMessage));
      if ("pathToRead" in $$props)
        $$invalidate(0, (pathToRead = $$props.pathToRead));
    };

    if ($$props && "$$inject" in $$props) {
      $$self.$inject_state($$props.$$inject);
    }

    return [pathToRead, DirOptions, read, onMessage, input_input_handler];
  }

  class FileSystem extends SvelteComponentDev {
    constructor(options) {
      super(options);
      init(this, options, instance$7, create_fragment$7, safe_not_equal, {
        onMessage: 3,
      });

      dispatch_dev("SvelteRegisterComponent", {
        component: this,
        tagName: "FileSystem",
        options,
        id: create_fragment$7.name,
      });

      const { ctx } = this.$$;
      const props = options.props || {};

      if (/*onMessage*/ ctx[3] === undefined && !("onMessage" in props)) {
        console.warn(
          "<FileSystem> was created without expected prop 'onMessage'"
        );
      }
    }

    get onMessage() {
      throw new Error(
        "<FileSystem>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }

    set onMessage(value) {
      throw new Error(
        "<FileSystem>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }
  }

  var i$1;
  !(function (t) {
    (t[(t.JSON = 1)] = "JSON"),
      (t[(t.Text = 2)] = "Text"),
      (t[(t.Binary = 3)] = "Binary");
  })(i$1 || (i$1 = {}));
  var o$1 = (function () {
      function t(t, n) {
        (this.type = t), (this.payload = n);
      }
      return (
        (t.form = function (n) {
          return new t("Form", n);
        }),
        (t.json = function (n) {
          return new t("Json", n);
        }),
        (t.text = function (n) {
          return new t("Text", n);
        }),
        (t.bytes = function (n) {
          return new t("Bytes", n);
        }),
        t
      );
    })(),
    u$2 = (function () {
      function i(t) {
        this.id = t;
      }
      return (
        (i.prototype.drop = function () {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Http",
                  message: { cmd: "dropClient", client: this.id },
                }),
              ];
            });
          });
        }),
        (i.prototype.request = function (e) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Http",
                  message: { cmd: "httpRequest", client: this.id, options: e },
                }),
              ];
            });
          });
        }),
        (i.prototype.get = function (r, i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [2, this.request(e$2({ method: "GET", url: r }, i))];
            });
          });
        }),
        (i.prototype.post = function (r, i, o) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                this.request(e$2({ method: "POST", url: r, body: i }, o)),
              ];
            });
          });
        }),
        (i.prototype.put = function (r, i, o) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                this.request(e$2({ method: "PUT", url: r, body: i }, o)),
              ];
            });
          });
        }),
        (i.prototype.patch = function (r, i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [2, this.request(e$2({ method: "PATCH", url: r }, i))];
            });
          });
        }),
        (i.prototype.delete = function (r, i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [2, this.request(e$2({ method: "DELETE", url: r }, i))];
            });
          });
        }),
        i
      );
    })();
  function s$1(e) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [
          2,
          n$4({
            __tauriModule: "Http",
            message: { cmd: "createClient", options: e },
          }).then(function (t) {
            return new u$2(t);
          }),
        ];
      });
    });
  }
  var c$1 = null;
  function d$1(r, i) {
    var o;
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        switch (t.label) {
          case 0:
            return null !== c$1 ? [3, 2] : [4, s$1()];
          case 1:
            (c$1 = t.sent()), (t.label = 2);
          case 2:
            return [
              2,
              c$1.request(
                e$2(
                  {
                    url: r,
                    method:
                      null !== (o = null == i ? void 0 : i.method) &&
                      void 0 !== o
                        ? o
                        : "GET",
                  },
                  i
                )
              ),
            ];
        }
      });
    });
  }
  Object.freeze({
    __proto__: null,
    get ResponseType() {
      return i$1;
    },
    Body: o$1,
    Client: u$2,
    getClient: s$1,
    fetch: d$1,
  });

  /* src/components/Http.svelte generated by Svelte v3.35.0 */
  const file$6 = "src/components/Http.svelte";

  function create_fragment$6(ctx) {
    let form;
    let select;
    let option0;
    let option1;
    let option2;
    let option3;
    let option4;
    let t5;
    let input;
    let t6;
    let br;
    let t7;
    let textarea;
    let t8;
    let button;
    let mounted;
    let dispose;

    const block = {
      c: function create() {
        form = element("form");
        select = element("select");
        option0 = element("option");
        option0.textContent = "GET";
        option1 = element("option");
        option1.textContent = "POST";
        option2 = element("option");
        option2.textContent = "PUT";
        option3 = element("option");
        option3.textContent = "PATCH";
        option4 = element("option");
        option4.textContent = "DELETE";
        t5 = space();
        input = element("input");
        t6 = space();
        br = element("br");
        t7 = space();
        textarea = element("textarea");
        t8 = space();
        button = element("button");
        button.textContent = "Make request";
        option0.__value = "GET";
        option0.value = option0.__value;
        add_location(option0, file$6, 33, 4, 862);
        option1.__value = "POST";
        option1.value = option1.__value;
        add_location(option1, file$6, 34, 4, 899);
        option2.__value = "PUT";
        option2.value = option2.__value;
        add_location(option2, file$6, 35, 4, 938);
        option3.__value = "PATCH";
        option3.value = option3.__value;
        add_location(option3, file$6, 36, 4, 975);
        option4.__value = "DELETE";
        option4.value = option4.__value;
        add_location(option4, file$6, 37, 4, 1016);
        attr_dev(select, "class", "button");
        attr_dev(select, "id", "request-method");
        if (/*httpMethod*/ ctx[0] === void 0)
          add_render_callback(() =>
            /*select_change_handler*/ ctx[5].call(select)
          );
        add_location(select, file$6, 32, 2, 790);
        attr_dev(input, "id", "request-url");
        attr_dev(input, "placeholder", "Type the request URL...");
        add_location(input, file$6, 39, 2, 1069);
        add_location(br, file$6, 44, 2, 1171);
        attr_dev(textarea, "id", "request-body");
        attr_dev(textarea, "placeholder", "Request body");
        attr_dev(textarea, "rows", "5");
        set_style(textarea, "width", "100%");
        set_style(textarea, "margin-right", "10px");
        set_style(textarea, "font-size", "12px");
        add_location(textarea, file$6, 45, 2, 1180);
        attr_dev(button, "class", "button");
        attr_dev(button, "id", "make-request");
        add_location(button, file$6, 52, 2, 1345);
        add_location(form, file$6, 31, 0, 738);
      },
      l: function claim(nodes) {
        throw new Error(
          "options.hydrate only works if the component was compiled with the `hydratable: true` option"
        );
      },
      m: function mount(target, anchor) {
        insert_dev(target, form, anchor);
        append_dev(form, select);
        append_dev(select, option0);
        append_dev(select, option1);
        append_dev(select, option2);
        append_dev(select, option3);
        append_dev(select, option4);
        select_option(select, /*httpMethod*/ ctx[0]);
        append_dev(form, t5);
        append_dev(form, input);
        set_input_value(input, /*httpUrl*/ ctx[1]);
        append_dev(form, t6);
        append_dev(form, br);
        append_dev(form, t7);
        append_dev(form, textarea);
        set_input_value(textarea, /*httpBody*/ ctx[2]);
        append_dev(form, t8);
        append_dev(form, button);

        if (!mounted) {
          dispose = [
            listen_dev(select, "change", /*select_change_handler*/ ctx[5]),
            listen_dev(input, "input", /*input_input_handler*/ ctx[6]),
            listen_dev(textarea, "input", /*textarea_input_handler*/ ctx[7]),
            listen_dev(
              form,
              "submit",
              prevent_default(/*makeHttpRequest*/ ctx[3]),
              false,
              true,
              false
            ),
          ];

          mounted = true;
        }
      },
      p: function update(ctx, [dirty]) {
        if (dirty & /*httpMethod*/ 1) {
          select_option(select, /*httpMethod*/ ctx[0]);
        }

        if (dirty & /*httpUrl*/ 2 && input.value !== /*httpUrl*/ ctx[1]) {
          set_input_value(input, /*httpUrl*/ ctx[1]);
        }

        if (dirty & /*httpBody*/ 4) {
          set_input_value(textarea, /*httpBody*/ ctx[2]);
        }
      },
      i: noop,
      o: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(form);
        mounted = false;
        run_all(dispose);
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_fragment$6.name,
      type: "component",
      source: "",
      ctx,
    });

    return block;
  }

  function instance$6($$self, $$props, $$invalidate) {
    let { $$slots: slots = {}, $$scope } = $$props;
    validate_slots("Http", slots, []);
    let httpMethod = "GET";
    let httpUrl = "";
    let httpBody = "";
    let { onMessage } = $$props;

    async function makeHttpRequest() {
      const client = await s$1();
      let method = httpMethod || "GET";
      let url = httpUrl || "";
      const options = { url: url || "", method: method || "GET" };

      if (
        (httpBody.startsWith("{") && httpBody.endsWith("}")) ||
        (httpBody.startsWith("[") && httpBody.endsWith("]"))
      ) {
        options.body = o$1.json(JSON.parse(httpBody));
      } else if (httpBody !== "") {
        options.body = o$1.text(httpBody);
      }

      client.request(options).then(onMessage).catch(onMessage);
    }

    const writable_props = ["onMessage"];

    Object.keys($$props).forEach((key) => {
      if (!~writable_props.indexOf(key) && key.slice(0, 2) !== "$$")
        console.warn(`<Http> was created with unknown prop '${key}'`);
    });

    function select_change_handler() {
      httpMethod = select_value(this);
      $$invalidate(0, httpMethod);
    }

    function input_input_handler() {
      httpUrl = this.value;
      $$invalidate(1, httpUrl);
    }

    function textarea_input_handler() {
      httpBody = this.value;
      $$invalidate(2, httpBody);
    }

    $$self.$$set = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(4, (onMessage = $$props.onMessage));
    };

    $$self.$capture_state = () => ({
      getClient: s$1,
      Body: o$1,
      httpMethod,
      httpUrl,
      httpBody,
      onMessage,
      makeHttpRequest,
    });

    $$self.$inject_state = ($$props) => {
      if ("httpMethod" in $$props)
        $$invalidate(0, (httpMethod = $$props.httpMethod));
      if ("httpUrl" in $$props) $$invalidate(1, (httpUrl = $$props.httpUrl));
      if ("httpBody" in $$props) $$invalidate(2, (httpBody = $$props.httpBody));
      if ("onMessage" in $$props)
        $$invalidate(4, (onMessage = $$props.onMessage));
    };

    if ($$props && "$$inject" in $$props) {
      $$self.$inject_state($$props.$$inject);
    }

    return [
      httpMethod,
      httpUrl,
      httpBody,
      makeHttpRequest,
      onMessage,
      select_change_handler,
      input_input_handler,
      textarea_input_handler,
    ];
  }

  class Http extends SvelteComponentDev {
    constructor(options) {
      super(options);
      init(this, options, instance$6, create_fragment$6, safe_not_equal, {
        onMessage: 4,
      });

      dispatch_dev("SvelteRegisterComponent", {
        component: this,
        tagName: "Http",
        options,
        id: create_fragment$6.name,
      });

      const { ctx } = this.$$;
      const props = options.props || {};

      if (/*onMessage*/ ctx[4] === undefined && !("onMessage" in props)) {
        console.warn("<Http> was created without expected prop 'onMessage'");
      }
    }

    get onMessage() {
      throw new Error(
        "<Http>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }

    set onMessage(value) {
      throw new Error(
        "<Http>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }
  }

  /* src/components/Notifications.svelte generated by Svelte v3.35.0 */

  const file$5 = "src/components/Notifications.svelte";

  function create_fragment$5(ctx) {
    let button;
    let mounted;
    let dispose;

    const block = {
      c: function create() {
        button = element("button");
        button.textContent = "Send test notification";
        attr_dev(button, "class", "button");
        attr_dev(button, "id", "notification");
        add_location(button, file$5, 28, 0, 678);
      },
      l: function claim(nodes) {
        throw new Error(
          "options.hydrate only works if the component was compiled with the `hydratable: true` option"
        );
      },
      m: function mount(target, anchor) {
        insert_dev(target, button, anchor);

        if (!mounted) {
          dispose = listen_dev(
            button,
            "click",
            /*sendNotification*/ ctx[0],
            false,
            false,
            false
          );
          mounted = true;
        }
      },
      p: noop,
      i: noop,
      o: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(button);
        mounted = false;
        dispose();
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_fragment$5.name,
      type: "component",
      source: "",
      ctx,
    });

    return block;
  }

  function _sendNotification() {
    new Notification("Notification title", {
      body: "This is the notification body",
    });
  }

  function instance$5($$self, $$props, $$invalidate) {
    let { $$slots: slots = {}, $$scope } = $$props;
    validate_slots("Notifications", slots, []);
    let { onMessage } = $$props;

    function sendNotification() {
      if (Notification.permission === "default") {
        Notification.requestPermission()
          .then(function (response) {
            if (response === "granted") {
              _sendNotification();
            } else {
              onMessage("Permission is " + response);
            }
          })
          .catch(onMessage);
      } else if (Notification.permission === "granted") {
        _sendNotification();
      } else {
        onMessage("Permission is denied");
      }
    }

    const writable_props = ["onMessage"];

    Object.keys($$props).forEach((key) => {
      if (!~writable_props.indexOf(key) && key.slice(0, 2) !== "$$")
        console.warn(`<Notifications> was created with unknown prop '${key}'`);
    });

    $$self.$$set = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(1, (onMessage = $$props.onMessage));
    };

    $$self.$capture_state = () => ({
      onMessage,
      _sendNotification,
      sendNotification,
    });

    $$self.$inject_state = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(1, (onMessage = $$props.onMessage));
    };

    if ($$props && "$$inject" in $$props) {
      $$self.$inject_state($$props.$$inject);
    }

    return [sendNotification, onMessage];
  }

  class Notifications extends SvelteComponentDev {
    constructor(options) {
      super(options);
      init(this, options, instance$5, create_fragment$5, safe_not_equal, {
        onMessage: 1,
      });

      dispatch_dev("SvelteRegisterComponent", {
        component: this,
        tagName: "Notifications",
        options,
        id: create_fragment$5.name,
      });

      const { ctx } = this.$$;
      const props = options.props || {};

      if (/*onMessage*/ ctx[1] === undefined && !("onMessage" in props)) {
        console.warn(
          "<Notifications> was created without expected prop 'onMessage'"
        );
      }
    }

    get onMessage() {
      throw new Error(
        "<Notifications>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }

    set onMessage(value) {
      throw new Error(
        "<Notifications>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }
  }

  function d() {
    return new f(window.__TAURI__.__currentWindow.label);
  }
  function c() {
    return window.__TAURI__.__windows;
  }
  var a$1 = ["tauri://created", "tauri://error"],
    f = (function () {
      function i(t) {
        (this.label = t), (this.listeners = Object.create(null));
      }
      return (
        (i.prototype.listen = function (i, e) {
          return r$3(this, void 0, void 0, function () {
            var t = this;
            return o$6(this, function (n) {
              return this._handleTauriEvent(i, e)
                ? [
                    2,
                    Promise.resolve(function () {
                      var n = t.listeners[i];
                      n.splice(n.indexOf(e), 1);
                    }),
                  ]
                : [2, o$3(i, e)];
            });
          });
        }),
        (i.prototype.once = function (i, e) {
          return r$3(this, void 0, void 0, function () {
            var t = this;
            return o$6(this, function (n) {
              return this._handleTauriEvent(i, e)
                ? [
                    2,
                    Promise.resolve(function () {
                      var n = t.listeners[i];
                      n.splice(n.indexOf(e), 1);
                    }),
                  ]
                : [2, s$3(i, e)];
            });
          });
        }),
        (i.prototype.emit = function (i, e) {
          return r$3(this, void 0, void 0, function () {
            var t, o;
            return o$6(this, function (n) {
              if (a$1.includes(i)) {
                for (t = 0, o = this.listeners[i] || []; t < o.length; t++)
                  (0, o[t])({ event: i, id: -1, payload: e });
                return [2, Promise.resolve()];
              }
              return [2, c$3(i, this.label, e)];
            });
          });
        }),
        (i.prototype._handleTauriEvent = function (t, n) {
          return (
            !!a$1.includes(t) &&
            (t in this.listeners
              ? this.listeners[t].push(n)
              : (this.listeners[t] = [n]),
            !0)
          );
        }),
        i
      );
    })(),
    h = (function (r) {
      function u(i, u) {
        void 0 === u && (u = {});
        var s = r.call(this, i) || this;
        return (
          n$4({
            __tauriModule: "Window",
            message: { cmd: "createWebview", options: e$2({ label: i }, u) },
          })
            .then(function () {
              return r$3(s, void 0, void 0, function () {
                return o$6(this, function (t) {
                  return [2, this.emit("tauri://created")];
                });
              });
            })
            .catch(function (i) {
              return r$3(s, void 0, void 0, function () {
                return o$6(this, function (t) {
                  return [2, this.emit("tauri://error", i)];
                });
              });
            }),
          s
        );
      }
      return (
        n$5(u, r),
        (u.getByLabel = function (t) {
          return c().some(function (n) {
            return n.label === t;
          })
            ? new f(t)
            : null;
        }),
        u
      );
    })(f),
    l = new ((function () {
      function i() {}
      return (
        (i.prototype.setResizable = function (i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setResizable", resizable: i },
                }),
              ];
            });
          });
        }),
        (i.prototype.setTitle = function (i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setTitle", title: i },
                }),
              ];
            });
          });
        }),
        (i.prototype.maximize = function () {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({ __tauriModule: "Window", message: { cmd: "maximize" } }),
              ];
            });
          });
        }),
        (i.prototype.unmaximize = function () {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "unmaximize" },
                }),
              ];
            });
          });
        }),
        (i.prototype.minimize = function () {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({ __tauriModule: "Window", message: { cmd: "minimize" } }),
              ];
            });
          });
        }),
        (i.prototype.unminimize = function () {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "unminimize" },
                }),
              ];
            });
          });
        }),
        (i.prototype.show = function () {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({ __tauriModule: "Window", message: { cmd: "show" } }),
              ];
            });
          });
        }),
        (i.prototype.hide = function () {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({ __tauriModule: "Window", message: { cmd: "hide" } }),
              ];
            });
          });
        }),
        (i.prototype.close = function () {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({ __tauriModule: "Window", message: { cmd: "close" } }),
              ];
            });
          });
        }),
        (i.prototype.setDecorations = function (i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setDecorations", decorations: i },
                }),
              ];
            });
          });
        }),
        (i.prototype.setAlwaysOnTop = function (i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setAlwaysOnTop", alwaysOnTop: i },
                }),
              ];
            });
          });
        }),
        (i.prototype.setWidth = function (i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setWidth", width: i },
                }),
              ];
            });
          });
        }),
        (i.prototype.setHeight = function (i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setHeight", height: i },
                }),
              ];
            });
          });
        }),
        (i.prototype.resize = function (i, e) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "resize", width: i, height: e },
                }),
              ];
            });
          });
        }),
        (i.prototype.setMinSize = function (i, e) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setMinSize", minWidth: i, minHeight: e },
                }),
              ];
            });
          });
        }),
        (i.prototype.setMaxSize = function (i, e) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setMaxSize", maxWidth: i, maxHeight: e },
                }),
              ];
            });
          });
        }),
        (i.prototype.setX = function (i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setX", x: i },
                }),
              ];
            });
          });
        }),
        (i.prototype.setY = function (i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setY", y: i },
                }),
              ];
            });
          });
        }),
        (i.prototype.setPosition = function (i, e) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setPosition", x: i, y: e },
                }),
              ];
            });
          });
        }),
        (i.prototype.setFullscreen = function (i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setFullscreen", fullscreen: i },
                }),
              ];
            });
          });
        }),
        (i.prototype.setIcon = function (i) {
          return r$3(this, void 0, void 0, function () {
            return o$6(this, function (t) {
              return [
                2,
                n$4({
                  __tauriModule: "Window",
                  message: { cmd: "setIcon", icon: i },
                }),
              ];
            });
          });
        }),
        i
      );
    })())();
  Object.freeze({
    __proto__: null,
    WebviewWindow: h,
    getCurrent: d,
    getAll: c,
    appWindow: l,
  });

  /* src/components/Window.svelte generated by Svelte v3.35.0 */
  const file$4 = "src/components/Window.svelte";

  function add_css() {
    var style = element("style");
    style.id = "svelte-b76pvm-style";
    style.textContent =
      ".flex-row.svelte-b76pvm.svelte-b76pvm{flex-direction:row}.grow.svelte-b76pvm.svelte-b76pvm{flex-grow:1}.window-controls.svelte-b76pvm input.svelte-b76pvm{width:50px}\n/*# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiV2luZG93LnN2ZWx0ZSIsInNvdXJjZXMiOlsiV2luZG93LnN2ZWx0ZSJdLCJzb3VyY2VzQ29udGVudCI6WyI8c2NyaXB0PlxuICBpbXBvcnQgeyBhcHBXaW5kb3cgfSBmcm9tIFwiQHRhdXJpLWFwcHMvYXBpL3dpbmRvd1wiO1xuICBpbXBvcnQgeyBvcGVuIGFzIG9wZW5EaWFsb2cgfSBmcm9tIFwiQHRhdXJpLWFwcHMvYXBpL2RpYWxvZ1wiO1xuICBpbXBvcnQgeyBvcGVuIH0gZnJvbSBcIkB0YXVyaS1hcHBzL2FwaS9zaGVsbFwiO1xuXG4gIGNvbnN0IHtcbiAgICBzZXRSZXNpemFibGUsXG4gICAgc2V0VGl0bGUsXG4gICAgbWF4aW1pemUsXG4gICAgdW5tYXhpbWl6ZSxcbiAgICBtaW5pbWl6ZSxcbiAgICB1bm1pbmltaXplLFxuICAgIHNob3csXG4gICAgaGlkZSxcbiAgICBzZXRUcmFuc3BhcmVudCxcbiAgICBzZXREZWNvcmF0aW9ucyxcbiAgICBzZXRBbHdheXNPblRvcCxcbiAgICBzZXRXaWR0aCxcbiAgICBzZXRIZWlnaHQsXG4gICAgLy8gcmVzaXplLFxuICAgIHNldE1pblNpemUsXG4gICAgc2V0TWF4U2l6ZSxcbiAgICBzZXRYLFxuICAgIHNldFksXG4gICAgLy8gc2V0UG9zaXRpb24sXG4gICAgc2V0RnVsbHNjcmVlbixcbiAgICBzZXRJY29uLFxuICB9ID0gYXBwV2luZG93O1xuXG4gIGxldCB1cmxWYWx1ZSA9IFwiaHR0cHM6Ly90YXVyaS5zdHVkaW9cIjtcbiAgbGV0IHJlc2l6YWJsZSA9IHRydWU7XG4gIGxldCBtYXhpbWl6ZWQgPSBmYWxzZTtcbiAgbGV0IHRyYW5zcGFyZW50ID0gZmFsc2U7XG4gIGxldCBkZWNvcmF0aW9ucyA9IHRydWU7XG4gIGxldCBhbHdheXNPblRvcCA9IGZhbHNlO1xuICBsZXQgZnVsbHNjcmVlbiA9IGZhbHNlO1xuICBsZXQgd2lkdGggPSA5MDA7XG4gIGxldCBoZWlnaHQgPSA3MDA7XG4gIGxldCBtaW5XaWR0aCA9IDYwMDtcbiAgbGV0IG1pbkhlaWdodCA9IDYwMDtcbiAgbGV0IG1heFdpZHRoID0gbnVsbDtcbiAgbGV0IG1heEhlaWdodCA9IG51bGw7XG4gIGxldCB4ID0gMTAwO1xuICBsZXQgeSA9IDEwMDtcblxuICBsZXQgd2luZG93VGl0bGUgPSBcIkF3ZXNvbWUgVGF1cmkgRXhhbXBsZSFcIjtcblxuICBmdW5jdGlvbiBvcGVuVXJsKCkge1xuICAgIG9wZW4odXJsVmFsdWUpO1xuICB9XG5cbiAgZnVuY3Rpb24gc2V0VGl0bGVfKCkge1xuICAgIHNldFRpdGxlKHdpbmRvd1RpdGxlKTtcbiAgfVxuXG4gIGZ1bmN0aW9uIGhpZGVfKCkge1xuICAgIGhpZGUoKTtcbiAgICBzZXRUaW1lb3V0KHNob3csIDIwMDApO1xuICB9XG5cbiAgZnVuY3Rpb24gbWluaW1pemVfKCkge1xuICAgIG1pbmltaXplKCk7XG4gICAgc2V0VGltZW91dCh1bm1pbmltaXplLCAyMDAwKTtcbiAgfVxuXG4gIGZ1bmN0aW9uIGdldEljb24oKSB7XG4gICAgb3BlbkRpYWxvZyh7XG4gICAgICBtdWx0aXBsZTogZmFsc2UsXG4gICAgfSkudGhlbihzZXRJY29uKTtcbiAgfVxuXG4gICQ6IHNldFJlc2l6YWJsZShyZXNpemFibGUpO1xuICAkOiBtYXhpbWl6ZWQgPyBtYXhpbWl6ZSgpIDogdW5tYXhpbWl6ZSgpO1xuICAvLyQ6IHNldFRyYW5zcGFyZW50KHRyYW5zcGFyZW50KVxuICAkOiBzZXREZWNvcmF0aW9ucyhkZWNvcmF0aW9ucyk7XG4gICQ6IHNldEFsd2F5c09uVG9wKGFsd2F5c09uVG9wKTtcbiAgJDogc2V0RnVsbHNjcmVlbihmdWxsc2NyZWVuKTtcblxuICAkOiBzZXRXaWR0aCh3aWR0aCk7XG4gICQ6IHNldEhlaWdodChoZWlnaHQpO1xuICAkOiBtaW5XaWR0aCAmJiBtaW5IZWlnaHQgJiYgc2V0TWluU2l6ZShtaW5XaWR0aCwgbWluSGVpZ2h0KTtcbiAgJDogbWF4V2lkdGggJiYgbWF4SGVpZ2h0ICYmIHNldE1heFNpemUobWF4V2lkdGgsIG1heEhlaWdodCk7XG4gICQ6IHNldFgoeCk7XG4gICQ6IHNldFkoeSk7XG48L3NjcmlwdD5cblxuPGRpdiBjbGFzcz1cImZsZXggY29sXCI+XG4gIDxkaXY+XG4gICAgPGxhYmVsPlxuICAgICAgPGlucHV0IHR5cGU9XCJjaGVja2JveFwiIGJpbmQ6Y2hlY2tlZD17cmVzaXphYmxlfSAvPlxuICAgICAgUmVzaXphYmxlXG4gICAgPC9sYWJlbD5cbiAgICA8bGFiZWw+XG4gICAgICA8aW5wdXQgdHlwZT1cImNoZWNrYm94XCIgYmluZDpjaGVja2VkPXttYXhpbWl6ZWR9IC8+XG4gICAgICBNYXhpbWl6ZVxuICAgIDwvbGFiZWw+XG4gICAgPGJ1dHRvbiB0aXRsZT1cIlVubWluaW1pemVzIGFmdGVyIDIgc2Vjb25kc1wiIG9uOmNsaWNrPXttaW5pbWl6ZV99PlxuICAgICAgTWluaW1pemVcbiAgICA8L2J1dHRvbj5cbiAgICA8YnV0dG9uIHRpdGxlPVwiVmlzaWJsZSBhZ2FpbiBhZnRlciAyIHNlY29uZHNcIiBvbjpjbGljaz17aGlkZV99PlxuICAgICAgSGlkZVxuICAgIDwvYnV0dG9uPlxuICAgIDxsYWJlbD5cbiAgICAgIDxpbnB1dCB0eXBlPVwiY2hlY2tib3hcIiBiaW5kOmNoZWNrZWQ9e3RyYW5zcGFyZW50fSAvPlxuICAgICAgVHJhbnNwYXJlbnRcbiAgICA8L2xhYmVsPlxuICAgIDxsYWJlbD5cbiAgICAgIDxpbnB1dCB0eXBlPVwiY2hlY2tib3hcIiBiaW5kOmNoZWNrZWQ9e2RlY29yYXRpb25zfSAvPlxuICAgICAgSGFzIGRlY29yYXRpb25zXG4gICAgPC9sYWJlbD5cbiAgICA8bGFiZWw+XG4gICAgICA8aW5wdXQgdHlwZT1cImNoZWNrYm94XCIgYmluZDpjaGVja2VkPXthbHdheXNPblRvcH0gLz5cbiAgICAgIEFsd2F5cyBvbiB0b3BcbiAgICA8L2xhYmVsPlxuICAgIDxsYWJlbD5cbiAgICAgIDxpbnB1dCB0eXBlPVwiY2hlY2tib3hcIiBiaW5kOmNoZWNrZWQ9e2Z1bGxzY3JlZW59IC8+XG4gICAgICBGdWxsc2NyZWVuXG4gICAgPC9sYWJlbD5cbiAgICA8YnV0dG9uIG9uOmNsaWNrPXtnZXRJY29ufT4gQ2hhbmdlIGljb24gPC9idXR0b24+XG4gIDwvZGl2PlxuICA8ZGl2PlxuICAgIDxkaXYgY2xhc3M9XCJ3aW5kb3ctY29udHJvbHMgZmxleCBmbGV4LXJvd1wiPlxuICAgICAgPGRpdiBjbGFzcz1cImZsZXggY29sIGdyb3dcIj5cbiAgICAgICAgPGRpdj5cbiAgICAgICAgICBYXG4gICAgICAgICAgPGlucHV0IHR5cGU9XCJudW1iZXJcIiBiaW5kOnZhbHVlPXt4fSBtaW49XCIwXCIgLz5cbiAgICAgICAgPC9kaXY+XG4gICAgICAgIDxkaXY+XG4gICAgICAgICAgWVxuICAgICAgICAgIDxpbnB1dCB0eXBlPVwibnVtYmVyXCIgYmluZDp2YWx1ZT17eX0gbWluPVwiMFwiIC8+XG4gICAgICAgIDwvZGl2PlxuICAgICAgPC9kaXY+XG5cbiAgICAgIDxkaXYgY2xhc3M9XCJmbGV4IGNvbCBncm93XCI+XG4gICAgICAgIDxkaXY+XG4gICAgICAgICAgV2lkdGhcbiAgICAgICAgICA8aW5wdXQgdHlwZT1cIm51bWJlclwiIGJpbmQ6dmFsdWU9e3dpZHRofSBtaW49XCI0MDBcIiAvPlxuICAgICAgICA8L2Rpdj5cbiAgICAgICAgPGRpdj5cbiAgICAgICAgICBIZWlnaHRcbiAgICAgICAgICA8aW5wdXQgdHlwZT1cIm51bWJlclwiIGJpbmQ6dmFsdWU9e2hlaWdodH0gbWluPVwiNDAwXCIgLz5cbiAgICAgICAgPC9kaXY+XG4gICAgICA8L2Rpdj5cblxuICAgICAgPGRpdiBjbGFzcz1cImZsZXggY29sIGdyb3dcIj5cbiAgICAgICAgPGRpdj5cbiAgICAgICAgICBNaW4gd2lkdGhcbiAgICAgICAgICA8aW5wdXQgdHlwZT1cIm51bWJlclwiIGJpbmQ6dmFsdWU9e21pbldpZHRofSAvPlxuICAgICAgICA8L2Rpdj5cbiAgICAgICAgPGRpdj5cbiAgICAgICAgICBNaW4gaGVpZ2h0XG4gICAgICAgICAgPGlucHV0IHR5cGU9XCJudW1iZXJcIiBiaW5kOnZhbHVlPXttaW5IZWlnaHR9IC8+XG4gICAgICAgIDwvZGl2PlxuICAgICAgPC9kaXY+XG5cbiAgICAgIDxkaXYgY2xhc3M9XCJmbGV4IGNvbCBncm93XCI+XG4gICAgICAgIDxkaXY+XG4gICAgICAgICAgTWF4IHdpZHRoXG4gICAgICAgICAgPGlucHV0IHR5cGU9XCJudW1iZXJcIiBiaW5kOnZhbHVlPXttYXhXaWR0aH0gbWluPVwiNDAwXCIgLz5cbiAgICAgICAgPC9kaXY+XG4gICAgICAgIDxkaXY+XG4gICAgICAgICAgTWF4IGhlaWdodFxuICAgICAgICAgIDxpbnB1dCB0eXBlPVwibnVtYmVyXCIgYmluZDp2YWx1ZT17bWF4SGVpZ2h0fSBtaW49XCI0MDBcIiAvPlxuICAgICAgICA8L2Rpdj5cbiAgICAgIDwvZGl2PlxuICAgIDwvZGl2PlxuICA8L2Rpdj5cbjwvZGl2PlxuPGZvcm0gc3R5bGU9XCJtYXJnaW4tdG9wOiAyNHB4XCIgb246c3VibWl0fHByZXZlbnREZWZhdWx0PXtzZXRUaXRsZV99PlxuICA8aW5wdXQgaWQ9XCJ0aXRsZVwiIGJpbmQ6dmFsdWU9e3dpbmRvd1RpdGxlfSAvPlxuICA8YnV0dG9uIGNsYXNzPVwiYnV0dG9uXCIgdHlwZT1cInN1Ym1pdFwiPlNldCB0aXRsZTwvYnV0dG9uPlxuPC9mb3JtPlxuPGZvcm0gc3R5bGU9XCJtYXJnaW4tdG9wOiAyNHB4XCIgb246c3VibWl0fHByZXZlbnREZWZhdWx0PXtvcGVuVXJsfT5cbiAgPGlucHV0IGlkPVwidXJsXCIgYmluZDp2YWx1ZT17dXJsVmFsdWV9IC8+XG4gIDxidXR0b24gY2xhc3M9XCJidXR0b25cIiBpZD1cIm9wZW4tdXJsXCI+IE9wZW4gVVJMIDwvYnV0dG9uPlxuPC9mb3JtPlxuXG48c3R5bGU+XG4gIC5mbGV4LXJvdyB7XG4gICAgZmxleC1kaXJlY3Rpb246IHJvdztcbiAgfVxuXG4gIC5ncm93IHtcbiAgICBmbGV4LWdyb3c6IDE7XG4gIH1cblxuICAud2luZG93LWNvbnRyb2xzIGlucHV0IHtcbiAgICB3aWR0aDogNTBweDtcbiAgfVxuPC9zdHlsZT5cbiJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFrTEUsU0FBUyw0QkFBQyxDQUFDLEFBQ1QsY0FBYyxDQUFFLEdBQUcsQUFDckIsQ0FBQyxBQUVELEtBQUssNEJBQUMsQ0FBQyxBQUNMLFNBQVMsQ0FBRSxDQUFDLEFBQ2QsQ0FBQyxBQUVELDhCQUFnQixDQUFDLEtBQUssY0FBQyxDQUFDLEFBQ3RCLEtBQUssQ0FBRSxJQUFJLEFBQ2IsQ0FBQyJ9 */";
    append_dev(document.head, style);
  }

  function create_fragment$4(ctx) {
    let div15;
    let div0;
    let label0;
    let input0;
    let t0;
    let t1;
    let label1;
    let input1;
    let t2;
    let t3;
    let button0;
    let t5;
    let button1;
    let t7;
    let label2;
    let input2;
    let t8;
    let t9;
    let label3;
    let input3;
    let t10;
    let t11;
    let label4;
    let input4;
    let t12;
    let t13;
    let label5;
    let input5;
    let t14;
    let t15;
    let button2;
    let t17;
    let div14;
    let div13;
    let div3;
    let div1;
    let t18;
    let input6;
    let t19;
    let div2;
    let t20;
    let input7;
    let t21;
    let div6;
    let div4;
    let t22;
    let input8;
    let t23;
    let div5;
    let t24;
    let input9;
    let t25;
    let div9;
    let div7;
    let t26;
    let input10;
    let t27;
    let div8;
    let t28;
    let input11;
    let t29;
    let div12;
    let div10;
    let t30;
    let input12;
    let t31;
    let div11;
    let t32;
    let input13;
    let t33;
    let form0;
    let input14;
    let t34;
    let button3;
    let t36;
    let form1;
    let input15;
    let t37;
    let button4;
    let mounted;
    let dispose;

    const block = {
      c: function create() {
        div15 = element("div");
        div0 = element("div");
        label0 = element("label");
        input0 = element("input");
        t0 = text("\n      Resizable");
        t1 = space();
        label1 = element("label");
        input1 = element("input");
        t2 = text("\n      Maximize");
        t3 = space();
        button0 = element("button");
        button0.textContent = "Minimize";
        t5 = space();
        button1 = element("button");
        button1.textContent = "Hide";
        t7 = space();
        label2 = element("label");
        input2 = element("input");
        t8 = text("\n      Transparent");
        t9 = space();
        label3 = element("label");
        input3 = element("input");
        t10 = text("\n      Has decorations");
        t11 = space();
        label4 = element("label");
        input4 = element("input");
        t12 = text("\n      Always on top");
        t13 = space();
        label5 = element("label");
        input5 = element("input");
        t14 = text("\n      Fullscreen");
        t15 = space();
        button2 = element("button");
        button2.textContent = "Change icon";
        t17 = space();
        div14 = element("div");
        div13 = element("div");
        div3 = element("div");
        div1 = element("div");
        t18 = text("X\n          ");
        input6 = element("input");
        t19 = space();
        div2 = element("div");
        t20 = text("Y\n          ");
        input7 = element("input");
        t21 = space();
        div6 = element("div");
        div4 = element("div");
        t22 = text("Width\n          ");
        input8 = element("input");
        t23 = space();
        div5 = element("div");
        t24 = text("Height\n          ");
        input9 = element("input");
        t25 = space();
        div9 = element("div");
        div7 = element("div");
        t26 = text("Min width\n          ");
        input10 = element("input");
        t27 = space();
        div8 = element("div");
        t28 = text("Min height\n          ");
        input11 = element("input");
        t29 = space();
        div12 = element("div");
        div10 = element("div");
        t30 = text("Max width\n          ");
        input12 = element("input");
        t31 = space();
        div11 = element("div");
        t32 = text("Max height\n          ");
        input13 = element("input");
        t33 = space();
        form0 = element("form");
        input14 = element("input");
        t34 = space();
        button3 = element("button");
        button3.textContent = "Set title";
        t36 = space();
        form1 = element("form");
        input15 = element("input");
        t37 = space();
        button4 = element("button");
        button4.textContent = "Open URL";
        attr_dev(input0, "type", "checkbox");
        add_location(input0, file$4, 89, 6, 1739);
        add_location(label0, file$4, 88, 4, 1725);
        attr_dev(input1, "type", "checkbox");
        add_location(input1, file$4, 93, 6, 1837);
        add_location(label1, file$4, 92, 4, 1823);
        attr_dev(button0, "title", "Unminimizes after 2 seconds");
        add_location(button0, file$4, 96, 4, 1920);
        attr_dev(button1, "title", "Visible again after 2 seconds");
        add_location(button1, file$4, 99, 4, 2019);
        attr_dev(input2, "type", "checkbox");
        add_location(input2, file$4, 103, 6, 2126);
        add_location(label2, file$4, 102, 4, 2112);
        attr_dev(input3, "type", "checkbox");
        add_location(input3, file$4, 107, 6, 2228);
        add_location(label3, file$4, 106, 4, 2214);
        attr_dev(input4, "type", "checkbox");
        add_location(input4, file$4, 111, 6, 2334);
        add_location(label4, file$4, 110, 4, 2320);
        attr_dev(input5, "type", "checkbox");
        add_location(input5, file$4, 115, 6, 2438);
        add_location(label5, file$4, 114, 4, 2424);
        add_location(button2, file$4, 118, 4, 2524);
        add_location(div0, file$4, 87, 2, 1715);
        attr_dev(input6, "type", "number");
        attr_dev(input6, "min", "0");
        attr_dev(input6, "class", "svelte-b76pvm");
        add_location(input6, file$4, 125, 10, 2709);
        add_location(div1, file$4, 123, 8, 2681);
        attr_dev(input7, "type", "number");
        attr_dev(input7, "min", "0");
        attr_dev(input7, "class", "svelte-b76pvm");
        add_location(input7, file$4, 129, 10, 2807);
        add_location(div2, file$4, 127, 8, 2779);
        attr_dev(div3, "class", "flex col grow svelte-b76pvm");
        add_location(div3, file$4, 122, 6, 2645);
        attr_dev(input8, "type", "number");
        attr_dev(input8, "min", "400");
        attr_dev(input8, "class", "svelte-b76pvm");
        add_location(input8, file$4, 136, 10, 2957);
        add_location(div4, file$4, 134, 8, 2925);
        attr_dev(input9, "type", "number");
        attr_dev(input9, "min", "400");
        attr_dev(input9, "class", "svelte-b76pvm");
        add_location(input9, file$4, 140, 10, 3066);
        add_location(div5, file$4, 138, 8, 3033);
        attr_dev(div6, "class", "flex col grow svelte-b76pvm");
        add_location(div6, file$4, 133, 6, 2889);
        attr_dev(input10, "type", "number");
        attr_dev(input10, "class", "svelte-b76pvm");
        add_location(input10, file$4, 147, 10, 3227);
        add_location(div7, file$4, 145, 8, 3191);
        attr_dev(input11, "type", "number");
        attr_dev(input11, "class", "svelte-b76pvm");
        add_location(input11, file$4, 151, 10, 3333);
        add_location(div8, file$4, 149, 8, 3296);
        attr_dev(div9, "class", "flex col grow svelte-b76pvm");
        add_location(div9, file$4, 144, 6, 3155);
        attr_dev(input12, "type", "number");
        attr_dev(input12, "min", "400");
        attr_dev(input12, "class", "svelte-b76pvm");
        add_location(input12, file$4, 158, 10, 3487);
        add_location(div10, file$4, 156, 8, 3451);
        attr_dev(input13, "type", "number");
        attr_dev(input13, "min", "400");
        attr_dev(input13, "class", "svelte-b76pvm");
        add_location(input13, file$4, 162, 10, 3603);
        add_location(div11, file$4, 160, 8, 3566);
        attr_dev(div12, "class", "flex col grow svelte-b76pvm");
        add_location(div12, file$4, 155, 6, 3415);
        attr_dev(div13, "class", "window-controls flex flex-row svelte-b76pvm");
        add_location(div13, file$4, 121, 4, 2595);
        add_location(div14, file$4, 120, 2, 2585);
        attr_dev(div15, "class", "flex col");
        add_location(div15, file$4, 86, 0, 1690);
        attr_dev(input14, "id", "title");
        add_location(input14, file$4, 169, 2, 3786);
        attr_dev(button3, "class", "button");
        attr_dev(button3, "type", "submit");
        add_location(button3, file$4, 170, 2, 3834);
        set_style(form0, "margin-top", "24px");
        add_location(form0, file$4, 168, 0, 3715);
        attr_dev(input15, "id", "url");
        add_location(input15, file$4, 173, 2, 3967);
        attr_dev(button4, "class", "button");
        attr_dev(button4, "id", "open-url");
        add_location(button4, file$4, 174, 2, 4010);
        set_style(form1, "margin-top", "24px");
        add_location(form1, file$4, 172, 0, 3898);
      },
      l: function claim(nodes) {
        throw new Error(
          "options.hydrate only works if the component was compiled with the `hydratable: true` option"
        );
      },
      m: function mount(target, anchor) {
        insert_dev(target, div15, anchor);
        append_dev(div15, div0);
        append_dev(div0, label0);
        append_dev(label0, input0);
        input0.checked = /*resizable*/ ctx[0];
        append_dev(label0, t0);
        append_dev(div0, t1);
        append_dev(div0, label1);
        append_dev(label1, input1);
        input1.checked = /*maximized*/ ctx[1];
        append_dev(label1, t2);
        append_dev(div0, t3);
        append_dev(div0, button0);
        append_dev(div0, t5);
        append_dev(div0, button1);
        append_dev(div0, t7);
        append_dev(div0, label2);
        append_dev(label2, input2);
        input2.checked = /*transparent*/ ctx[14];
        append_dev(label2, t8);
        append_dev(div0, t9);
        append_dev(div0, label3);
        append_dev(label3, input3);
        input3.checked = /*decorations*/ ctx[2];
        append_dev(label3, t10);
        append_dev(div0, t11);
        append_dev(div0, label4);
        append_dev(label4, input4);
        input4.checked = /*alwaysOnTop*/ ctx[3];
        append_dev(label4, t12);
        append_dev(div0, t13);
        append_dev(div0, label5);
        append_dev(label5, input5);
        input5.checked = /*fullscreen*/ ctx[4];
        append_dev(label5, t14);
        append_dev(div0, t15);
        append_dev(div0, button2);
        append_dev(div15, t17);
        append_dev(div15, div14);
        append_dev(div14, div13);
        append_dev(div13, div3);
        append_dev(div3, div1);
        append_dev(div1, t18);
        append_dev(div1, input6);
        set_input_value(input6, /*x*/ ctx[11]);
        append_dev(div3, t19);
        append_dev(div3, div2);
        append_dev(div2, t20);
        append_dev(div2, input7);
        set_input_value(input7, /*y*/ ctx[12]);
        append_dev(div13, t21);
        append_dev(div13, div6);
        append_dev(div6, div4);
        append_dev(div4, t22);
        append_dev(div4, input8);
        set_input_value(input8, /*width*/ ctx[5]);
        append_dev(div6, t23);
        append_dev(div6, div5);
        append_dev(div5, t24);
        append_dev(div5, input9);
        set_input_value(input9, /*height*/ ctx[6]);
        append_dev(div13, t25);
        append_dev(div13, div9);
        append_dev(div9, div7);
        append_dev(div7, t26);
        append_dev(div7, input10);
        set_input_value(input10, /*minWidth*/ ctx[7]);
        append_dev(div9, t27);
        append_dev(div9, div8);
        append_dev(div8, t28);
        append_dev(div8, input11);
        set_input_value(input11, /*minHeight*/ ctx[8]);
        append_dev(div13, t29);
        append_dev(div13, div12);
        append_dev(div12, div10);
        append_dev(div10, t30);
        append_dev(div10, input12);
        set_input_value(input12, /*maxWidth*/ ctx[9]);
        append_dev(div12, t31);
        append_dev(div12, div11);
        append_dev(div11, t32);
        append_dev(div11, input13);
        set_input_value(input13, /*maxHeight*/ ctx[10]);
        insert_dev(target, t33, anchor);
        insert_dev(target, form0, anchor);
        append_dev(form0, input14);
        set_input_value(input14, /*windowTitle*/ ctx[15]);
        append_dev(form0, t34);
        append_dev(form0, button3);
        insert_dev(target, t36, anchor);
        insert_dev(target, form1, anchor);
        append_dev(form1, input15);
        set_input_value(input15, /*urlValue*/ ctx[13]);
        append_dev(form1, t37);
        append_dev(form1, button4);

        if (!mounted) {
          dispose = [
            listen_dev(input0, "change", /*input0_change_handler*/ ctx[21]),
            listen_dev(input1, "change", /*input1_change_handler*/ ctx[22]),
            listen_dev(
              button0,
              "click",
              /*minimize_*/ ctx[19],
              false,
              false,
              false
            ),
            listen_dev(
              button1,
              "click",
              /*hide_*/ ctx[18],
              false,
              false,
              false
            ),
            listen_dev(input2, "change", /*input2_change_handler*/ ctx[23]),
            listen_dev(input3, "change", /*input3_change_handler*/ ctx[24]),
            listen_dev(input4, "change", /*input4_change_handler*/ ctx[25]),
            listen_dev(input5, "change", /*input5_change_handler*/ ctx[26]),
            listen_dev(
              button2,
              "click",
              /*getIcon*/ ctx[20],
              false,
              false,
              false
            ),
            listen_dev(input6, "input", /*input6_input_handler*/ ctx[27]),
            listen_dev(input7, "input", /*input7_input_handler*/ ctx[28]),
            listen_dev(input8, "input", /*input8_input_handler*/ ctx[29]),
            listen_dev(input9, "input", /*input9_input_handler*/ ctx[30]),
            listen_dev(input10, "input", /*input10_input_handler*/ ctx[31]),
            listen_dev(input11, "input", /*input11_input_handler*/ ctx[32]),
            listen_dev(input12, "input", /*input12_input_handler*/ ctx[33]),
            listen_dev(input13, "input", /*input13_input_handler*/ ctx[34]),
            listen_dev(input14, "input", /*input14_input_handler*/ ctx[35]),
            listen_dev(
              form0,
              "submit",
              prevent_default(/*setTitle_*/ ctx[17]),
              false,
              true,
              false
            ),
            listen_dev(input15, "input", /*input15_input_handler*/ ctx[36]),
            listen_dev(
              form1,
              "submit",
              prevent_default(/*openUrl*/ ctx[16]),
              false,
              true,
              false
            ),
          ];

          mounted = true;
        }
      },
      p: function update(ctx, dirty) {
        if (dirty[0] & /*resizable*/ 1) {
          input0.checked = /*resizable*/ ctx[0];
        }

        if (dirty[0] & /*maximized*/ 2) {
          input1.checked = /*maximized*/ ctx[1];
        }

        if (dirty[0] & /*transparent*/ 16384) {
          input2.checked = /*transparent*/ ctx[14];
        }

        if (dirty[0] & /*decorations*/ 4) {
          input3.checked = /*decorations*/ ctx[2];
        }

        if (dirty[0] & /*alwaysOnTop*/ 8) {
          input4.checked = /*alwaysOnTop*/ ctx[3];
        }

        if (dirty[0] & /*fullscreen*/ 16) {
          input5.checked = /*fullscreen*/ ctx[4];
        }

        if (
          dirty[0] & /*x*/ 2048 &&
          to_number(input6.value) !== /*x*/ ctx[11]
        ) {
          set_input_value(input6, /*x*/ ctx[11]);
        }

        if (
          dirty[0] & /*y*/ 4096 &&
          to_number(input7.value) !== /*y*/ ctx[12]
        ) {
          set_input_value(input7, /*y*/ ctx[12]);
        }

        if (
          dirty[0] & /*width*/ 32 &&
          to_number(input8.value) !== /*width*/ ctx[5]
        ) {
          set_input_value(input8, /*width*/ ctx[5]);
        }

        if (
          dirty[0] & /*height*/ 64 &&
          to_number(input9.value) !== /*height*/ ctx[6]
        ) {
          set_input_value(input9, /*height*/ ctx[6]);
        }

        if (
          dirty[0] & /*minWidth*/ 128 &&
          to_number(input10.value) !== /*minWidth*/ ctx[7]
        ) {
          set_input_value(input10, /*minWidth*/ ctx[7]);
        }

        if (
          dirty[0] & /*minHeight*/ 256 &&
          to_number(input11.value) !== /*minHeight*/ ctx[8]
        ) {
          set_input_value(input11, /*minHeight*/ ctx[8]);
        }

        if (
          dirty[0] & /*maxWidth*/ 512 &&
          to_number(input12.value) !== /*maxWidth*/ ctx[9]
        ) {
          set_input_value(input12, /*maxWidth*/ ctx[9]);
        }

        if (
          dirty[0] & /*maxHeight*/ 1024 &&
          to_number(input13.value) !== /*maxHeight*/ ctx[10]
        ) {
          set_input_value(input13, /*maxHeight*/ ctx[10]);
        }

        if (
          dirty[0] & /*windowTitle*/ 32768 &&
          input14.value !== /*windowTitle*/ ctx[15]
        ) {
          set_input_value(input14, /*windowTitle*/ ctx[15]);
        }

        if (
          dirty[0] & /*urlValue*/ 8192 &&
          input15.value !== /*urlValue*/ ctx[13]
        ) {
          set_input_value(input15, /*urlValue*/ ctx[13]);
        }
      },
      i: noop,
      o: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(div15);
        if (detaching) detach_dev(t33);
        if (detaching) detach_dev(form0);
        if (detaching) detach_dev(t36);
        if (detaching) detach_dev(form1);
        mounted = false;
        run_all(dispose);
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_fragment$4.name,
      type: "component",
      source: "",
      ctx,
    });

    return block;
  }

  function instance$4($$self, $$props, $$invalidate) {
    let { $$slots: slots = {}, $$scope } = $$props;
    validate_slots("Window", slots, []);

    const {
      setResizable,
      setTitle,
      maximize,
      unmaximize,
      minimize,
      unminimize,
      show,
      hide,
      setTransparent,
      setDecorations,
      setAlwaysOnTop,
      setWidth,
      setHeight, // resize,
      setMinSize,
      setMaxSize,
      setX,
      setY, // setPosition,
      setFullscreen,
      setIcon,
    } = l;

    let urlValue = "https://tauri.studio";
    let resizable = true;
    let maximized = false;
    let transparent = false;
    let decorations = true;
    let alwaysOnTop = false;
    let fullscreen = false;
    let width = 900;
    let height = 700;
    let minWidth = 600;
    let minHeight = 600;
    let maxWidth = null;
    let maxHeight = null;
    let x = 100;
    let y = 100;
    let windowTitle = "Awesome Tauri Example!";

    function openUrl() {
      d$3(urlValue);
    }

    function setTitle_() {
      setTitle(windowTitle);
    }

    function hide_() {
      hide();
      setTimeout(show, 2000);
    }

    function minimize_() {
      minimize();
      setTimeout(unminimize, 2000);
    }

    function getIcon() {
      i$2({ multiple: false }).then(setIcon);
    }

    const writable_props = [];

    Object.keys($$props).forEach((key) => {
      if (!~writable_props.indexOf(key) && key.slice(0, 2) !== "$$")
        console.warn(`<Window> was created with unknown prop '${key}'`);
    });

    function input0_change_handler() {
      resizable = this.checked;
      $$invalidate(0, resizable);
    }

    function input1_change_handler() {
      maximized = this.checked;
      $$invalidate(1, maximized);
    }

    function input2_change_handler() {
      transparent = this.checked;
      $$invalidate(14, transparent);
    }

    function input3_change_handler() {
      decorations = this.checked;
      $$invalidate(2, decorations);
    }

    function input4_change_handler() {
      alwaysOnTop = this.checked;
      $$invalidate(3, alwaysOnTop);
    }

    function input5_change_handler() {
      fullscreen = this.checked;
      $$invalidate(4, fullscreen);
    }

    function input6_input_handler() {
      x = to_number(this.value);
      $$invalidate(11, x);
    }

    function input7_input_handler() {
      y = to_number(this.value);
      $$invalidate(12, y);
    }

    function input8_input_handler() {
      width = to_number(this.value);
      $$invalidate(5, width);
    }

    function input9_input_handler() {
      height = to_number(this.value);
      $$invalidate(6, height);
    }

    function input10_input_handler() {
      minWidth = to_number(this.value);
      $$invalidate(7, minWidth);
    }

    function input11_input_handler() {
      minHeight = to_number(this.value);
      $$invalidate(8, minHeight);
    }

    function input12_input_handler() {
      maxWidth = to_number(this.value);
      $$invalidate(9, maxWidth);
    }

    function input13_input_handler() {
      maxHeight = to_number(this.value);
      $$invalidate(10, maxHeight);
    }

    function input14_input_handler() {
      windowTitle = this.value;
      $$invalidate(15, windowTitle);
    }

    function input15_input_handler() {
      urlValue = this.value;
      $$invalidate(13, urlValue);
    }

    $$self.$capture_state = () => ({
      appWindow: l,
      openDialog: i$2,
      open: d$3,
      setResizable,
      setTitle,
      maximize,
      unmaximize,
      minimize,
      unminimize,
      show,
      hide,
      setTransparent,
      setDecorations,
      setAlwaysOnTop,
      setWidth,
      setHeight,
      setMinSize,
      setMaxSize,
      setX,
      setY,
      setFullscreen,
      setIcon,
      urlValue,
      resizable,
      maximized,
      transparent,
      decorations,
      alwaysOnTop,
      fullscreen,
      width,
      height,
      minWidth,
      minHeight,
      maxWidth,
      maxHeight,
      x,
      y,
      windowTitle,
      openUrl,
      setTitle_,
      hide_,
      minimize_,
      getIcon,
    });

    $$self.$inject_state = ($$props) => {
      if ("urlValue" in $$props)
        $$invalidate(13, (urlValue = $$props.urlValue));
      if ("resizable" in $$props)
        $$invalidate(0, (resizable = $$props.resizable));
      if ("maximized" in $$props)
        $$invalidate(1, (maximized = $$props.maximized));
      if ("transparent" in $$props)
        $$invalidate(14, (transparent = $$props.transparent));
      if ("decorations" in $$props)
        $$invalidate(2, (decorations = $$props.decorations));
      if ("alwaysOnTop" in $$props)
        $$invalidate(3, (alwaysOnTop = $$props.alwaysOnTop));
      if ("fullscreen" in $$props)
        $$invalidate(4, (fullscreen = $$props.fullscreen));
      if ("width" in $$props) $$invalidate(5, (width = $$props.width));
      if ("height" in $$props) $$invalidate(6, (height = $$props.height));
      if ("minWidth" in $$props) $$invalidate(7, (minWidth = $$props.minWidth));
      if ("minHeight" in $$props)
        $$invalidate(8, (minHeight = $$props.minHeight));
      if ("maxWidth" in $$props) $$invalidate(9, (maxWidth = $$props.maxWidth));
      if ("maxHeight" in $$props)
        $$invalidate(10, (maxHeight = $$props.maxHeight));
      if ("x" in $$props) $$invalidate(11, (x = $$props.x));
      if ("y" in $$props) $$invalidate(12, (y = $$props.y));
      if ("windowTitle" in $$props)
        $$invalidate(15, (windowTitle = $$props.windowTitle));
    };

    if ($$props && "$$inject" in $$props) {
      $$self.$inject_state($$props.$$inject);
    }

    $$self.$$.update = () => {
      if ($$self.$$.dirty[0] & /*resizable*/ 1) {
        setResizable(resizable);
      }

      if ($$self.$$.dirty[0] & /*maximized*/ 2) {
        maximized ? maximize() : unmaximize();
      }

      if ($$self.$$.dirty[0] & /*decorations*/ 4) {
        //$: setTransparent(transparent)
        setDecorations(decorations);
      }

      if ($$self.$$.dirty[0] & /*alwaysOnTop*/ 8) {
        setAlwaysOnTop(alwaysOnTop);
      }

      if ($$self.$$.dirty[0] & /*fullscreen*/ 16) {
        setFullscreen(fullscreen);
      }

      if ($$self.$$.dirty[0] & /*width*/ 32) {
        setWidth(width);
      }

      if ($$self.$$.dirty[0] & /*height*/ 64) {
        setHeight(height);
      }

      if ($$self.$$.dirty[0] & /*minWidth, minHeight*/ 384) {
        minWidth && minHeight && setMinSize(minWidth, minHeight);
      }

      if ($$self.$$.dirty[0] & /*maxWidth, maxHeight*/ 1536) {
        maxWidth && maxHeight && setMaxSize(maxWidth, maxHeight);
      }

      if ($$self.$$.dirty[0] & /*x*/ 2048) {
        setX(x);
      }

      if ($$self.$$.dirty[0] & /*y*/ 4096) {
        setY(y);
      }
    };

    return [
      resizable,
      maximized,
      decorations,
      alwaysOnTop,
      fullscreen,
      width,
      height,
      minWidth,
      minHeight,
      maxWidth,
      maxHeight,
      x,
      y,
      urlValue,
      transparent,
      windowTitle,
      openUrl,
      setTitle_,
      hide_,
      minimize_,
      getIcon,
      input0_change_handler,
      input1_change_handler,
      input2_change_handler,
      input3_change_handler,
      input4_change_handler,
      input5_change_handler,
      input6_input_handler,
      input7_input_handler,
      input8_input_handler,
      input9_input_handler,
      input10_input_handler,
      input11_input_handler,
      input12_input_handler,
      input13_input_handler,
      input14_input_handler,
      input15_input_handler,
    ];
  }

  class Window extends SvelteComponentDev {
    constructor(options) {
      super(options);
      if (!document.getElementById("svelte-b76pvm-style")) add_css();
      init(this, options, instance$4, create_fragment$4, safe_not_equal, {}, [
        -1,
        -1,
      ]);

      dispatch_dev("SvelteRegisterComponent", {
        component: this,
        tagName: "Window",
        options,
        id: create_fragment$4.name,
      });
    }
  }

  const subscriber_queue = [];
  /**
   * Create a `Writable` store that allows both updating and reading by subscription.
   * @param {*=}value initial value
   * @param {StartStopNotifier=}start start and stop notifications for subscriptions
   */
  function writable(value, start = noop) {
    let stop;
    const subscribers = [];
    function set(new_value) {
      if (safe_not_equal(value, new_value)) {
        value = new_value;
        if (stop) {
          // store is ready
          const run_queue = !subscriber_queue.length;
          for (let i = 0; i < subscribers.length; i += 1) {
            const s = subscribers[i];
            s[1]();
            subscriber_queue.push(s, value);
          }
          if (run_queue) {
            for (let i = 0; i < subscriber_queue.length; i += 2) {
              subscriber_queue[i][0](subscriber_queue[i + 1]);
            }
            subscriber_queue.length = 0;
          }
        }
      }
    }
    function update(fn) {
      set(fn(value));
    }
    function subscribe(run, invalidate = noop) {
      const subscriber = [run, invalidate];
      subscribers.push(subscriber);
      if (subscribers.length === 1) {
        stop = start(set) || noop;
      }
      run(value);
      return () => {
        const index = subscribers.indexOf(subscriber);
        if (index !== -1) {
          subscribers.splice(index, 1);
        }
        if (subscribers.length === 0) {
          stop();
          stop = null;
        }
      };
    }
    return { set, update, subscribe };
  }

  function u$1(u, n) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [
          2,
          n$4({
            __tauriModule: "GlobalShortcut",
            message: { cmd: "register", shortcut: u, handler: a$5(n) },
          }),
        ];
      });
    });
  }
  function n(u, n) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [
          2,
          n$4({
            __tauriModule: "GlobalShortcut",
            message: { cmd: "registerAll", shortcuts: u, handler: a$5(n) },
          }),
        ];
      });
    });
  }
  function o(e) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [
          2,
          n$4({
            __tauriModule: "GlobalShortcut",
            message: { cmd: "isRegistered", shortcut: e },
          }),
        ];
      });
    });
  }
  function s(e) {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [
          2,
          n$4({
            __tauriModule: "GlobalShortcut",
            message: { cmd: "unregister", shortcut: e },
          }),
        ];
      });
    });
  }
  function a() {
    return r$3(this, void 0, void 0, function () {
      return o$6(this, function (t) {
        return [
          2,
          n$4({
            __tauriModule: "GlobalShortcut",
            message: { cmd: "unregisterAll" },
          }),
        ];
      });
    });
  }
  Object.freeze({
    __proto__: null,
    register: u$1,
    registerAll: n,
    isRegistered: o,
    unregister: s,
    unregisterAll: a,
  });

  /* src/components/Shortcuts.svelte generated by Svelte v3.35.0 */

  const file$3 = "src/components/Shortcuts.svelte";

  function get_each_context$1(ctx, list, i) {
    const child_ctx = ctx.slice();
    child_ctx[9] = list[i];
    return child_ctx;
  }

  // (56:4) {#each $shortcuts as savedShortcut}
  function create_each_block$1(ctx) {
    let div;
    let t0_value = /*savedShortcut*/ ctx[9] + "";
    let t0;
    let t1;
    let button;
    let mounted;
    let dispose;

    function click_handler() {
      return /*click_handler*/ ctx[8](/*savedShortcut*/ ctx[9]);
    }

    const block = {
      c: function create() {
        div = element("div");
        t0 = text(t0_value);
        t1 = space();
        button = element("button");
        button.textContent = "Unregister";
        attr_dev(button, "type", "button");
        add_location(button, file$3, 58, 8, 1488);
        add_location(div, file$3, 56, 6, 1450);
      },
      m: function mount(target, anchor) {
        insert_dev(target, div, anchor);
        append_dev(div, t0);
        append_dev(div, t1);
        append_dev(div, button);

        if (!mounted) {
          dispose = listen_dev(
            button,
            "click",
            click_handler,
            false,
            false,
            false
          );
          mounted = true;
        }
      },
      p: function update(new_ctx, dirty) {
        ctx = new_ctx;
        if (
          dirty & /*$shortcuts*/ 2 &&
          t0_value !== (t0_value = /*savedShortcut*/ ctx[9] + "")
        )
          set_data_dev(t0, t0_value);
      },
      d: function destroy(detaching) {
        if (detaching) detach_dev(div);
        mounted = false;
        dispose();
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_each_block$1.name,
      type: "each",
      source: "(56:4) {#each $shortcuts as savedShortcut}",
      ctx,
    });

    return block;
  }

  // (64:4) {#if $shortcuts.length}
  function create_if_block$1(ctx) {
    let button;
    let mounted;
    let dispose;

    const block = {
      c: function create() {
        button = element("button");
        button.textContent = "Unregister all";
        attr_dev(button, "type", "button");
        add_location(button, file$3, 64, 6, 1652);
      },
      m: function mount(target, anchor) {
        insert_dev(target, button, anchor);

        if (!mounted) {
          dispose = listen_dev(
            button,
            "click",
            /*unregisterAll*/ ctx[5],
            false,
            false,
            false
          );
          mounted = true;
        }
      },
      p: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(button);
        mounted = false;
        dispose();
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_if_block$1.name,
      type: "if",
      source: "(64:4) {#if $shortcuts.length}",
      ctx,
    });

    return block;
  }

  function create_fragment$3(ctx) {
    let div2;
    let div0;
    let input;
    let t0;
    let button;
    let t2;
    let div1;
    let t3;
    let mounted;
    let dispose;
    let each_value = /*$shortcuts*/ ctx[1];
    validate_each_argument(each_value);
    let each_blocks = [];

    for (let i = 0; i < each_value.length; i += 1) {
      each_blocks[i] = create_each_block$1(
        get_each_context$1(ctx, each_value, i)
      );
    }

    let if_block = /*$shortcuts*/ ctx[1].length && create_if_block$1(ctx);

    const block = {
      c: function create() {
        div2 = element("div");
        div0 = element("div");
        input = element("input");
        t0 = space();
        button = element("button");
        button.textContent = "Register";
        t2 = space();
        div1 = element("div");

        for (let i = 0; i < each_blocks.length; i += 1) {
          each_blocks[i].c();
        }

        t3 = space();
        if (if_block) if_block.c();
        attr_dev(
          input,
          "placeholder",
          "Type a shortcut with '+' as separator..."
        );
        add_location(input, file$3, 48, 4, 1220);
        attr_dev(button, "type", "button");
        add_location(button, file$3, 52, 4, 1327);
        add_location(div0, file$3, 47, 2, 1210);
        add_location(div1, file$3, 54, 2, 1398);
        add_location(div2, file$3, 46, 0, 1202);
      },
      l: function claim(nodes) {
        throw new Error(
          "options.hydrate only works if the component was compiled with the `hydratable: true` option"
        );
      },
      m: function mount(target, anchor) {
        insert_dev(target, div2, anchor);
        append_dev(div2, div0);
        append_dev(div0, input);
        set_input_value(input, /*shortcut*/ ctx[0]);
        append_dev(div0, t0);
        append_dev(div0, button);
        append_dev(div2, t2);
        append_dev(div2, div1);

        for (let i = 0; i < each_blocks.length; i += 1) {
          each_blocks[i].m(div1, null);
        }

        append_dev(div1, t3);
        if (if_block) if_block.m(div1, null);

        if (!mounted) {
          dispose = [
            listen_dev(input, "input", /*input_input_handler*/ ctx[7]),
            listen_dev(
              button,
              "click",
              /*register*/ ctx[3],
              false,
              false,
              false
            ),
          ];

          mounted = true;
        }
      },
      p: function update(ctx, [dirty]) {
        if (dirty & /*shortcut*/ 1 && input.value !== /*shortcut*/ ctx[0]) {
          set_input_value(input, /*shortcut*/ ctx[0]);
        }

        if (dirty & /*unregister, $shortcuts*/ 18) {
          each_value = /*$shortcuts*/ ctx[1];
          validate_each_argument(each_value);
          let i;

          for (i = 0; i < each_value.length; i += 1) {
            const child_ctx = get_each_context$1(ctx, each_value, i);

            if (each_blocks[i]) {
              each_blocks[i].p(child_ctx, dirty);
            } else {
              each_blocks[i] = create_each_block$1(child_ctx);
              each_blocks[i].c();
              each_blocks[i].m(div1, t3);
            }
          }

          for (; i < each_blocks.length; i += 1) {
            each_blocks[i].d(1);
          }

          each_blocks.length = each_value.length;
        }

        if (/*$shortcuts*/ ctx[1].length) {
          if (if_block) {
            if_block.p(ctx, dirty);
          } else {
            if_block = create_if_block$1(ctx);
            if_block.c();
            if_block.m(div1, null);
          }
        } else if (if_block) {
          if_block.d(1);
          if_block = null;
        }
      },
      i: noop,
      o: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(div2);
        destroy_each(each_blocks, detaching);
        if (if_block) if_block.d();
        mounted = false;
        run_all(dispose);
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_fragment$3.name,
      type: "component",
      source: "",
      ctx,
    });

    return block;
  }

  function instance$3($$self, $$props, $$invalidate) {
    let $shortcuts;
    let { $$slots: slots = {}, $$scope } = $$props;
    validate_slots("Shortcuts", slots, []);
    let { onMessage } = $$props;
    const shortcuts = writable([]);
    validate_store(shortcuts, "shortcuts");
    component_subscribe($$self, shortcuts, (value) =>
      $$invalidate(1, ($shortcuts = value))
    );
    let shortcut = "CmdOrControl+X";

    function register() {
      const shortcut_ = shortcut;

      u$1(shortcut_, () => {
        onMessage(`Shortcut ${shortcut_} triggered`);
      })
        .then(() => {
          shortcuts.update((shortcuts_) => [...shortcuts_, shortcut_]);
          onMessage(`Shortcut ${shortcut_} registered successfully`);
        })
        .catch(onMessage);
    }

    function unregister(shortcut) {
      const shortcut_ = shortcut;

      s(shortcut_)
        .then(() => {
          shortcuts.update((shortcuts_) =>
            shortcuts_.filter((s) => s !== shortcut_)
          );
          onMessage(`Shortcut ${shortcut_} unregistered`);
        })
        .catch(onMessage);
    }

    function unregisterAll() {
      a()
        .then(() => {
          shortcuts.update(() => []);
          onMessage(`Unregistered all shortcuts`);
        })
        .catch(onMessage);
    }

    const writable_props = ["onMessage"];

    Object.keys($$props).forEach((key) => {
      if (!~writable_props.indexOf(key) && key.slice(0, 2) !== "$$")
        console.warn(`<Shortcuts> was created with unknown prop '${key}'`);
    });

    function input_input_handler() {
      shortcut = this.value;
      $$invalidate(0, shortcut);
    }

    const click_handler = (savedShortcut) => unregister(savedShortcut);

    $$self.$$set = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(6, (onMessage = $$props.onMessage));
    };

    $$self.$capture_state = () => ({
      writable,
      registerShortcut: u$1,
      unregisterShortcut: s,
      unregisterAllShortcuts: a,
      onMessage,
      shortcuts,
      shortcut,
      register,
      unregister,
      unregisterAll,
      $shortcuts,
    });

    $$self.$inject_state = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(6, (onMessage = $$props.onMessage));
      if ("shortcut" in $$props) $$invalidate(0, (shortcut = $$props.shortcut));
    };

    if ($$props && "$$inject" in $$props) {
      $$self.$inject_state($$props.$$inject);
    }

    return [
      shortcut,
      $shortcuts,
      shortcuts,
      register,
      unregister,
      unregisterAll,
      onMessage,
      input_input_handler,
      click_handler,
    ];
  }

  class Shortcuts extends SvelteComponentDev {
    constructor(options) {
      super(options);
      init(this, options, instance$3, create_fragment$3, safe_not_equal, {
        onMessage: 6,
      });

      dispatch_dev("SvelteRegisterComponent", {
        component: this,
        tagName: "Shortcuts",
        options,
        id: create_fragment$3.name,
      });

      const { ctx } = this.$$;
      const props = options.props || {};

      if (/*onMessage*/ ctx[6] === undefined && !("onMessage" in props)) {
        console.warn(
          "<Shortcuts> was created without expected prop 'onMessage'"
        );
      }
    }

    get onMessage() {
      throw new Error(
        "<Shortcuts>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }

    set onMessage(value) {
      throw new Error(
        "<Shortcuts>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }
  }

  /* src/components/Shell.svelte generated by Svelte v3.35.0 */
  const file$2 = "src/components/Shell.svelte";

  // (47:4) {#if child}
  function create_if_block(ctx) {
    let input;
    let t0;
    let button;
    let mounted;
    let dispose;

    const block = {
      c: function create() {
        input = element("input");
        t0 = space();
        button = element("button");
        button.textContent = "Write";
        attr_dev(input, "placeholder", "write to stdin");
        add_location(input, file$2, 47, 6, 1223);
        attr_dev(button, "class", "button");
        add_location(button, file$2, 48, 6, 1285);
      },
      m: function mount(target, anchor) {
        insert_dev(target, input, anchor);
        set_input_value(input, /*stdin*/ ctx[1]);
        insert_dev(target, t0, anchor);
        insert_dev(target, button, anchor);

        if (!mounted) {
          dispose = [
            listen_dev(input, "input", /*input_input_handler_1*/ ctx[8]),
            listen_dev(
              button,
              "click",
              /*writeToStdin*/ ctx[5],
              false,
              false,
              false
            ),
          ];

          mounted = true;
        }
      },
      p: function update(ctx, dirty) {
        if (dirty & /*stdin*/ 2 && input.value !== /*stdin*/ ctx[1]) {
          set_input_value(input, /*stdin*/ ctx[1]);
        }
      },
      d: function destroy(detaching) {
        if (detaching) detach_dev(input);
        if (detaching) detach_dev(t0);
        if (detaching) detach_dev(button);
        mounted = false;
        run_all(dispose);
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_if_block.name,
      type: "if",
      source: "(47:4) {#if child}",
      ctx,
    });

    return block;
  }

  function create_fragment$2(ctx) {
    let div1;
    let div0;
    let input;
    let t0;
    let button0;
    let t2;
    let button1;
    let t4;
    let mounted;
    let dispose;
    let if_block = /*child*/ ctx[2] && create_if_block(ctx);

    const block = {
      c: function create() {
        div1 = element("div");
        div0 = element("div");
        input = element("input");
        t0 = space();
        button0 = element("button");
        button0.textContent = "Run";
        t2 = space();
        button1 = element("button");
        button1.textContent = "Kill";
        t4 = space();
        if (if_block) if_block.c();
        add_location(input, file$2, 43, 4, 1059);
        attr_dev(button0, "class", "button");
        add_location(button0, file$2, 44, 4, 1091);
        attr_dev(button1, "class", "button");
        add_location(button1, file$2, 45, 4, 1148);
        add_location(div0, file$2, 42, 2, 1049);
        add_location(div1, file$2, 41, 0, 1041);
      },
      l: function claim(nodes) {
        throw new Error(
          "options.hydrate only works if the component was compiled with the `hydratable: true` option"
        );
      },
      m: function mount(target, anchor) {
        insert_dev(target, div1, anchor);
        append_dev(div1, div0);
        append_dev(div0, input);
        set_input_value(input, /*script*/ ctx[0]);
        append_dev(div0, t0);
        append_dev(div0, button0);
        append_dev(div0, t2);
        append_dev(div0, button1);
        append_dev(div0, t4);
        if (if_block) if_block.m(div0, null);

        if (!mounted) {
          dispose = [
            listen_dev(input, "input", /*input_input_handler*/ ctx[7]),
            listen_dev(button0, "click", /*spawn*/ ctx[3], false, false, false),
            listen_dev(button1, "click", /*kill*/ ctx[4], false, false, false),
          ];

          mounted = true;
        }
      },
      p: function update(ctx, [dirty]) {
        if (dirty & /*script*/ 1 && input.value !== /*script*/ ctx[0]) {
          set_input_value(input, /*script*/ ctx[0]);
        }

        if (/*child*/ ctx[2]) {
          if (if_block) {
            if_block.p(ctx, dirty);
          } else {
            if_block = create_if_block(ctx);
            if_block.c();
            if_block.m(div0, null);
          }
        } else if (if_block) {
          if_block.d(1);
          if_block = null;
        }
      },
      i: noop,
      o: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(div1);
        if (if_block) if_block.d();
        mounted = false;
        run_all(dispose);
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_fragment$2.name,
      type: "component",
      source: "",
      ctx,
    });

    return block;
  }

  function instance$2($$self, $$props, $$invalidate) {
    let { $$slots: slots = {}, $$scope } = $$props;
    validate_slots("Shell", slots, []);
    const windows = navigator.userAgent.includes("Windows");
    let cmd = windows ? "cmd" : "sh";
    let args = windows ? ["/C"] : ["-c"];
    let { onMessage } = $$props;
    let script = 'echo "hello world"';
    let stdin = "";
    let child;

    function spawn() {
      $$invalidate(2, (child = null));
      const command = new a$4(cmd, [...args, script]);

      command.on("close", (data) => {
        onMessage(
          `command finished with code ${data.code} and signal ${data.signal}`
        );
        $$invalidate(2, (child = null));
      });

      command.on("error", (error) => onMessage(`command error: "${error}"`));
      command.stdout.on("data", (line) =>
        onMessage(`command stdout: "${line}"`)
      );
      command.stderr.on("data", (line) =>
        onMessage(`command stderr: "${line}"`)
      );

      command
        .spawn()
        .then((c) => {
          $$invalidate(2, (child = c));
        })
        .catch(onMessage);
    }

    function kill() {
      child
        .kill()
        .then(() => onMessage("killed child process"))
        .error(onMessage);
    }

    function writeToStdin() {
      child.write(stdin).catch(onMessage);
    }

    const writable_props = ["onMessage"];

    Object.keys($$props).forEach((key) => {
      if (!~writable_props.indexOf(key) && key.slice(0, 2) !== "$$")
        console.warn(`<Shell> was created with unknown prop '${key}'`);
    });

    function input_input_handler() {
      script = this.value;
      $$invalidate(0, script);
    }

    function input_input_handler_1() {
      stdin = this.value;
      $$invalidate(1, stdin);
    }

    $$self.$$set = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(6, (onMessage = $$props.onMessage));
    };

    $$self.$capture_state = () => ({
      Command: a$4,
      windows,
      cmd,
      args,
      onMessage,
      script,
      stdin,
      child,
      spawn,
      kill,
      writeToStdin,
    });

    $$self.$inject_state = ($$props) => {
      if ("cmd" in $$props) cmd = $$props.cmd;
      if ("args" in $$props) args = $$props.args;
      if ("onMessage" in $$props)
        $$invalidate(6, (onMessage = $$props.onMessage));
      if ("script" in $$props) $$invalidate(0, (script = $$props.script));
      if ("stdin" in $$props) $$invalidate(1, (stdin = $$props.stdin));
      if ("child" in $$props) $$invalidate(2, (child = $$props.child));
    };

    if ($$props && "$$inject" in $$props) {
      $$self.$inject_state($$props.$$inject);
    }

    return [
      script,
      stdin,
      child,
      spawn,
      kill,
      writeToStdin,
      onMessage,
      input_input_handler,
      input_input_handler_1,
    ];
  }

  class Shell extends SvelteComponentDev {
    constructor(options) {
      super(options);
      init(this, options, instance$2, create_fragment$2, safe_not_equal, {
        onMessage: 6,
      });

      dispatch_dev("SvelteRegisterComponent", {
        component: this,
        tagName: "Shell",
        options,
        id: create_fragment$2.name,
      });

      const { ctx } = this.$$;
      const props = options.props || {};

      if (/*onMessage*/ ctx[6] === undefined && !("onMessage" in props)) {
        console.warn("<Shell> was created without expected prop 'onMessage'");
      }
    }

    get onMessage() {
      throw new Error(
        "<Shell>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }

    set onMessage(value) {
      throw new Error(
        "<Shell>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }
  }

  function i() {
    return r$3(this, void 0, void 0, function () {
      function t() {
        r && r(), (r = void 0);
      }
      var r;
      return o$6(this, function (n) {
        return [
          2,
          new Promise(function (n, i) {
            o$3("tauri://update-status", function (o) {
              var a;
              (a = null == o ? void 0 : o.payload).error
                ? (t(), i(a.error))
                : "DONE" === a.status && (t(), n());
            })
              .then(function (t) {
                r = t;
              })
              .catch(function (n) {
                throw (t(), n);
              }),
              console.log("EMIT EVENT"),
              c$3("tauri://update-install").catch(function (n) {
                throw (t(), n);
              });
          }),
        ];
      });
    });
  }
  function u() {
    return r$3(this, void 0, void 0, function () {
      function t() {
        i && i(), (i = void 0);
      }
      var i;
      return o$6(this, function (n) {
        return [
          2,
          new Promise(function (n, u) {
            s$3("tauri://update-available", function (o) {
              var a;
              (a = null == o ? void 0 : o.payload),
                t(),
                n({ manifest: a, shouldUpdate: !0 });
            }).catch(function (n) {
              throw (t(), n);
            }),
              o$3("tauri://update-status", function (o) {
                var a;
                (a = null == o ? void 0 : o.payload).error
                  ? (t(), u(a.error))
                  : "UPTODATE" === a.status && (t(), n({ shouldUpdate: !1 }));
              })
                .then(function (t) {
                  i = t;
                })
                .catch(function (n) {
                  throw (t(), n);
                }),
              c$3("tauri://update").catch(function (n) {
                throw (t(), n);
              });
          }),
        ];
      });
    });
  }
  Object.freeze({ __proto__: null, installUpdate: i, checkUpdate: u });

  /* src/components/Updater.svelte generated by Svelte v3.35.0 */
  const file$1 = "src/components/Updater.svelte";

  function create_fragment$1(ctx) {
    let div;
    let button0;
    let t1;
    let button1;
    let mounted;
    let dispose;

    const block = {
      c: function create() {
        div = element("div");
        button0 = element("button");
        button0.textContent = "Check update";
        t1 = space();
        button1 = element("button");
        button1.textContent = "Install update";
        attr_dev(button0, "class", "button");
        attr_dev(button0, "id", "check_update");
        add_location(button0, file$1, 56, 2, 1362);
        attr_dev(button1, "class", "button hidden");
        attr_dev(button1, "id", "start_update");
        add_location(button1, file$1, 57, 2, 1444);
        add_location(div, file$1, 55, 0, 1354);
      },
      l: function claim(nodes) {
        throw new Error(
          "options.hydrate only works if the component was compiled with the `hydratable: true` option"
        );
      },
      m: function mount(target, anchor) {
        insert_dev(target, div, anchor);
        append_dev(div, button0);
        append_dev(div, t1);
        append_dev(div, button1);

        if (!mounted) {
          dispose = [
            listen_dev(button0, "click", /*check*/ ctx[0], false, false, false),
            listen_dev(
              button1,
              "click",
              /*install*/ ctx[1],
              false,
              false,
              false
            ),
          ];

          mounted = true;
        }
      },
      p: noop,
      i: noop,
      o: noop,
      d: function destroy(detaching) {
        if (detaching) detach_dev(div);
        mounted = false;
        run_all(dispose);
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_fragment$1.name,
      type: "component",
      source: "",
      ctx,
    });

    return block;
  }

  function instance$1($$self, $$props, $$invalidate) {
    let { $$slots: slots = {}, $$scope } = $$props;
    validate_slots("Updater", slots, []);
    let { onMessage } = $$props;
    let unlisten;

    onMount(async () => {
      unlisten = await o$3("tauri://update-status", onMessage);
    });

    onDestroy(() => {
      if (unlisten) {
        unlisten();
      }
    });

    async function check() {
      try {
        document.getElementById("check_update").classList.add("hidden");
        const { shouldUpdate, manifest } = await u();
        onMessage(`Should update: ${shouldUpdate}`);
        onMessage(manifest);

        if (shouldUpdate) {
          document.getElementById("start_update").classList.remove("hidden");
        }
      } catch (e) {
        onMessage(e);
      }
    }

    async function install() {
      try {
        document.getElementById("start_update").classList.add("hidden");
        await i();
        onMessage("Installation complete, restart required.");
        await a$3();
      } catch (e) {
        onMessage(e);
      }
    }

    const writable_props = ["onMessage"];

    Object.keys($$props).forEach((key) => {
      if (!~writable_props.indexOf(key) && key.slice(0, 2) !== "$$")
        console.warn(`<Updater> was created with unknown prop '${key}'`);
    });

    $$self.$$set = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(2, (onMessage = $$props.onMessage));
    };

    $$self.$capture_state = () => ({
      onMount,
      onDestroy,
      checkUpdate: u,
      installUpdate: i,
      listen: o$3,
      relaunch: a$3,
      onMessage,
      unlisten,
      check,
      install,
    });

    $$self.$inject_state = ($$props) => {
      if ("onMessage" in $$props)
        $$invalidate(2, (onMessage = $$props.onMessage));
      if ("unlisten" in $$props) unlisten = $$props.unlisten;
    };

    if ($$props && "$$inject" in $$props) {
      $$self.$inject_state($$props.$$inject);
    }

    return [check, install, onMessage];
  }

  class Updater extends SvelteComponentDev {
    constructor(options) {
      super(options);
      init(this, options, instance$1, create_fragment$1, safe_not_equal, {
        onMessage: 2,
      });

      dispatch_dev("SvelteRegisterComponent", {
        component: this,
        tagName: "Updater",
        options,
        id: create_fragment$1.name,
      });

      const { ctx } = this.$$;
      const props = options.props || {};

      if (/*onMessage*/ ctx[2] === undefined && !("onMessage" in props)) {
        console.warn("<Updater> was created without expected prop 'onMessage'");
      }
    }

    get onMessage() {
      throw new Error(
        "<Updater>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }

    set onMessage(value) {
      throw new Error(
        "<Updater>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'"
      );
    }
  }

  /* src/App.svelte generated by Svelte v3.35.0 */
  const file = "src/App.svelte";

  function get_each_context(ctx, list, i) {
    const child_ctx = ctx.slice();
    child_ctx[8] = list[i];
    return child_ctx;
  }

  // (99:6) {#each views as view}
  function create_each_block(ctx) {
    let p;
    let t0_value = /*view*/ ctx[8].label + "";
    let t0;
    let t1;
    let p_class_value;
    let mounted;
    let dispose;

    function click_handler() {
      return /*click_handler*/ ctx[6](/*view*/ ctx[8]);
    }

    const block = {
      c: function create() {
        p = element("p");
        t0 = text(t0_value);
        t1 = space();

        attr_dev(
          p,
          "class",
          (p_class_value =
            "nv noselect " +
            /*selected*/ (ctx[0] === /*view*/ ctx[8] ? "nv_selected" : ""))
        );

        add_location(p, file, 99, 6, 2408);
      },
      m: function mount(target, anchor) {
        insert_dev(target, p, anchor);
        append_dev(p, t0);
        append_dev(p, t1);

        if (!mounted) {
          dispose = listen_dev(p, "click", click_handler, false, false, false);
          mounted = true;
        }
      },
      p: function update(new_ctx, dirty) {
        ctx = new_ctx;

        if (
          dirty & /*selected*/ 1 &&
          p_class_value !==
            (p_class_value =
              "nv noselect " +
              /*selected*/ (ctx[0] === /*view*/ ctx[8] ? "nv_selected" : ""))
        ) {
          attr_dev(p, "class", p_class_value);
        }
      },
      d: function destroy(detaching) {
        if (detaching) detach_dev(p);
        mounted = false;
        dispose();
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_each_block.name,
      type: "each",
      source: "(99:6) {#each views as view}",
      ctx,
    });

    return block;
  }

  function create_fragment(ctx) {
    let main;
    let div1;
    let img;
    let img_src_value;
    let t0;
    let div0;
    let a0;
    let t2;
    let a1;
    let t4;
    let a2;
    let t6;
    let div4;
    let div2;
    let t7;
    let div3;
    let switch_instance;
    let t8;
    let div5;
    let p;
    let strong;
    let t10;
    let a3;
    let t12;
    let t13;
    let current;
    let mounted;
    let dispose;
    let each_value = /*views*/ ctx[2];
    validate_each_argument(each_value);
    let each_blocks = [];

    for (let i = 0; i < each_value.length; i += 1) {
      each_blocks[i] = create_each_block(get_each_context(ctx, each_value, i));
    }

    var switch_value = /*selected*/ ctx[0].component;

    function switch_props(ctx) {
      return {
        props: { onMessage: /*onMessage*/ ctx[4] },
        $$inline: true,
      };
    }

    if (switch_value) {
      switch_instance = new switch_value(switch_props(ctx));
    }

    const block = {
      c: function create() {
        main = element("main");
        div1 = element("div");
        img = element("img");
        t0 = space();
        div0 = element("div");
        a0 = element("a");
        a0.textContent = "Documentation";
        t2 = space();
        a1 = element("a");
        a1.textContent = "Github";
        t4 = space();
        a2 = element("a");
        a2.textContent = "Source";
        t6 = space();
        div4 = element("div");
        div2 = element("div");

        for (let i = 0; i < each_blocks.length; i += 1) {
          each_blocks[i].c();
        }

        t7 = space();
        div3 = element("div");
        if (switch_instance) create_component(switch_instance.$$.fragment);
        t8 = space();
        div5 = element("div");
        p = element("p");
        strong = element("strong");
        strong.textContent = "Tauri Console";
        t10 = space();
        a3 = element("a");
        a3.textContent = "clear";
        t12 = space();
        t13 = text(/*responses*/ ctx[1]);
        if (img.src !== (img_src_value = "tauri.png"))
          attr_dev(img, "src", img_src_value);
        attr_dev(img, "height", "60");
        attr_dev(img, "alt", "logo");
        add_location(img, file, 83, 4, 1812);
        attr_dev(a0, "class", "dark-link");
        attr_dev(a0, "target", "_blank");
        attr_dev(
          a0,
          "href",
          "https://tauri.studio/en/docs/getting-started/intro"
        );
        add_location(a0, file, 85, 6, 1898);
        attr_dev(a1, "class", "dark-link");
        attr_dev(a1, "target", "_blank");
        attr_dev(a1, "href", "https://github.com/tauri-apps/tauri");
        add_location(a1, file, 88, 6, 2033);
        attr_dev(a2, "class", "dark-link");
        attr_dev(a2, "target", "_blank");
        attr_dev(
          a2,
          "href",
          "https://github.com/tauri-apps/tauri/tree/dev/tauri/examples/api"
        );
        add_location(a2, file, 91, 6, 2146);
        add_location(div0, file, 84, 4, 1886);
        attr_dev(div1, "class", "flex row noselect just-around");
        attr_dev(div1, "style", "margin=1em;");
        add_location(div1, file, 82, 2, 1744);
        set_style(div2, "width", "15em");
        set_style(div2, "margin-left", "0.5em");
        add_location(div2, file, 97, 4, 2330);
        attr_dev(div3, "class", "content");
        add_location(div3, file, 105, 4, 2572);
        attr_dev(div4, "class", "flex row");
        add_location(div4, file, 96, 2, 2303);
        add_location(strong, file, 111, 6, 2774);
        attr_dev(a3, "class", "nv");
        add_location(a3, file, 112, 6, 2811);
        attr_dev(p, "class", "flex row just-around");
        add_location(p, file, 110, 4, 2735);
        attr_dev(div5, "id", "response");
        set_style(div5, "white-space", "pre-line");
        add_location(div5, file, 109, 2, 2681);
        add_location(main, file, 81, 0, 1735);
      },
      l: function claim(nodes) {
        throw new Error(
          "options.hydrate only works if the component was compiled with the `hydratable: true` option"
        );
      },
      m: function mount(target, anchor) {
        insert_dev(target, main, anchor);
        append_dev(main, div1);
        append_dev(div1, img);
        append_dev(div1, t0);
        append_dev(div1, div0);
        append_dev(div0, a0);
        append_dev(div0, t2);
        append_dev(div0, a1);
        append_dev(div0, t4);
        append_dev(div0, a2);
        append_dev(main, t6);
        append_dev(main, div4);
        append_dev(div4, div2);

        for (let i = 0; i < each_blocks.length; i += 1) {
          each_blocks[i].m(div2, null);
        }

        append_dev(div4, t7);
        append_dev(div4, div3);

        if (switch_instance) {
          mount_component(switch_instance, div3, null);
        }

        append_dev(main, t8);
        append_dev(main, div5);
        append_dev(div5, p);
        append_dev(p, strong);
        append_dev(p, t10);
        append_dev(p, a3);
        append_dev(div5, t12);
        append_dev(div5, t13);
        current = true;

        if (!mounted) {
          dispose = [
            listen_dev(
              img,
              "click",
              /*onLogoClick*/ ctx[5],
              false,
              false,
              false
            ),
            listen_dev(
              a3,
              "click",
              /*click_handler_1*/ ctx[7],
              false,
              false,
              false
            ),
          ];

          mounted = true;
        }
      },
      p: function update(ctx, [dirty]) {
        if (dirty & /*selected, views, select*/ 13) {
          each_value = /*views*/ ctx[2];
          validate_each_argument(each_value);
          let i;

          for (i = 0; i < each_value.length; i += 1) {
            const child_ctx = get_each_context(ctx, each_value, i);

            if (each_blocks[i]) {
              each_blocks[i].p(child_ctx, dirty);
            } else {
              each_blocks[i] = create_each_block(child_ctx);
              each_blocks[i].c();
              each_blocks[i].m(div2, null);
            }
          }

          for (; i < each_blocks.length; i += 1) {
            each_blocks[i].d(1);
          }

          each_blocks.length = each_value.length;
        }

        if (switch_value !== (switch_value = /*selected*/ ctx[0].component)) {
          if (switch_instance) {
            group_outros();
            const old_component = switch_instance;

            transition_out(old_component.$$.fragment, 1, 0, () => {
              destroy_component(old_component, 1);
            });

            check_outros();
          }

          if (switch_value) {
            switch_instance = new switch_value(switch_props(ctx));
            create_component(switch_instance.$$.fragment);
            transition_in(switch_instance.$$.fragment, 1);
            mount_component(switch_instance, div3, null);
          } else {
            switch_instance = null;
          }
        }

        if (!current || dirty & /*responses*/ 2)
          set_data_dev(t13, /*responses*/ ctx[1]);
      },
      i: function intro(local) {
        if (current) return;
        if (switch_instance) transition_in(switch_instance.$$.fragment, local);
        current = true;
      },
      o: function outro(local) {
        if (switch_instance) transition_out(switch_instance.$$.fragment, local);
        current = false;
      },
      d: function destroy(detaching) {
        if (detaching) detach_dev(main);
        destroy_each(each_blocks, detaching);
        if (switch_instance) destroy_component(switch_instance);
        mounted = false;
        run_all(dispose);
      },
    };

    dispatch_dev("SvelteRegisterBlock", {
      block,
      id: create_fragment.name,
      type: "component",
      source: "",
      ctx,
    });

    return block;
  }

  function instance($$self, $$props, $$invalidate) {
    let { $$slots: slots = {}, $$scope } = $$props;
    validate_slots("App", slots, []);

    const views = [
      { label: "Welcome", component: Welcome },
      {
        label: "Messages",
        component: Communication,
      },
      { label: "CLI", component: Cli },
      { label: "Dialog", component: Dialog },
      {
        label: "File system",
        component: FileSystem,
      },
      { label: "HTTP", component: Http },
      {
        label: "Notifications",
        component: Notifications,
      },
      { label: "Window", component: Window },
      { label: "Shortcuts", component: Shortcuts },
      { label: "Shell", component: Shell },
      { label: "Updater", component: Updater },
    ];

    let selected = views[0];
    let responses = [""];

    function select(view) {
      $$invalidate(0, (selected = view));
    }

    function onMessage(value) {
      $$invalidate(
        1,
        (responses += typeof value === "string" ? value : JSON.stringify(value))
      );

      $$invalidate(1, (responses += "\n"));
    }

    function onLogoClick() {
      d$3("https://tauri.studio/");
    }

    const writable_props = [];

    Object.keys($$props).forEach((key) => {
      if (!~writable_props.indexOf(key) && key.slice(0, 2) !== "$$")
        console.warn(`<App> was created with unknown prop '${key}'`);
    });

    const click_handler = (view) => select(view);

    const click_handler_1 = () => {
      $$invalidate(1, (responses = [""]));
    };

    $$self.$capture_state = () => ({
      onMount,
      open: d$3,
      Welcome,
      Cli,
      Communication,
      Dialog,
      FileSystem,
      Http,
      Notifications,
      Window,
      Shortcuts,
      Shell,
      Updater,
      views,
      selected,
      responses,
      select,
      onMessage,
      onLogoClick,
    });

    $$self.$inject_state = ($$props) => {
      if ("selected" in $$props) $$invalidate(0, (selected = $$props.selected));
      if ("responses" in $$props)
        $$invalidate(1, (responses = $$props.responses));
    };

    if ($$props && "$$inject" in $$props) {
      $$self.$inject_state($$props.$$inject);
    }

    return [
      selected,
      responses,
      views,
      select,
      onMessage,
      onLogoClick,
      click_handler,
      click_handler_1,
    ];
  }

  class App extends SvelteComponentDev {
    constructor(options) {
      super(options);
      init(this, options, instance, create_fragment, safe_not_equal, {});

      dispatch_dev("SvelteRegisterComponent", {
        component: this,
        tagName: "App",
        options,
        id: create_fragment.name,
      });
    }
  }

  const app = new App({
    target: document.body,
  });

  return app;
})();
//# sourceMappingURL=bundle.js.map
