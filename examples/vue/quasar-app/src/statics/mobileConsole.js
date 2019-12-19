/*!
 * hnl.mobileConsole - javascript mobile console - v1.3.5 - 19/4/2018
 * Adds html console to webpage. Especially useful for debugging JS on mobile devices.
 * Supports 'log', 'trace', 'info', 'warn', 'error', 'group', 'groupEnd', 'table', 'assert', 'clear'
 * Inspired by code by jakub fiala (https://gist.github.com/jakubfiala/8fe3461ab6508f46003d)
 * Licensed under the MIT license
 *
 * Original author: @hnldesign
 * Further changes, comments: @hnldesign
 * Copyright (c) 2014-2016 HN Leussink
 * Dual licensed under the MIT and GPL licenses.
 *
 * Info: http://www.hnldesign.nl/work/code/javascript-mobile-console/
 * Demo: http://code.hnldesign.nl/demo/hnl.MobileConsole.html
 */

//Polyfills

//Date.now polyfill
if (!Date.now) {
  Date.now = function now() {
    return new Date().getTime();
  };
}
//Array.isArray polyfill
if (typeof Array.isArray === 'undefined') {
  Array.isArray = function(obj) {
    return Object.prototype.toString.call(obj) === '[object Array]';
  };
}
//Array.filter polyfill
if (!Array.prototype.filter) {
  Array.prototype.filter = function(fun/*, thisArg*/) {
    if (this === void 0 || this === null) {
      throw new TypeError();
    }
    var t = Object(this);
    var len = t.length >>> 0;
    if (typeof fun !== 'function') {
      throw new TypeError();
    }
    var res = [];
    var thisArg = arguments.length >= 2 ? arguments[1] : void 0;
    for (var i = 0; i < len; i++) {
      if (i in t) {
        var val = t[i];
        if (fun.call(thisArg, val, i, t)) {
          res.push(val);
        }
      }
    }

    return res;
  };
}
//Function.bind polyfill
if (!Function.prototype.bind) {
  Function.prototype.bind = function(oThis) {
    if (typeof this !== 'function') {
      // closest thing possible to the ECMAScript 5
      // internal IsCallable function
      throw new TypeError('Function.prototype.bind - what is trying to be bound is not callable');
    }
    var aArgs   = Array.prototype.slice.call(arguments, 1),
      fToBind = this,
      fNOP    = function() {},
      fBound  = function() {
        return fToBind.apply(this instanceof fNOP
          ? this
          : oThis,
          aArgs.concat(Array.prototype.slice.call(arguments)));
      };
    if (this.prototype) {
      // Function.prototype doesn't have a prototype property
      fNOP.prototype = this.prototype;
    }
    fBound.prototype = new fNOP();
    return fBound;
  };
}
//Array.prototype.indexOf polyfill
// Production steps of ECMA-262, Edition 5, 15.4.4.14
// Referentie: http://es5.github.io/#x15.4.4.14
if (!Array.prototype.indexOf) {
  Array.prototype.indexOf = function(searchElement, fromIndex) {
    var k;
    if (this == null) {
      throw new TypeError('"this" is null or not defined');
    }
    var o = Object(this);
    var len = o.length >>> 0;
    if (len === 0) {
      return -1;
    }
    var n = +fromIndex || 0;
    if (Math.abs(n) === Infinity) {
      n = 0;
    }
    if (n >= len) {
      return -1;
    }
    k = Math.max(n >= 0 ? n : len - Math.abs(n), 0);
    while (k < len) {
      if (k in o && o[k] === searchElement) {
        return k;
      }
      k++;
    }
    return -1;
  };
}
//String.prototype.trim polyfill
if (!String.prototype.trim) {
  String.prototype.trim = function () {
    return this.replace(/^[\s\uFEFF\xA0]+|[\s\uFEFF\xA0]+$/g, '');
  };
}
//Array.prototype.map polyfill
// Production steps of ECMA-262, Edition 5, 15.4.4.19
// Reference: http://es5.github.io/#x15.4.4.19
if (!Array.prototype.map) {
  Array.prototype.map = function(callback/*, thisArg*/) {
    var T, A, k;
    if (this == null) {
      throw new TypeError('this is null or not defined');
    }
    var O = Object(this);
    var len = O.length >>> 0;
    if (typeof callback !== 'function') {
      throw new TypeError(callback + ' is not a function');
    }
    if (arguments.length > 1) {
      T = arguments[1];
    }
    A = new Array(len);
    k = 0;
    while (k < len) {
      var kValue, mappedValue;
      if (k in O) {
        kValue = O[k];
        mappedValue = callback.call(T, kValue, k, O);
        A[k] = mappedValue;
      }
      k++;
    }
    return A;
  };
}

// DocReady - Fires supplied function when document is ready
if (typeof 'docReady' !== 'function') {
  (function (funcName, baseObj) {
    // The public function name defaults to window.docReady
    // but you can pass in your own object and own function name and those will be used
    // if you want to put them in a different namespace
    funcName = funcName || 'docReady';
    baseObj = baseObj || window;
    var i, len, readyList = [], readyFired = false, readyEventHandlersInstalled = false;

    // call this when the document is ready
    // this function protects itself against being called more than once
    function ready() {
      if (!readyFired) {
        // this must be set to true before we start calling callbacks
        readyFired = true;
        for (i = 0, len = readyList.length; i < len; i = i + 1) {
          // if a callback here happens to add new ready handlers,
          // the docReady() function will see that it already fired
          // and will schedule the callback to run right after
          // this event loop finishes so all handlers will still execute
          // in order and no new ones will be added to the readyList
          // while we are processing the list
          readyList[i].fn.call(window, readyList[i].ctx);
        }
        // allow any closures held by these functions to free
        readyList = [];
      }
    }

    function readyStateChange() {
      if (document.readyState === 'complete') {
        ready();
      }
    }

    // This is the one public interface
    // docReady(fn, context);
    // the context argument is optional - if present, it will be passed
    // as an argument to the callback
    baseObj[funcName] = function (callback, context) {
      // if ready has already fired, then just schedule the callback
      // to fire asynchronously, but right away
      if (readyFired) {
        setTimeout(function () {callback(context); }, 1);
        return;
      }
      // add the function and context to the list
      readyList.push({fn: callback, ctx: context});
      // if document already ready to go, schedule the ready function to run
      if (document.readyState === 'complete') {
        setTimeout(ready, 1);
      } else if (!readyEventHandlersInstalled) {
        // otherwise if we don't have event handlers installed, install them
        if (document.addEventListener) {
          // first choice is DOMContentLoaded event
          document.addEventListener('DOMContentLoaded', ready, false);
          // backup is window load event
          window.addEventListener('load', ready, false);
        } else {
          // must be IE
          document.attachEvent('onreadystatechange', readyStateChange);
          window.attachEvent('onload', ready);
        }
        readyEventHandlersInstalled = true;
      }
    };
  }('docReady', window));
}

