function ownKeys(e,t){var r=Object.keys(e);if(Object.getOwnPropertySymbols){var n=Object.getOwnPropertySymbols(e);t&&(n=n.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),r.push.apply(r,n)}return r}function _objectSpread(e){for(var t=1;t<arguments.length;t++){var r=null!=arguments[t]?arguments[t]:{};t%2?ownKeys(Object(r),!0).forEach((function(t){_defineProperty(e,t,r[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(r)):ownKeys(Object(r)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(r,t))}))}return e}function _defineProperty(e,t,r){return t in e?Object.defineProperty(e,t,{value:r,enumerable:!0,configurable:!0,writable:!0}):e[t]=r,e}function _classCallCheck(e,t){if(!(e instanceof t))throw new TypeError("Cannot call a class as a function")}function _defineProperties(e,t){for(var r=0;r<t.length;r++){var n=t[r];n.enumerable=n.enumerable||!1,n.configurable=!0,"value"in n&&(n.writable=!0),Object.defineProperty(e,n.key,n)}}function _createClass(e,t,r){return t&&_defineProperties(e.prototype,t),r&&_defineProperties(e,r),e}function asyncGeneratorStep(e,t,r,n,o,a,u){try{var i=e[a](u),s=i.value}catch(e){return void r(e)}i.done?t(s):Promise.resolve(s).then(n,o)}function _asyncToGenerator(e){return function(){var t=this,r=arguments;return new Promise((function(n,o){var a=e.apply(t,r);function u(e){asyncGeneratorStep(a,n,o,u,i,"next",e)}function i(e){asyncGeneratorStep(a,n,o,u,i,"throw",e)}u(void 0)}))}}function _typeof(e){return(_typeof="function"==typeof Symbol&&"symbol"==typeof Symbol.iterator?function(e){return typeof e}:function(e){return e&&"function"==typeof Symbol&&e.constructor===Symbol&&e!==Symbol.prototype?"symbol":typeof e})(e)}!function(e,t){"object"===("undefined"==typeof exports?"undefined":_typeof(exports))&&"undefined"!=typeof module?t(exports):"function"==typeof define&&define.amd?define(["exports"],t):t((e="undefined"!=typeof globalThis?globalThis:e||self).__TAURI__={})}(this,(function(e){"use strict";var t=function(e){var t,r=Object.prototype,n=r.hasOwnProperty,o="function"==typeof Symbol?Symbol:{},a=o.iterator||"@@iterator",u=o.asyncIterator||"@@asyncIterator",i=o.toStringTag||"@@toStringTag";function s(e,t,r){return Object.defineProperty(e,t,{value:r,enumerable:!0,configurable:!0,writable:!0}),e[t]}try{s({},"")}catch(e){s=function(e,t,r){return e[t]=r}}function c(e,t,r,n){var o=t&&t.prototype instanceof y?t:y,a=Object.create(o.prototype),u=new P(n||[]);return a._invoke=function(e,t,r){var n=f;return function(o,a){if(n===h)throw new Error("Generator is already running");if(n===m){if("throw"===o)throw a;return O()}for(r.method=o,r.arg=a;;){var u=r.delegate;if(u){var i=k(u,r);if(i){if(i===d)continue;return i}}if("next"===r.method)r.sent=r._sent=r.arg;else if("throw"===r.method){if(n===f)throw n=m,r.arg;r.dispatchException(r.arg)}else"return"===r.method&&r.abrupt("return",r.arg);n=h;var s=p(e,t,r);if("normal"===s.type){if(n=r.done?m:l,s.arg===d)continue;return{value:s.arg,done:r.done}}"throw"===s.type&&(n=m,r.method="throw",r.arg=s.arg)}}}(e,r,u),a}function p(e,t,r){try{return{type:"normal",arg:e.call(t,r)}}catch(e){return{type:"throw",arg:e}}}e.wrap=c;var f="suspendedStart",l="suspendedYield",h="executing",m="completed",d={};function y(){}function g(){}function _(){}var w={};w[a]=function(){return this};var v=Object.getPrototypeOf,x=v&&v(v(j([])));x&&x!==r&&n.call(x,a)&&(w=x);var b=_.prototype=y.prototype=Object.create(w);function R(e){["next","throw","return"].forEach((function(t){s(e,t,(function(e){return this._invoke(t,e)}))}))}function T(e,t){function r(o,a,u,i){var s=p(e[o],e,a);if("throw"!==s.type){var c=s.arg,f=c.value;return f&&"object"===_typeof(f)&&n.call(f,"__await")?t.resolve(f.__await).then((function(e){r("next",e,u,i)}),(function(e){r("throw",e,u,i)})):t.resolve(f).then((function(e){c.value=e,u(c)}),(function(e){return r("throw",e,u,i)}))}i(s.arg)}var o;this._invoke=function(e,n){function a(){return new t((function(t,o){r(e,n,t,o)}))}return o=o?o.then(a,a):a()}}function k(e,r){var n=e.iterator[r.method];if(n===t){if(r.delegate=null,"throw"===r.method){if(e.iterator.return&&(r.method="return",r.arg=t,k(e,r),"throw"===r.method))return d;r.method="throw",r.arg=new TypeError("The iterator does not provide a 'throw' method")}return d}var o=p(n,e.iterator,r.arg);if("throw"===o.type)return r.method="throw",r.arg=o.arg,r.delegate=null,d;var a=o.arg;return a?a.done?(r[e.resultName]=a.value,r.next=e.nextLoc,"return"!==r.method&&(r.method="next",r.arg=t),r.delegate=null,d):a:(r.method="throw",r.arg=new TypeError("iterator result is not an object"),r.delegate=null,d)}function G(e){var t={tryLoc:e[0]};1 in e&&(t.catchLoc=e[1]),2 in e&&(t.finallyLoc=e[2],t.afterLoc=e[3]),this.tryEntries.push(t)}function M(e){var t=e.completion||{};t.type="normal",delete t.arg,e.completion=t}function P(e){this.tryEntries=[{tryLoc:"root"}],e.forEach(G,this),this.reset(!0)}function j(e){if(e){var r=e[a];if(r)return r.call(e);if("function"==typeof e.next)return e;if(!isNaN(e.length)){var o=-1,u=function r(){for(;++o<e.length;)if(n.call(e,o))return r.value=e[o],r.done=!1,r;return r.value=t,r.done=!0,r};return u.next=u}}return{next:O}}function O(){return{value:t,done:!0}}return g.prototype=b.constructor=_,_.constructor=g,g.displayName=s(_,i,"GeneratorFunction"),e.isGeneratorFunction=function(e){var t="function"==typeof e&&e.constructor;return!!t&&(t===g||"GeneratorFunction"===(t.displayName||t.name))},e.mark=function(e){return Object.setPrototypeOf?Object.setPrototypeOf(e,_):(e.__proto__=_,s(e,i,"GeneratorFunction")),e.prototype=Object.create(b),e},e.awrap=function(e){return{__await:e}},R(T.prototype),T.prototype[u]=function(){return this},e.AsyncIterator=T,e.async=function(t,r,n,o,a){void 0===a&&(a=Promise);var u=new T(c(t,r,n,o),a);return e.isGeneratorFunction(r)?u:u.next().then((function(e){return e.done?e.value:u.next()}))},R(b),s(b,i,"Generator"),b[a]=function(){return this},b.toString=function(){return"[object Generator]"},e.keys=function(e){var t=[];for(var r in e)t.push(r);return t.reverse(),function r(){for(;t.length;){var n=t.pop();if(n in e)return r.value=n,r.done=!1,r}return r.done=!0,r}},e.values=j,P.prototype={constructor:P,reset:function(e){if(this.prev=0,this.next=0,this.sent=this._sent=t,this.done=!1,this.delegate=null,this.method="next",this.arg=t,this.tryEntries.forEach(M),!e)for(var r in this)"t"===r.charAt(0)&&n.call(this,r)&&!isNaN(+r.slice(1))&&(this[r]=t)},stop:function(){this.done=!0;var e=this.tryEntries[0].completion;if("throw"===e.type)throw e.arg;return this.rval},dispatchException:function(e){if(this.done)throw e;var r=this;function o(n,o){return i.type="throw",i.arg=e,r.next=n,o&&(r.method="next",r.arg=t),!!o}for(var a=this.tryEntries.length-1;a>=0;--a){var u=this.tryEntries[a],i=u.completion;if("root"===u.tryLoc)return o("end");if(u.tryLoc<=this.prev){var s=n.call(u,"catchLoc"),c=n.call(u,"finallyLoc");if(s&&c){if(this.prev<u.catchLoc)return o(u.catchLoc,!0);if(this.prev<u.finallyLoc)return o(u.finallyLoc)}else if(s){if(this.prev<u.catchLoc)return o(u.catchLoc,!0)}else{if(!c)throw new Error("try statement without catch or finally");if(this.prev<u.finallyLoc)return o(u.finallyLoc)}}}},abrupt:function(e,t){for(var r=this.tryEntries.length-1;r>=0;--r){var o=this.tryEntries[r];if(o.tryLoc<=this.prev&&n.call(o,"finallyLoc")&&this.prev<o.finallyLoc){var a=o;break}}a&&("break"===e||"continue"===e)&&a.tryLoc<=t&&t<=a.finallyLoc&&(a=null);var u=a?a.completion:{};return u.type=e,u.arg=t,a?(this.method="next",this.next=a.finallyLoc,d):this.complete(u)},complete:function(e,t){if("throw"===e.type)throw e.arg;return"break"===e.type||"continue"===e.type?this.next=e.arg:"return"===e.type?(this.rval=this.arg=e.arg,this.method="return",this.next="end"):"normal"===e.type&&t&&(this.next=t),d},finish:function(e){for(var t=this.tryEntries.length-1;t>=0;--t){var r=this.tryEntries[t];if(r.finallyLoc===e)return this.complete(r.completion,r.afterLoc),M(r),d}},catch:function(e){for(var t=this.tryEntries.length-1;t>=0;--t){var r=this.tryEntries[t];if(r.tryLoc===e){var n=r.completion;if("throw"===n.type){var o=n.arg;M(r)}return o}}throw new Error("illegal catch attempt")},delegateYield:function(e,r,n){return this.delegate={iterator:j(e),resultName:r,nextLoc:n},"next"===this.method&&(this.arg=t),d}},e}("object"===("undefined"==typeof module?"undefined":_typeof(module))?module.exports:{});try{regeneratorRuntime=t}catch(e){Function("r","regeneratorRuntime = r")(t)}function r(e){for(var t=void 0,r=e[0],n=1;n<e.length;){var o=e[n],a=e[n+1];if(n+=2,("optionalAccess"===o||"optionalCall"===o)&&null==r)return;"access"===o||"optionalAccess"===o?(t=r,r=a(r)):"call"!==o&&"optionalCall"!==o||(r=a((function(){for(var e,n=arguments.length,o=new Array(n),a=0;a<n;a++)o[a]=arguments[a];return(e=r).call.apply(e,[t].concat(o))})),t=void 0)}return r}function n(){return Math.floor(65536*(1+Math.random())).toString(16).substring(1)}function o(){return n()+n()+"-"+n()+"-"+n()+"-"+n()+"-"+n()+n()+n()}function a(e){var t=arguments.length>1&&void 0!==arguments[1]&&arguments[1],n=o();return Object.defineProperty(window,n,{value:function(o){return t&&Reflect.deleteProperty(window,n),r([e,"optionalCall",function(e){return e(o)}])},writable:!1,configurable:!0}),n}function u(e){return i.apply(this,arguments)}function i(){return(i=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,new Promise((function(e,r){var n=a((function(t){e(t),Reflect.deleteProperty(window,o)}),!0),o=a((function(e){r(e),Reflect.deleteProperty(window,n)}),!0);window.__TAURI_INVOKE_HANDLER__(JSON.stringify(_objectSpread({callback:n,error:o},t)))}));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var s=Object.freeze({__proto__:null,transformCallback:a,invoke:u});function c(){return(c=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Cli",message:{cmd:"cliMatches"}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var p=Object.freeze({__proto__:null,getMatches:function(){return c.apply(this,arguments)}});function f(){return(f=_asyncToGenerator(regeneratorRuntime.mark((function e(){var t,r=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(t=r.length>0&&void 0!==r[0]?r[0]:{})&&Object.freeze(t),e.next=4,u({__tauriModule:"Dialog",mainThread:!0,message:{cmd:"openDialog",options:t}});case 4:return e.abrupt("return",e.sent);case 5:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function l(){return(l=_asyncToGenerator(regeneratorRuntime.mark((function e(){var t,r=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(t=r.length>0&&void 0!==r[0]?r[0]:{})&&Object.freeze(t),e.next=4,u({__tauriModule:"Dialog",mainThread:!0,message:{cmd:"saveDialog",options:t}});case 4:return e.abrupt("return",e.sent);case 5:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var h=Object.freeze({__proto__:null,open:function(){return f.apply(this,arguments)},save:function(){return l.apply(this,arguments)}});function m(){return(m=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){var n,o=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return n=o.length>2&&void 0!==o[2]&&o[2],e.next=3,u({__tauriModule:"Event",message:{cmd:"listen",event:t,handler:a(r,n),once:n}});case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function d(){return(d=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Event",message:{cmd:"emit",event:t,payload:r}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var y,g=Object.freeze({__proto__:null,listen:function(e,t){return m.apply(this,arguments)},emit:function(e,t){return d.apply(this,arguments)}});function _(){return(_=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return r=n.length>1&&void 0!==n[1]?n[1]:{},e.next=3,u({__tauriModule:"Fs",message:{cmd:"readTextFile",path:t,options:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function w(){return(w=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return r=n.length>1&&void 0!==n[1]?n[1]:{},e.next=3,u({__tauriModule:"Fs",message:{cmd:"readBinaryFile",path:t,options:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function v(){return(v=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(r=n.length>1&&void 0!==n[1]?n[1]:{})&&Object.freeze(r),"object"===_typeof(t)&&Object.freeze(t),e.next=5,u({__tauriModule:"Fs",message:{cmd:"writeFile",path:t.path,contents:t.contents,options:r}});case 5:return e.abrupt("return",e.sent);case 6:case"end":return e.stop()}}),e)})))).apply(this,arguments)}!function(e){e[e.Audio=1]="Audio";e[e.Cache=2]="Cache";e[e.Config=3]="Config";e[e.Data=4]="Data";e[e.LocalData=5]="LocalData";e[e.Desktop=6]="Desktop";e[e.Document=7]="Document";e[e.Download=8]="Download";e[e.Executable=9]="Executable";e[e.Font=10]="Font";e[e.Home=11]="Home";e[e.Picture=12]="Picture";e[e.Public=13]="Public";e[e.Runtime=14]="Runtime";e[e.Template=15]="Template";e[e.Video=16]="Video";e[e.Resource=17]="Resource";e[e.App=18]="App"}(y||(y={}));var x=65536;function b(e){var t=function(e){if(e.length<x)return String.fromCharCode.apply(null,Array.from(e));for(var t="",r=e.length,n=0;n<r;n++){var o=e.subarray(n*x,(n+1)*x);t+=String.fromCharCode.apply(null,Array.from(o))}return t}(new Uint8Array(e));return btoa(t)}function R(){return(R=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(r=n.length>1&&void 0!==n[1]?n[1]:{})&&Object.freeze(r),"object"===_typeof(t)&&Object.freeze(t),e.next=5,u({__tauriModule:"Fs",message:{cmd:"writeBinaryFile",path:t.path,contents:b(t.contents),options:r}});case 5:return e.abrupt("return",e.sent);case 6:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function T(){return(T=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return r=n.length>1&&void 0!==n[1]?n[1]:{},e.next=3,u({__tauriModule:"Fs",message:{cmd:"readDir",path:t,options:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function k(){return(k=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return r=n.length>1&&void 0!==n[1]?n[1]:{},e.next=3,u({__tauriModule:"Fs",message:{cmd:"createDir",path:t,options:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function G(){return(G=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return r=n.length>1&&void 0!==n[1]?n[1]:{},e.next=3,u({__tauriModule:"Fs",message:{cmd:"removeDir",path:t,options:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function M(){return(M=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){var n,o=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return n=o.length>2&&void 0!==o[2]?o[2]:{},e.next=3,u({__tauriModule:"Fs",message:{cmd:"copyFile",source:t,destination:r,options:n}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function P(){return(P=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return r=n.length>1&&void 0!==n[1]?n[1]:{},e.next=3,u({__tauriModule:"Fs",message:{cmd:"removeFile",path:t,options:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function j(){return(j=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){var n,o=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return n=o.length>2&&void 0!==o[2]?o[2]:{},e.next=3,u({__tauriModule:"Fs",message:{cmd:"renameFile",oldPath:t,newPath:r,options:n}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var O=Object.freeze({__proto__:null,get BaseDirectory(){return y},get Dir(){return y},readTextFile:function(e){return _.apply(this,arguments)},readBinaryFile:function(e){return w.apply(this,arguments)},writeFile:function(e){return v.apply(this,arguments)},writeBinaryFile:function(e){return R.apply(this,arguments)},readDir:function(e){return T.apply(this,arguments)},createDir:function(e){return k.apply(this,arguments)},removeDir:function(e){return G.apply(this,arguments)},copyFile:function(e,t){return M.apply(this,arguments)},removeFile:function(e){return P.apply(this,arguments)},renameFile:function(e,t){return j.apply(this,arguments)}});function F(){return(F=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.App}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function D(){return(D=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Audio}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function S(){return(S=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Cache}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function E(){return(E=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Config}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function z(){return(z=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Data}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function L(){return(L=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Desktop}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function C(){return(C=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Document}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function W(){return(W=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Download}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function A(){return(A=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Executable}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function N(){return(N=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Font}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function H(){return(H=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Home}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function q(){return(q=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.LocalData}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function B(){return(B=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Picture}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function I(){return(I=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Public}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function J(){return(J=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Resource}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function K(){return(K=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Runtime}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function U(){return(U=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Template}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function V(){return(V=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:y.Video}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Y(){return(Y=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:t,directory:r}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var X,Q=Object.freeze({__proto__:null,appDir:function(){return F.apply(this,arguments)},audioDir:function(){return D.apply(this,arguments)},cacheDir:function(){return S.apply(this,arguments)},configDir:function(){return E.apply(this,arguments)},dataDir:function(){return z.apply(this,arguments)},desktopDir:function(){return L.apply(this,arguments)},documentDir:function(){return C.apply(this,arguments)},downloadDir:function(){return W.apply(this,arguments)},executableDir:function(){return A.apply(this,arguments)},fontDir:function(){return N.apply(this,arguments)},homeDir:function(){return H.apply(this,arguments)},localDataDir:function(){return q.apply(this,arguments)},pictureDir:function(){return B.apply(this,arguments)},publicDir:function(){return I.apply(this,arguments)},resourceDir:function(){return J.apply(this,arguments)},runtimeDir:function(){return K.apply(this,arguments)},templateDir:function(){return U.apply(this,arguments)},videoDir:function(){return V.apply(this,arguments)},resolvePath:function(e,t){return Y.apply(this,arguments)}});function Z(e,t){return null!=e?e:t()}function $(e){for(var t=void 0,r=e[0],n=1;n<e.length;){var o=e[n],a=e[n+1];if(n+=2,("optionalAccess"===o||"optionalCall"===o)&&null==r)return;"access"===o||"optionalAccess"===o?(t=r,r=a(r)):"call"!==o&&"optionalCall"!==o||(r=a((function(){for(var e,n=arguments.length,o=new Array(n),a=0;a<n;a++)o[a]=arguments[a];return(e=r).call.apply(e,[t].concat(o))})),t=void 0)}return r}!function(e){e[e.JSON=1]="JSON";e[e.Text=2]="Text";e[e.Binary=3]="Binary"}(X||(X={}));var ee=function(){function e(t,r){_classCallCheck(this,e),this.type=t,this.payload=r}return _createClass(e,null,[{key:"form",value:function(t){return new e("Form",t)}},{key:"json",value:function(t){return new e("Json",t)}},{key:"text",value:function(t){return new e("Text",t)}},{key:"bytes",value:function(t){return new e("Bytes",t)}}]),e}(),te=function(){function e(t){_classCallCheck(this,e),this.id=t}var t,r,n,o,a,i,s;return _createClass(e,[{key:"drop",value:(s=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Http",message:{cmd:"dropClient",client:this.id}});case 2:case"end":return e.stop()}}),e,this)}))),function(){return s.apply(this,arguments)})},{key:"request",value:(i=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Http",message:{cmd:"httpRequest",client:this.id,options:t}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e,this)}))),function(e){return i.apply(this,arguments)})},{key:"get",value:(a=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,this.request(_objectSpread({method:"GET",url:t},r));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e,this)}))),function(e,t){return a.apply(this,arguments)})},{key:"post",value:(o=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r,n){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,this.request(_objectSpread({method:"POST",url:t,body:r},n));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e,this)}))),function(e,t,r){return o.apply(this,arguments)})},{key:"put",value:(n=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r,n){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,this.request(_objectSpread({method:"PUT",url:t,body:r},n));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e,this)}))),function(e,t,r){return n.apply(this,arguments)})},{key:"patch",value:(r=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,this.request(_objectSpread({method:"PATCH",url:t},r));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e,this)}))),function(e,t){return r.apply(this,arguments)})},{key:"delete",value:(t=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,this.request(_objectSpread({method:"DELETE",url:t},r));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e,this)}))),function(e,r){return t.apply(this,arguments)})}]),e}();function re(e){return ne.apply(this,arguments)}function ne(){return(ne=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Http",message:{cmd:"createClient",options:t}}).then((function(e){return new te(e)}));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var oe=null;function ae(){return(ae=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:if(null!==oe){e.next=4;break}return e.next=3,re();case 3:oe=e.sent;case 4:return e.next=6,oe.request(_objectSpread({url:t,method:Z($([r,"optionalAccess",function(e){return e.method}]),(function(){return"GET"}))},r));case 6:return e.abrupt("return",e.sent);case 7:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var ue=Object.freeze({__proto__:null,get ResponseType(){return X},Body:ee,Client:te,getClient:re,fetch:function(e,t){return ae.apply(this,arguments)}});function ie(){return(ie=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(r)&&Object.freeze(r),e.next=3,u({__tauriModule:"Shell",message:{cmd:"execute",command:t,args:"string"==typeof r?[r]:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function se(){return(se=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Shell",message:{cmd:"open",uri:t}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var ce=Object.freeze({__proto__:null,execute:function(e,t){return ie.apply(this,arguments)},open:function(e){return se.apply(this,arguments)}});function pe(){return(pe=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setResizable",resizable:t}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function fe(){return(fe=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setTitle",title:t}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function le(){return(le=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"maximize"}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function he(){return(he=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"unmaximize"}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function me(){return(me=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"minimize"}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function de(){return(de=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"unminimize"}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function ye(){return(ye=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"show"}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function ge(){return(ge=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"hide"}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function _e(){return(_e=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setTransparent",transparent:t}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function we(){return(we=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setDecorations",decorations:t}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function ve(){return(ve=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setAlwaysOnTop",alwaysOnTop:t}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function xe(){return(xe=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setWidth",width:t}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function be(){return(be=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setHeight",height:t}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Re(){return(Re=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"resize",width:t,height:r}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Te(){return(Te=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setMinSize",minWidth:t,minHeight:r}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function ke(){return(ke=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setMaxSize",maxWidth:t,maxHeight:r}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Ge(){return(Ge=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setX",x:t}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Me(){return(Me=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setY",y:t}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Pe(){return(Pe=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setPosition",x:t,y:r}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function je(){return(je=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setFullscreen",fullscreen:t}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Oe(){return(Oe=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Window",message:{cmd:"setIcon",icon:t}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var Fe=Object.freeze({__proto__:null,setResizable:function(e){return pe.apply(this,arguments)},setTitle:function(e){return fe.apply(this,arguments)},maximize:function(){return le.apply(this,arguments)},unmaximize:function(){return he.apply(this,arguments)},minimize:function(){return me.apply(this,arguments)},unminimize:function(){return de.apply(this,arguments)},show:function(){return ye.apply(this,arguments)},hide:function(){return ge.apply(this,arguments)},setTransparent:function(e){return _e.apply(this,arguments)},setDecorations:function(e){return we.apply(this,arguments)},setAlwaysOnTop:function(e){return ve.apply(this,arguments)},setWidth:function(e){return xe.apply(this,arguments)},setHeight:function(e){return be.apply(this,arguments)},resize:function(e,t){return Re.apply(this,arguments)},setMinSize:function(e,t){return Te.apply(this,arguments)},setMaxSize:function(e,t){return ke.apply(this,arguments)},setX:function(e){return Ge.apply(this,arguments)},setY:function(e){return Me.apply(this,arguments)},setPosition:function(e,t){return Pe.apply(this,arguments)},setFullscreen:function(e){return je.apply(this,arguments)},setIcon:function(e){return Oe.apply(this,arguments)}});function De(){return(De=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:if("default"===window.Notification.permission){e.next=4;break}return e.next=3,Promise.resolve("granted"===window.Notification.permission);case 3:return e.abrupt("return",e.sent);case 4:return e.next=6,u({__tauriModule:"Notification",message:{cmd:"isNotificationPermissionGranted"}});case 6:return e.abrupt("return",e.sent);case 7:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Se(){return(Se=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,window.Notification.requestPermission();case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var Ee=Object.freeze({__proto__:null,sendNotification:function(e){"string"==typeof e?new window.Notification(e):new window.Notification(e.title,e)},requestPermission:function(){return Se.apply(this,arguments)},isPermissionGranted:function(){return De.apply(this,arguments)}});function ze(){return(ze=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"GlobalShortcut",message:{cmd:"register",shortcut:t,handler:a(r)}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Le(){return(Le=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"GlobalShortcut",message:{cmd:"unregister",shortcut:t}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var Ce=Object.freeze({__proto__:null,registerShortcut:function(e,t){return ze.apply(this,arguments)},unregisterShortcut:function(e){return Le.apply(this,arguments)}});e.cli=p,e.dialog=h,e.event=g,e.fs=O,e.globalShortcut=Ce,e.http=ue,e.notification=Ee,e.path=Q,e.shell=ce,e.tauri=s,e.window=Fe,Object.defineProperty(e,"__esModule",{value:!0})}));


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
      "-" +
      s4() +
      "-" +
      s4() +
      "-" +
      s4() +
      "-" +
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

  window.__TAURI__.invoke = function invoke(args) {
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

      if (window.__TAURI_INVOKE_HANDLER__) {
        window.__TAURI_INVOKE_HANDLER__(
          JSON.stringify(
            _objectSpread(
              {
                callback: callback,
                error: error,
              },
              args
            )
          )
        );
      } else {
        window.addEventListener("DOMContentLoaded", function () {
          window.__TAURI_INVOKE_HANDLER__(
            JSON.stringify(
              _objectSpread(
                {
                  callback: callback,
                  error: error,
                },
                args
              )
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
              window.__TAURI__.invoke({
                __tauriModule: "Shell",
                message: {
                  cmd: "open",
                  uri: target.href,
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

  let permissionSettable = false;
  let permissionValue = "default";

  function isPermissionGranted() {
    if (window.Notification.permission !== "default") {
      return Promise.resolve(window.Notification.permission === "granted");
    }
    return window.__TAURI__.invoke({
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
      .invoke({
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
        return window.__TAURI__.invoke({
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
    window.__TAURI__.invoke({
      __tauriModule: "Dialog",
      mainThread: true,
      message: {
        cmd: "messageDialog",
        message: message,
      },
    });
  };

  window.confirm = function (message) {
    return window.__TAURI__.invoke({
      __tauriModule: "Dialog",
      mainThread: true,
      message: {
        cmd: "askDialog",
        message: message,
      },
    });
  };
})();