//define console variable
var console = window.console;

var mobileConsole = (function () {
  'use strict';

  //options and other variable containers
  var options = {
      overrideAutorun: true,
      version : '1.3.5',
      baseClass : 'mobileConsole_',
      animParams: 'all 200ms ease',
      browserinfo: {
        isMobile: (function (a) {
          return (/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|ipad|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino/i.test(a) || /1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s\-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|\-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw\-(n|u)|c55\/|capi|ccwa|cdm\-|cell|chtm|cldc|cmd\-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc\-s|devi|dica|dmob|do(c|p)o|ds(12|\-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(\-|_)|g1 u|g560|gene|gf\-5|g\-mo|go(\.w|od)|gr(ad|un)|haie|hcit|hd\-(m|p|t)|hei\-|hi(pt|ta)|hp( i|ip)|hs\-c|ht(c(\-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i\-(20|go|ma)|i230|iac( |\-|\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\/)|klon|kpt |kwc\-|kyo(c|k)|le(no|xi)|lg( g|\/(k|l|u)|50|54|\-[a-w])|libw|lynx|m1\-w|m3ga|m50\/|ma(te|ui|xo)|mc(01|21|ca)|m\-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(\-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)\-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|\-([1-8]|c))|phil|pire|pl(ay|uc)|pn\-2|po(ck|rt|se)|prox|psio|pt\-g|qa\-a|qc(07|12|21|32|60|\-[2-7]|i\-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h\-|oo|p\-)|sdk\/|se(c(\-|0|1)|47|mc|nd|ri)|sgh\-|shar|sie(\-|m)|sk\-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h\-|v\-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl\-|tdg\-|tel(i|m)|tim\-|t\-mo|to(pl|sh)|ts(70|m\-|m3|m5)|tx\-9|up(\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|\-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(\-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas\-|your|zeto|zte\-/i.test(a.substr(0, 4)));
        }(navigator.userAgent || navigator.vendor || window.opera)),
        browserChrome: /chrome/.test(navigator.userAgent.toLowerCase()),
        ffox: /firefox/.test(navigator.userAgent.toLowerCase()) && !/chrome/.test(navigator.userAgent.toLowerCase()),
        safari: /safari/.test(navigator.userAgent.toLowerCase()) && !/chrome/.test(navigator.userAgent.toLowerCase()),
        trident: /trident/.test(navigator.userAgent.toLowerCase()),
        evtLstn: typeof window.addEventListener === 'function'
      },
      methods : ['log', 'trace', 'info', 'warn', 'error', 'group', 'groupCollapsed', 'groupEnd', 'table', 'assert', 'time', 'timeEnd', 'clear'],
      hideButtons : ['group', 'groupCollapsed', 'groupEnd', 'table', 'assert', 'time', 'timeEnd'],
      ratio: 0.4,
      paddingLeft: 0,
      groupDepth: 0
    },
    messages = {
      clear : 'Console was cleared',
      empty: '(Empty string)'
    },
    status = {
      initialized: false,
      acActive : false,
      acHovered : false,
      acInput : '',
      timers : {}
    },
    history = {
      output : {
        prevMsg : '',
        prevMethod : '',
        counter : 0
      },
      input : {
        commands : window.sessionStorage ? (sessionStorage.getItem('mobileConsoleCommandHistory') ? JSON.parse(sessionStorage.getItem('mobileConsoleCommandHistory')) : []) : [],
        commandIdx: window.sessionStorage ? (sessionStorage.getItem('mobileConsoleCommandHistory') ? JSON.parse(sessionStorage.getItem('mobileConsoleCommandHistory')).length : 0) : 0,
        acIdx: 0,
        acHovered: false
      }
    },
    //'backup' original console for reference & internal debugging
    missingMethod = function() { return true; },  //method is not supported on this device's original console, return dummy
    originalConsole = {
      log:        (console && typeof console.log === 'function') ?       console.log.bind(console) :       missingMethod,
      info:       (console && typeof console.info === 'function') ?      console.info.bind(console) :      missingMethod,
      dir:        (console && typeof console.dir === 'function') ?       console.dir.bind(console) :       missingMethod,
      group:      (console && typeof console.group === 'function') ?     console.group.bind(console) :     missingMethod,
      groupEnd:   (console && typeof console.groupEnd === 'function') ?  console.groupEnd.bind(console) :  missingMethod,
      warn:       (console && typeof console.warn === 'function') ?      console.warn.bind(console) :      missingMethod,
      error:      (console && typeof console.error === 'function') ?     console.error.bind(console) :     missingMethod,
      trace:      (console && typeof console.trace === 'function') ?     console.trace.bind(console) :     missingMethod,
      clear:      (console && typeof console.clear === 'function') ?     console.clear.bind(console) :     missingMethod
    },
    // reference variables
    mobileConsole, consoleElement, commandLine;

  //helpers for all sub functions
  function setCSS(el, css) {
    var i;
    for (i in css) {
      if (css.hasOwnProperty(i)) {
        el.style[i] = css[i];
      }
    }
    return el;
  }
  function htmlToString(html) {
    var string;
    try { string = String(html); } catch(e) { string = JSON.stringify(html); } //this should be done differently, but works for now
    return string.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/ /g, '\u00a0').replace(/(?:\r\n|\r|\n)/g, '<br />').trim();
  }
  function createElem(type, className, css) {
    if (!type || typeof setCSS !== 'function') { return; }
    var element = setCSS(document.createElement(type), css);
    if (className) { element.className = options.baseClass + className; }
    return setCSS(element, css);
  }
  function storeCommand(command) {
    if (history) {
      history.input.commands.push(encodeURI(command.trim()));
      history.input.commandIdx = history.input.commands.length;
      if (window.sessionStorage) { sessionStorage.setItem('mobileConsoleCommandHistory', JSON.stringify(history.input.commands)); }
    }
  }
  function valBetween(val, min, max) {
    return (Math.min(max, Math.max(min, val)));
  }
  function getMaxHeight() {
    return valBetween(Math.floor((window.innerHeight || document.documentElement.clientHeight) * options.ratio), 55, 300);
  }
  function getClass(item) {
    var returnVal = '';
    if (item && item.constructor) {
      returnVal = item.constructor.name;
    } else {
      returnVal = Object.prototype.toString.call(item);
    }
    return String(returnVal);
  }

  // elements
  var elements = {
    lines: [],
    acItems: [],
    base: createElem('div', 'base', {
      boxSizing: 'border-box',
      position: 'fixed',
      resize: 'none',
      fontSize: '12px',
      lineHeight: '14px',
      bottom: 0,
      top: 'auto',
      right: 0,
      width: '100%',
      zIndex: 10000,
      padding: 0,
      paddingBottom: options.browserinfo.isMobile ? '35px' : '25px',
      margin: 0,
      border: '0 none',
      borderTop: '1px solid #808080',
      backgroundColor: '#ffffff'
    }),
    topbar : createElem('div', 'topbar', {
      boxSizing: 'border-box',
      position: 'absolute',
      height: '28px',
      left: 0,
      right: 0,
      display: 'block',
      padding: '0 2px',
      overflow: 'hidden',
      webkitOverflowScrolling: 'touch',
      color: '#444444',
      backgroundColor: '#f3f3f3',
      border: '0 none',
      borderTop: '1px solid #a3a3a3',
      borderBottom: '1px solid #a3a3a3',
      whiteSpace: 'nowrap',
      overflowX: 'auto'
    }),
    scrollcontainer : createElem('div', 'scroller', {
      boxSizing: 'border-box',
      border: '0 none',
      fontFamily: 'Consolas, monaco, monospace',
      position: 'relative',
      display: 'block',
      height: getMaxHeight() + 'px',
      overflow: 'auto',
      webkitOverflowScrolling: 'touch',
      '-webkit-transition': options.animParams,
      '-moz-transition': options.animParams,
      '-o-transition': options.animParams,
      'transition': options.animParams
    }),
    table : createElem('table', 'table', {
      border: '0 none',
      margin: 0,
      position: 'relative',
      tableLayout: 'auto',
      width: '100%',
      borderCollapse: 'collapse'
    }),
    stackTraceTable : createElem('table', 'stackTraceTable', {
      border: '0 none',
      margin: 0,
      display: 'none',
      marginLeft: '10px',
      marginTop: options.browserinfo.isMobile ? '8px' : '4px',
      tableLayout: 'auto',
      maxWidth: '100%',
      color: '#333333'
    }),
    tr : createElem('tr', 'table_row', {
      verticalAlign: 'top'
    }),
    td : createElem('td', 'table_row', {
      border: '0 none',
      padding: '2px 4px',
      verticalAlign: 'top'
    }),
    msgContainer : createElem('span', 'msgContainer', {
      border: '0 none',
      margin: 0,
      display: 'inline',
      overflow: 'hidden'
    }),
    tdLeft : createElem('td', 'table_row_data', {
      border: '0 none',
      textAlign: 'left',
      padding: options.browserinfo.isMobile ? '8px 12px' : '4px 8px'
    }),
    tdRight : createElem('td', 'table_row_data', {
      border: '0 none',
      textAlign: 'left',
      padding: options.browserinfo.isMobile ? '8px 12px' : '4px 8px',
      whiteSpace: 'nowrap',
      overflow: 'hidden'
    }),
    link : createElem('a', 'link', {
      color: '#1155cc',
      textDecoration: 'underline'
    }),
    dot : createElem('div', 'table_row_data_dot', {
      display: 'inline',
      borderRadius: '50%',
      fontSize: '80%',
      fontWeight: 'bold',
      padding: '2px 5px',
      textAlign: 'center',
      marginRight: '5px',
      backgroundColor: '#333333',
      color: '#ffffff'
    }),
    button : createElem('button', 'button', {
      display: 'inline-block',
      fontFamily: '"Helvetica Neue",Helvetica,Arial,sans-serif',
      fontWeight: 'normal',
      textTransform: 'capitalize',
      fontSize: '12px',
      lineHeight: '26px',
      height: '26px',
      padding: '0 8px',
      margin: 0,
      textAlign: 'center',
      marginRight: '5px',
      border: '0 none',
      backgroundColor: 'transparent',
      color: 'inherit',
      cursor: 'pointer'
    }),
    buttons : {
    },
    input : createElem('div', 'input', {
      boxSizing: 'border-box',
      height: options.browserinfo.isMobile ? '35px' : '29px',
      fontFamily: 'Consolas, monaco, monospace',
      position: 'absolute',
      bottom: 0,
      left: 0,
      right: 0,
      margin: 0,
      border: '0 none',
      borderTop: '1px solid #EEEEEE'
    }),
    gt : createElem('DIV', 'gt', {
      position: 'absolute',
      bottom: 0,
      width: '25px',
      lineHeight: options.browserinfo.isMobile ? '34px' : '28px',
      height: options.browserinfo.isMobile ? '34px' : '28px',
      textAlign: 'center',
      fontSize: '16px',
      fontFamily: 'Consolas, monaco, monospace',
      fontWeight: 'bold',
      color: '#3577B1',
      zIndex: 2
    }),
    consoleinput : createElem('input', 'consoleinput', {
      boxSizing: 'border-box',
      position: 'absolute',
      bottom: 0,
      width : '100%',
      fontSize: options.browserinfo.isMobile ? '16px' : 'inherit', //prevents ios safari's zoom on focus
      fontFamily: 'Consolas, monaco, monospace',
      paddingLeft: '25px',
      margin: 0,
      height: options.browserinfo.isMobile ? '35px' : '25px',
      border: '0 none',
      outline: 'none',
      outlineWidth: 0,
      boxShadow: 'none',
      '-moz-appearance': 'none',
      '-webkit-appearance': 'none',
      backgroundColor: 'transparent',
      color: '#000000',
      zIndex: 1
    }),
    autocomplete : createElem('div', 'autocomplete', {
      display: 'none',
      position: 'absolute',
      bottom: options.browserinfo.isMobile ? '35px' : '28px',
      left: 0,
      boxShadow: '1px 2px 5px rgba(0,0,0,0.1)',
      color: '#000000',
      backgroundColor: '#FFFFFF',
      border: '1px solid #b5b5b5'
    }),
    autocompleteItem : createElem('a', 'autocompleteitem', {
      display: 'block',
      textDecoration: 'none',
      fontSize: options.browserinfo.isMobile ? '16px' : 'inherit',
      padding: '5px 8px',
      wordWrap: 'break-word',
      whiteSpace: 'nowrap'
    }),
    arrowUp: '<img width="10" height="10" src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAcAAAAHCAMAAADzjKfhAAAACVBMVEUAAAD///93d3eMZ/YKAAAAAnRSTlMAAHaTzTgAAAAdSURBVHgBY2CEAAYgYAJiEMXEBKHADCYIgKmD0QAFdAA2OHJXEwAAAABJRU5ErkJggg==">',
    arrowDown: '<img width="10" height="10" src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAcAAAAHCAYAAADEUlfTAAAAG0lEQVR42mNgwAfKy8v/48I4FeA0AacVDFQBAP9wJkE/KhUMAAAAAElFTkSuQmCC">',
    arrowRight: '<img width="10" height="10" src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAcAAAAHCAYAAADEUlfTAAAAJUlEQVR42mNgAILy8vL/DLgASBKnApgkVgXIkhgKiNKJ005s4gDLbCZBiSxfygAAAABJRU5ErkJggg==">'
  };

  //shared functions

  var setLineStyle = (function () {
      var lineStyles = function (style) {
        switch (style) {
          case 'log':
            return {
              text : {
                borderBottom: '1px solid #DDDDDD',
                color: '#000000'
              },
              dot : {
                color: '#FFFFFF',
                backgroundColor: '#8097bd'
              }
            };
          case 'info':
            return {
              text : {
                borderBottom: '1px solid #DDDDDD',
                color: '#1f3dc4'
              },
              dot : {
                color: '#FFFFFF',
                backgroundColor: '#367AB4'
              }
            };
          case 'warn':
            return {
              text : {
                borderBottom: '1px solid #DDDDDD',
                color: '#CE8724',
                backgroundColor : '#fff6e0'
              },
              dot : {
                color: '#FFFFFF',
                backgroundColor: '#e8a400'
              }
            };
          case 'error':
          case 'table':
            return {
              text : {
                borderBottom: '1px solid #DDDDDD',
                color: '#FF0000',
                backgroundColor :  '#ffe5e5'
              },
              dot : {
                color: '#FFFFFF',
                backgroundColor: '#FF0000'
              }
            };
          case 'assert':
            return {
              text : {
                borderBottom: '1px solid #DDDDDD',
                color: '#FF0000',
                backgroundColor :  '#ffe5e5'
              },
              dot : {
                color: '#FFFFFF',
                backgroundColor: '#FF0000'
              }
            };
          case 'trace':
            return {
              text : {
                borderBottom: '1px solid #DDDDDD',
                color: '#000000'
              },
              dot : {
                //will not happen
              }
            };
          case 'time':
          case 'timeEnd':
            return {
              text : {
                borderBottom: '1px solid #DDDDDD',
                color: '#0000ff'
              },
              dot : {
                color: '#FFFFFF',
                backgroundColor: '#0000ff'
              }
            };
          default:
            return {
              text : {
                borderBottom: '1px solid #DDDDDD',
                color: '#000000'
              },
              dot : {
                color: '#FFFFFF',
                backgroundColor: '#8097bd'
              }
            };
        }

      };
      var color, dot;

      return function (element, type, msg) {
        if (status.initialized) {
          color = (typeof msg === 'undefined' || msg === htmlToString(messages.empty)) ? {color: '#808080'} : ((msg  === htmlToString(messages.clear)) ? {color: '#808080', fontStyle: 'italic'} : (lineStyles(type) !== undefined ? lineStyles(type).text : lineStyles.log.text));
          dot = typeof lineStyles(type) !== 'undefined' ? lineStyles(type).dot : lineStyles.log.dot;
          setCSS(element, color);
          //has dot?
          if (element.childNodes[0].childNodes[0].className.indexOf('dot') !== -1) {
            setCSS(element.childNodes[0].childNodes[0], lineStyles(type).dot);
          }
        }
      };
    }()),
    getLink = function (href, textString) {
      var HTMLurl = elements.link.cloneNode(false);
      if (href) {
        HTMLurl.setAttribute('href', href);
        HTMLurl.setAttribute('target', '_blank');
      }
      HTMLurl.innerHTML = textString || href.split('\\').pop().split('/').filter(Boolean).pop();
      return HTMLurl;
    },
    toggleHeight = function () {
      if (status.initialized) {
        var existingPadding = parseInt(document.body.style.paddingBottom, 10) - Math.abs(elements.base.offsetHeight + elements.topbar.offsetHeight);
        var newHeight = (elements.base.minimized) ? getMaxHeight() + 'px' : '0px';
        setCSS(elements.scrollcontainer, {
          height: newHeight
        });
        setCSS(document.body, {
          paddingBottom: existingPadding + Math.abs(parseInt(newHeight, 10) + elements.topbar.offsetHeight) + 'px'
        });
        elements.buttons.toggler.innerHTML = (elements.base.minimized) ? elements.arrowDown : elements.arrowUp;
        elements.buttons.toggler.setAttribute('title', (elements.base.minimized) ? 'Minimize console' : 'Maximize console');
        elements.base.minimized = !elements.base.minimized;
        return elements.base.minimized;
      }
      return 'Not built!';
    },
    about = (function () {
      return function () {
        console.info(
          '--==## Mobile Console ' + (status.initialized ? 'active' : 'inactive') + ' ##==--' + '\n' +
          '--===============================--' + '\n' +
          'MobileConsole v' + options.version + ', running on ' + navigator.userAgent.toLowerCase()
        );
      };
    }());

  // --==** sub functions start here **==--

  //initializes the console HTML element
  function initConsoleElement() {
    //reference
    var ref;
    //core
    function toggleScroll() {
      elements.scrollcontainer.scrollTop = elements.scrollcontainer.scrollHeight;
      elements.scrollcontainer.scrollLeft = 0;
    }
    function destroyConsole() {
      //conan the destroyer. Very basic; just removes the console element. mobileConsole will still 'pipe' console logging
      //don't see any reason for now to reverse that.
      elements.base.parentNode.removeChild(elements.base);
      status.initialized = false;
      console.warn(
        '--==## Mobile Console DESTROYED ##==--' + '\n' +
        'To enable again: reload the page. Tip: use the minimize button instead of closing.'
      );
    }
    function assemble() {
      var i = options.methods.length, key;

      //add buttons
      while (i--) {
        elements.buttons[options.methods[i]] = elements.button.cloneNode(false);
        elements.buttons[options.methods[i]].innerHTML = options.methods[i].charAt(0).toUpperCase() + options.methods[i].slice(1);
        elements.buttons[options.methods[i]].setAttribute('title', (options.methods[i] !== 'clear') ? 'Toggle the display of ' + options.methods[i] + ' messages' : 'Clear the console');
      }
      //add min/maximize button
      elements.buttons.toggler = elements.button.cloneNode(false);
      elements.buttons.toggler.innerHTML = elements.arrowDown;
      elements.buttons.toggler.setAttribute('title', 'Minimize console');
      //add close button
      elements.buttons.closer = elements.button.cloneNode(false);
      elements.buttons.closer.innerHTML = '&#10005;';
      elements.buttons.closer.setAttribute('title', 'Close (destroy) console');
      setCSS(elements.buttons.closer, { float: 'right', margin: '0'});

      //assemble everything
      for (key in elements.buttons) {
        if (elements.buttons.hasOwnProperty(key)) {
          elements.topbar.insertBefore(elements.buttons[key], elements.topbar.firstChild);
        }
      }
      elements.scrollcontainer.appendChild(elements.table);

      elements.base.appendChild(elements.topbar);
      elements.base.appendChild(elements.scrollcontainer);

      status.initialized = true;
      return elements.base;
    }
    function attach(console) {
      document.body.appendChild(console);
      setCSS(elements.topbar, {
        top: -Math.abs(elements.topbar.offsetHeight) + 'px'
      });
      var existingPadding = isNaN(parseInt(document.body.style.paddingBottom, 10)) ? 0 : parseInt(document.body.style.paddingBottom, 10);
      setCSS(document.body, {
        paddingBottom: existingPadding + Math.abs(console.offsetHeight + elements.topbar.offsetHeight) + 'px'
      });
      elements.scrollcontainer.scrollTop = elements.scrollcontainer.scrollHeight;

      return elements.base;
    }
    function toggleLogType(e) {
      var button = e.currentTarget || e.srcElement;
      var logType = button.innerHTML.toLowerCase();
      var elems = elements.lines[logType], i = elems.length;
      button.toggled = (typeof button.toggled === 'undefined') ? true : !button.toggled;
      setCSS(button, { opacity: (button.toggled) ? '0.5' : '' });
      while (i--) {
        setCSS(elems[i], { display: (button.toggled) ? 'none' : '' });
      }
      toggleScroll();
      button.blur();
      return button;
    }
    function setBinds() {
      var methods = options.methods, i = methods.length;
      while (i--) {
        if (methods[i] !== 'clear') {
          if (options.browserinfo.evtLstn) {
            elements.buttons[methods[i]].addEventListener('click', toggleLogType, false);
          } else {
            elements.buttons[methods[i]].attachEvent('onclick', toggleLogType);
          }
        }
        if (options.hideButtons.indexOf(methods[i]) !== -1) { //hide buttons that we don't want
          setCSS(elements.buttons[methods[i]], { display: 'none' });
        }
      }
      if (options.browserinfo.evtLstn) {
        elements.buttons.toggler.addEventListener('click', toggleHeight, false);
        elements.buttons.closer.addEventListener('click', destroyConsole, false);
        elements.buttons.clear.addEventListener('click', console.clear, false);
      } else {
        elements.buttons.toggler.attachEvent('onclick', toggleHeight);
        elements.buttons.closer.attachEvent('onclick', destroyConsole);
        elements.buttons.clear.attachEvent('onclick', console.clear);
      }
    }
    //init
    function init() {
      var element = assemble();
      docReady(function () {
        setBinds();
        attach(element);
      });
      //expose Public methods and variables
      return {
        toggleHeight : toggleHeight,
        toggleScroll : toggleScroll,
        destroy: destroyConsole
      };
    }
    if (!ref) {
      ref = init();
    }
    return ref;
  }

  //initializes the new console logger
  function initConsole() {
    //reference
    var ref;
    //sub helpers
    function isEmpty(obj) {
      for(var prop in obj) {
        if(obj.hasOwnProperty(prop))
          return false;
      }
      return JSON.stringify(obj) === JSON.stringify({});
    }
    function isElement(o) {
      return (
        typeof HTMLElement === 'object' ? o instanceof HTMLElement : //DOM2
        o && typeof o === 'object' && o !== null && o.nodeType === 1 && typeof o.nodeName === 'string'
      );
    }
    function objectToString(object) {
      var prop, output = '';
      if (!isElement(object)) {
        for (prop in object) {
          if (!object.hasOwnProperty(prop)) {
            continue;
          } else if (typeof (object[prop]) === 'object') {
            output += prop + ((Array.isArray(object[prop])) ? ': Array(' + object[prop].length + ')' : ': {&hellip;}');
          } else if (typeof (object[prop]) === 'function') {
            output += 'func: f';
          } else {
            output += prop + ': <span style="color:#c54300;">"' + object[prop] + '"</span>';
          }
          output += ', ';
        }
        return '<em>{' + output.slice(0, -2) + '}</em>'; // returns cleaned up JSON
      }
      return htmlToString(object.outerHTML);
    }
    function urlFromString(string) {
      string =  String(string);
      //searches for url in string, returns url as string
      var match, uriPattern = /\b((?:[a-z][\w-]+:(?:\/{1,3}|[a-z0-9%])|www\d{0,3}[.]|[a-z0-9.\-]+[.][a-z]{2,4}\/)(?:[^\s()<>]+|\(([^\s()<>]+|(\([^\s()<>]+\)))*\))+(?:\(([^\s()<>]+|(\([^\s()<>]+\)))*\)|[^\s`!()\[\]{};:'".,<>?«»“”‘’]))/ig;
      try {
        match = string.match(uriPattern)[0];
        return match;
      } catch (e) {
        return '';
      }
    }
    function filterOut(array, match) {
      return array.filter(function(item){
        return typeof item === 'string' && item.indexOf(match) === -1;
      });
    }
    function preFilterTrace(array) {
      var newArray = array.split('\n').filter(Boolean), //filter cleans out empty values
        isCommandLine = false, stealthThese, i;
      if (newArray[0].indexOf('http') === -1) { newArray.shift(); } //remove first line if contains no 'http' (Chrome starts with 'Error', Firefox doesn't..)
      if (newArray[0].indexOf('console.') !== -1 || newArray[0].indexOf('console[method]') !== -1) { newArray.shift(); }
      if (newArray.length > 0) {
        isCommandLine = newArray[newArray.length - 1].indexOf('keydown') !== -1;
        newArray = newArray.filter(function(item){ return item !== ''; });

        if (isCommandLine) {
          stealthThese = ['submitCommand', 'eval', 'setBinds', 'interceptConsole', 'newConsole'];
          newArray.pop(); //remove last index, as it is the keydown event.
          i = stealthThese.length;
          while(i--) {
            newArray = filterOut(newArray, stealthThese[i]);
          }
        }
      }
      if (isCommandLine || newArray.length === 0) {
        newArray.push('(anonymous function) console:1:1');
      }
      return newArray;
    }
    //core
    function formatStackTrace(trace, origtrace) {
      var callStack = [];
      //original stack is hidden inside trace object, if specified
      var stackTraceOrig = (typeof trace !== 'undefined' && typeof trace[4] !== 'undefined') ? trace[4].stack : undefined;
      //if the first line contains this, skip it. Meant for browsers that begin the stack with the error message itself (already captured before formatStackTrace)
      var traceToProcess = (origtrace && origtrace !== '') ? origtrace : stackTraceOrig,
        i,
        lines,
        url,
        txt,
        thisLine,
        lineAndColumn,
        caller,
        separator = options.browserinfo.ffox ? '@' : '()';

      //stop if no source trace can be determined
      if (!traceToProcess) { return; }

      lines = preFilterTrace(traceToProcess); //pre filters all lines by filtering out all mobileConsole's own methods so mobileConsole runs Stealth and unobtrusive
      i = lines.length;
      while (i--) {
        thisLine = lines[i].trim();
        lineAndColumn = thisLine.match(/(?::)(\d+)(?::)(\d+)/);
        url = urlFromString(thisLine).replace(lineAndColumn[0], '').split('#')[0] || '';
        caller = htmlToString(thisLine.replace(urlFromString(thisLine), '').replace(separator, '').replace('at ', '').trim());
        if (caller === '' || caller === lineAndColumn[0]) { continue; }
        if (url[url.length - 1] === '/') {
          txt = '(index)';
        } else {
          txt = url.split('\\').pop().split('/').filter(Boolean).pop() || caller;
        }
        callStack.push({
          caller: caller,
          url:    url ? url.split(':')[0] + ':' + url.split(':')[1] : caller,
          linkText: txt + lineAndColumn[0],
          line:   lineAndColumn[1],
          col:    lineAndColumn[2],
          originalLine: thisLine
        });
      }
      return callStack;
    }
    function traceToTable(table, trace) {
      var i, tdLeft, tdRight, tr;
      if (typeof trace === 'undefined') {
        return;
      }
      trace.reverse(); //reverse order of trace, just like in a browser's console
      i = trace.length;
      while (i--) {
        tdLeft = elements.td.cloneNode(false);
        tdRight = elements.td.cloneNode(false);
        tr = elements.tr.cloneNode(false);
        tdLeft.innerHTML = trace[i].caller;
        tdRight.innerHTML = '&nbsp;@&nbsp;';
        tdRight.appendChild(getLink((trace[i].url || ''), trace[i].linkText));
        tr.appendChild(tdLeft);
        tr.appendChild(tdRight);
        table.insertBefore(tr, table.firstChild);
      }
      return table;
    }
    function colorizeData(key, value) {
      var valueColor =  '#3c53da', keyColor = '#ae33b7', classname = getClass(value);
      if (value && classname.indexOf('HTML') !== -1) {
        value = htmlToString(value.outerHTML);
        valueColor = '#ad8200';
      } else if (key === 'innerHTML' || key === 'outerHTML') {
        value = htmlToString(value);
        valueColor = '#ad8200';
      }
      if (typeof value === 'string') {
        valueColor = '#c54300';
        //HARD limit, for speed/mem issues with consecutive logging of large strings
        if (value.length > 400) {
          value = '"' + String(value).substring(0, 400) + '" [...] <br/><span style="color:#FF0000;text-decoration: underline;">Note: string was truncated to 400 chars</span>';
        } else {
          value = '"' + value + '"';
        }
      } else if (value === null) {
        valueColor = '#808080';
        value = 'null';
      } else if (typeof value === 'undefined' || value === undefined) {
        valueColor = '#808080';
        value = 'undefined';
      } else if (typeof value === 'object') {
        if (isEmpty(value)) {
          value = '{}';
        } else {
          valueColor = '';
          //iterate over object to create another table inside
          var tempTable = createElem('table', 'stackTraceSubTable', {
              border: '0 none',
              margin: 0,
              display: 'none',
              marginLeft: '10px',
              marginTop: options.browserinfo.isMobile ? '8px' : '4px',
              tableLayout: 'auto',
              maxWidth: '100%',
              color: '#333333'
            }),
            wrap = document.createElement('div');
          wrap.appendChild(objectToTable(tempTable, value).cloneNode(true));
          if (Array.isArray(value)) {
            value = 'Array(' + value.length + ')' +  wrap.innerHTML;
          } else {
            value = wrap.innerHTML;
          }
        }
      }

      return '<span style="color:' + keyColor + ';">' + key + ':</span> <span style="color:' + valueColor + ';">' + value + '</span>';
    }
    function objectToTable(table, object) {
      var i, tdLeft, tr;
      if (isElement(object)){
        tdLeft = elements.td.cloneNode(false); tr = elements.tr.cloneNode(false);
        tdLeft.innerHTML = htmlToString(object.outerHTML);
        tr.appendChild(tdLeft);
        table.appendChild(tr);
      } else {
        for (i in object) {
          if (object.hasOwnProperty(i)) {
            tdLeft = elements.td.cloneNode(false); tr = elements.tr.cloneNode(false);
            tdLeft.innerHTML = colorizeData(i, object[i]);
            tr.appendChild(tdLeft);
            table.appendChild(tr);
          }
        }
      }
      return table;
    }
    function toggleDetails(e) {
      var button = e.currentTarget || e.srcElement, i, hidden;
      if (button.getAttribute('toggles') === 'table') {
        var tables = button.parentElement.getElementsByTagName('table');
        i = tables.length;
        while (i--) {
          hidden = (tables[i].currentStyle ? tables[i].currentStyle.display : window.getComputedStyle(tables[i], null).display) === 'none';
          button.innerHTML = button.innerHTML.replace((hidden ? elements.arrowRight : elements.arrowDown), (hidden ? elements.arrowDown : elements.arrowRight));
          setCSS(tables[i], { display: hidden ? 'table' : 'none' });
        }
      }
    }
    function isRepeat(message, method) {
      return (history.output.prevMsg === message && history.output.prevMethod === method) && (typeof message !== 'object') && (method !== 'trace') && (method !== 'group') && (method !== 'groupCollapsed') && (method !== 'groupEnd');
    }
    function newConsole() {
      try {
        //get arguments, set vars
        var method = arguments[0], className, isHTMLElement,
          message =       (typeof arguments[1].newMessage !== 'undefined') ? arguments[1].newMessage : undefined,
          stackTrace =    (typeof arguments[1].newStackTrace !== 'undefined') ? arguments[1].newStackTrace : undefined;

        //if message emtpy or undefined, show empty message-message
        if (message === '' || typeof message === 'undefined' || message === undefined) { message = messages.empty; }

        if (isRepeat(message, method) && method.indexOf('time') === -1) {
          // up the counter and add the dot
          history.output.counter = history.output.counter + 1;
          elements.table.lastChild.countDot = elements.table.lastChild.countDot || elements.dot.cloneNode(false);
          elements.table.lastChild.firstChild.insertBefore(elements.table.lastChild.countDot, elements.table.lastChild.firstChild.firstChild).innerHTML = history.output.counter;
          setLineStyle(elements.table.lastChild, method, message);
        } else {
          history.output.prevMsg = message;
          history.output.prevMethod = method;
          history.output.counter = 1;

          //an object requires some more handling
          if (typeof message === 'object' && method !== 'assert' && method !== 'timeEnd') {
            message = isElement(message) ?
                      htmlToString(message.outerHTML.match(/<(.*?)>/g)[0] + '...' + message.outerHTML.match(/<(.*?)>/g).pop()) : //gets e.g. <div>...</div>
                      objectToString(message);
          } else if (method !== 'assert' && method.indexOf('time') === -1) {
            message = htmlToString(message);
          }

          var detailTable,
            stackTable,
            msgContainer =    elements.msgContainer.cloneNode(false),
            lineContainer =   elements.tr.cloneNode(false),
            leftContainer =   elements.tdLeft.cloneNode(true),
            rightContainer =  elements.tdRight.cloneNode(false),
            arrows =          stackTrace ? elements.arrowRight + '&nbsp;' : '';

          switch (method) {
            case 'assert':
              if (message[0] === false) {
                msgContainer.innerHTML = arrows + 'Assertion failed: ' + message[1];
              }
              stackTable = traceToTable(elements.stackTraceTable.cloneNode(false), stackTrace);
              method = 'error'; //groups it under 'error' and is thus toggleable in view
              break;
            case 'log':
            case 'debug':
            case 'info':
            case 'warn':
              if (typeof arguments[1].newMessage === 'object') {
                detailTable = objectToTable(elements.stackTraceTable.cloneNode(false), arguments[1].newMessage);
                msgContainer.innerHTML = elements.arrowRight + '&nbsp;' + message;
              } else {
                msgContainer.innerHTML = message;
              }
              break;
            case 'error':
            case 'trace':
            case 'dir':
            case 'table':
              //left side
              if (method === 'table' || typeof arguments[1].newMessage === 'object') {
                detailTable = objectToTable(elements.stackTraceTable.cloneNode(false), arguments[1].newMessage);
                msgContainer.innerHTML = elements.arrowRight + '&nbsp;' + message;
              } else if (method === 'trace') {
                message = 'console.trace()';
                msgContainer.innerHTML = arrows + message;
              } else {
                msgContainer.innerHTML = arrows + message;
              }
              stackTable = traceToTable(elements.stackTraceTable.cloneNode(false), stackTrace);
              break;
            case 'group':
            case 'groupCollapsed':
            case 'groupEnd':
              if (method !== 'groupEnd') {
                options.groupDepth = options.groupDepth + 1;
                msgContainer.innerHTML = '<strong>' + message + '</strong>';
                msgContainer.setAttribute('toggles', 'group_' + options.groupDepth);
              } else {
                options.groupDepth = valBetween(options.groupDepth - 1, 0, 99);
                history.output.prevMsg = '';
              }
              if (options.groupDepth > 0) {
                options.paddingLeft = (options.groupDepth * 23) + 'px';
              } else {
                options.paddingLeft = 0;
              }
              break;
            case 'time':
            case 'timeEnd':
              var timerName = arguments[1].newMessage || 'default', now, passed;
              if (method === 'time') {
                status.timers[timerName] = Date.now();
                if (typeof arguments[1].original === 'function') {
                  arguments[1].original.apply(console, arguments[1].originalArguments); //make sure we still call the original console.time to start the browser's console timer
                }
                return;
              }
              now = Date.now();
              if (!status.timers[timerName]) {
                console.warn('Timer "' + timerName + '" does not exist.');
                return;
              }
              passed = now - (status.timers[timerName] || 0);
              message = timerName + ': ' + passed + 'ms';
              msgContainer.innerHTML = message;
              delete status.timers[timerName];
              break;
            default:
              msgContainer.innerHTML = message;
          }

          if (!msgContainer.innerHTML) { return; }
          leftContainer.appendChild(msgContainer);

          if (detailTable || stackTable) {
            setCSS(msgContainer, {cursor : 'pointer'});
            leftContainer.appendChild(detailTable || stackTable);
            msgContainer.setAttribute('toggles', 'table');
          }

          //populate right side
          if (stackTrace && typeof stackTrace[stackTrace.length - 1] !== 'undefined') {
            rightContainer.appendChild(setCSS(getLink(stackTrace[0].url, stackTrace[0].linkText), {color: '#808080'}));
          }

          //add to line
          lineContainer.appendChild(leftContainer);
          lineContainer.appendChild(rightContainer);

          //set colors
          setCSS(lineContainer, { display: (elements.buttons[method].toggled ? 'none' : '') });
          setLineStyle(lineContainer, method, message);

          //set binds
          if (options.browserinfo.evtLstn) {
            msgContainer.addEventListener('click', toggleDetails, false);
          } else {
            msgContainer.attachEvent('onclick', toggleDetails);
          }

          //store the lines in the object corresponding to the method used
          elements.lines[method].push(lineContainer);

          //handle grouping (group and groupEnd
          if (options.paddingLeft !== 0) {
            setCSS(leftContainer, {paddingLeft: options.paddingLeft});
            setCSS(msgContainer, {borderLeft: '1px solid #808080', paddingLeft: '5px'});
          }

          //add the line to the table
          elements.table.appendChild(lineContainer);
        }
        //scroll
        consoleElement.toggleScroll();
        //==========================================================
        //make sure we still call the original method, if applicable (not window.onerror)
        if (typeof arguments[1].original === 'function') {
          arguments[1].original.apply(console, arguments[1].originalArguments);
        }
      } catch (e) {
        //not logging. why? throw error
        if (options.browserinfo.isMobile) { alert(e); }
        originalConsole.error('mobileConsole generated an error logging this event! (type: ' + typeof message + ')');
        originalConsole.error(arguments);
        originalConsole.error(e);
        //try to re-log it as an error
        newConsole('error', e);
      }

    }
    function interceptConsole(method) {
      var original = console ? console[method] : missingMethod(), i, stackTraceOrig;
      if (!console) { console = {}; } //create empty console if we have no console (IE?)
      console[method] = function () {
        var args = Array.prototype.slice.call(arguments);
        args.original = original;
        args.originalArguments = arguments;
        args.newMessage = (method === 'assert') ? [args[0], args[1]] : args[0];
        //create an Error and get its stack trace and format it
        try { throw new Error(); } catch (e) { stackTraceOrig = e.stack; }
        args.newStackTrace = formatStackTrace(args.newStackTrace, stackTraceOrig);
        if (method === 'clear') {
          try {
            elements.table.innerHTML = '';
          } catch (e) {
            console.log('This browser does not allow clearing tables, the console cannot be cleared.');
            return;
          }
          history.output.prevMethod = '';
          i = options.methods.length;
          while (i--) {
            elements.lines[options.methods[i]] = [];
          }
          options.groupDepth = 0;
          options.paddingLeft = 0;
          console.log(messages.clear);
          originalConsole.clear();
          return;
        }
        //Handle the new console logging
        newConsole(method, args);
      };
    }
    //init
    function init() {
      //Intercept all original console methods including trace. Register the event type as a line type.
      var i = options.methods.length;
      while (i--) {
        elements.lines[options.methods[i]] = [];
        interceptConsole(options.methods[i]);
      }
      //Bind to window.onerror
      window.onerror = function() {
        var args = Array.prototype.slice.call(arguments);
        args.newMessage = args[0];
        args.newStackTrace = formatStackTrace(arguments);
        newConsole('error', args);
      };

      return {
        //nothing to expose
      };
    }
    //return
    if (!ref) {
      ref = init();
    }
    return ref;
  }

  //initialize the console commandline
  function initCommandLine() {
    //reference
    var ref;
    //sub helpers
    function getFromArrayById(id) {
      var pos = elements.acItems.map(function(x) {return x.id; }).indexOf(id);
      return {
        position: pos,
        element: (pos !== -1) ? elements.acItems[pos] : undefined
      };
    }
    function findInArray(array, match) {
      return array.filter(function(item, index, self){
        return (typeof item === 'string' && item.indexOf(match) > -1) && (index === self.indexOf(item));
      });
    }
    //core
    function assemble() {
      elements.consoleinput.setAttribute('type', 'text');
      elements.consoleinput.setAttribute('autocapitalize', 'off');
      elements.consoleinput.setAttribute('autocorrect', 'off');
      elements.autocompleteItem.setAttribute('href', '#');
      elements.gt.innerHTML = '&gt;';
      elements.input.appendChild(elements.gt);
      elements.input.appendChild(elements.consoleinput);
      elements.input.appendChild(elements.autocomplete);
      elements.base.appendChild(elements.input);

      return elements.base;
    }
    function submitCommand(command) {
      if (command !== '') {
        storeCommand(command);
        var result;
        try {
          result = eval.call(window, command.trim());
          console.log.call(window, result);
        } catch(e) {
          console.error(e.message);
        } finally {
          elements.consoleinput.value = '';
        }
      }
    }
    function hoverAutoComplete(e) {
      if (typeof e === 'undefined') { return; }
      //unset any already hovered elements
      var hovered = getFromArrayById('hover').element, target = e.target || e.srcElement, over;
      if (typeof hovered !== 'undefined') {
        setCSS(hovered, {
          color: '',
          backgroundColor: ''
        }).id = '';
      }
      if (e.type === 'mouseover') {
        status.acHovered = true;
        over = true;
      } else {
        over = false;
      }
      setCSS(target, {
        color: over ? '#FFFFFF' : '',
        backgroundColor: over ? 'rgb(66,139,202)' : ''
      }).id = over ? 'hover' : '';
    }
    function toggleAutoComplete(show) {
      var hidden = (elements.autocomplete.currentStyle ? elements.autocomplete.currentStyle.display : window.getComputedStyle(elements.autocomplete, null).display) === 'none';
      show = (typeof show === 'undefined') ? hidden : show;
      setCSS(elements.autocomplete, {display: (show) ? 'inherit' : 'none'});
      status.acActive = show;
      if (!show) { status.acHovered = false; }
    }
    function clickAutoComplete(e) {
      if (e.preventDefault) { e.preventDefault(); } else { e.returnValue = false; }
      var tgt = e.target || e.srcElement;
      elements.consoleinput.value = tgt.innerHTML;
      elements.consoleinput.focus();
      toggleAutoComplete();
    }
    function autoComplete(command) {
      if (command.length < 1) {
        toggleAutoComplete(false);
        return;
      }
      var searchString = encodeURI(command), matches, match, row, i, maxAmount = options.browserinfo.isMobile ? 3 : 5;
      elements.autocomplete.innerHTML = '';
      elements.acItems = [];
      matches = findInArray(history.input.commands, searchString);
      matches = matches.slice(Math.max(matches.length - maxAmount, 0));
      i = matches.length;
      while (i--) {
        match = decodeURI(matches[i]);
        row = elements.autocompleteItem.cloneNode(false);
        row.innerHTML = match;
        row.onmouseover = hoverAutoComplete;
        elements.autocomplete.insertBefore(row, elements.autocomplete.firstChild);
        elements.acItems.unshift(row);
      }
      toggleAutoComplete(matches.length > 0);
    }
    function setBinds() {
      if (options.browserinfo.evtLstn) {
        elements.autocomplete.addEventListener('click', clickAutoComplete, false);
      } else {
        elements.autocomplete.attachEvent('onclick', clickAutoComplete);
      }
      document.onkeydown = function (e) {
        e = e || window.event;
        var tgt = e.target || e.srcElement;
        if (tgt === elements.consoleinput) {
          if ((e.key === 'Enter' || e.keyCode === 13)) { //enter
            if (e.preventDefault) { e.preventDefault(); } else { e.returnValue = false; }
            if(!status.acHovered) {
              submitCommand(elements.consoleinput.value);
            } else {
              elements.consoleinput.value = getFromArrayById('hover').element.innerHTML;
              elements.consoleinput.focus();
            }
            toggleAutoComplete(false);
            status.acInput = '';
          } else if ((e.keyCode === 38 || e.keyCode === 40)) { //up and down arrows for history browsing
            if (e.preventDefault) { e.preventDefault(); } else { e.returnValue = false; }
            var up = (e.keyCode === 40);
            if(status.acActive) {
              //autocomplete window is opened
              //get id of currently hovered element
              var hovered = getFromArrayById('hover').position;
              var counter = (hovered === -1) ? elements.acItems.length : hovered;
              //hover new (in- or decreased number) one
              counter = valBetween((counter += (up) ? 1 : -1), 0, elements.acItems.length - 1);
              hoverAutoComplete({target : elements.acItems[counter], type : 'mouseover'});
            } else {
              //autocompete window not opened
              var hist = history.input.commands;
              history.input.commandIdx += (up) ? 1 : -1;
              history.input.commandIdx = valBetween(history.input.commandIdx, 0, hist.length);
              elements.consoleinput.value = typeof hist[history.input.commandIdx] === 'undefined' ? '' : decodeURI(hist[history.input.commandIdx]);
            }
          }
        }
        if (e.keyCode === 27 && status.acActive) {
          toggleAutoComplete(false);
        }
      };
      document.onkeyup = function (e) {
        e = e || window.event;
        var tgt = e.target || e.srcElement;
        if (tgt === elements.consoleinput && status.acInput !== elements.consoleinput.value && (e.keyCode !== 38 && e.keyCode !== 40 && e.keyCode !== 27 && e.key !== 'Enter' && e.keyCode !== 13)) {
          status.acInput = elements.consoleinput.value.trim();
          autoComplete(elements.consoleinput.value);
        }
      };
    }
    //init
    function init() {
      assemble();
      setBinds();
      return {
        //nothing  to expose
      };
    }
    //return
    if (!ref) {
      ref = init();
    }
    return ref;
  }

  function init() {
    if (!status.initialized) {
      if (consoleElement && mobileConsole) {
        console.error( 'Mobile Console cannot be reconstructed! Reload the page to enable Mobile Console again.' + '\n' +
          'Tip: use the minimize button instead of closing.' );
        return;
      } else {
        status.initialized = true;
        //populate references
        if (!mobileConsole) {
          //taps into native console and adds new functionality
          mobileConsole =   initConsole();
        }
        if (!consoleElement && mobileConsole) {
          //creates the new HTML console element and attaches it to document
          consoleElement =  initConsoleElement();
        }
        if (!commandLine && consoleElement && mobileConsole) {
          //creates an HTML commandline and attaches it to existing console element
          commandLine =   initCommandLine();
        }
      }
    }
    //log a 'welcome' message
    console.info( '--==## Mobile Console v' + options.version + ' ' + (status.initialized ? 'active' : 'inactive' ) + ' ##==--' );
  }

  //autorun if mobile
  if (options.browserinfo.isMobile || options.overrideAutorun) {
    init();
  }

  //expose the mobileConsole's methods
  return {
    init : init,
    about: about,
    toggle : toggleHeight,
    status : status,
    options : options
  };

}());