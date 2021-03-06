function _inherits(e,r){if("function"!=typeof r&&null!==r)throw new TypeError("Super expression must either be null or a function");e.prototype=Object.create(r&&r.prototype,{constructor:{value:e,writable:!0,configurable:!0}}),r&&_setPrototypeOf(e,r)}function _setPrototypeOf(e,r){return(_setPrototypeOf=Object.setPrototypeOf||function(e,r){return e.__proto__=r,e})(e,r)}function _createSuper(e){var r=_isNativeReflectConstruct();return function(){var t,n=_getPrototypeOf(e);if(r){var o=_getPrototypeOf(this).constructor;t=Reflect.construct(n,arguments,o)}else t=n.apply(this,arguments);return _possibleConstructorReturn(this,t)}}function _possibleConstructorReturn(e,r){return!r||"object"!==_typeof(r)&&"function"!=typeof r?_assertThisInitialized(e):r}function _assertThisInitialized(e){if(void 0===e)throw new ReferenceError("this hasn't been initialised - super() hasn't been called");return e}function _isNativeReflectConstruct(){if("undefined"==typeof Reflect||!Reflect.construct)return!1;if(Reflect.construct.sham)return!1;if("function"==typeof Proxy)return!0;try{return Date.prototype.toString.call(Reflect.construct(Date,[],(function(){}))),!0}catch(e){return!1}}function _getPrototypeOf(e){return(_getPrototypeOf=Object.setPrototypeOf?Object.getPrototypeOf:function(e){return e.__proto__||Object.getPrototypeOf(e)})(e)}function _createForOfIteratorHelper(e,r){var t;if("undefined"==typeof Symbol||null==e[Symbol.iterator]){if(Array.isArray(e)||(t=_unsupportedIterableToArray(e))||r&&e&&"number"==typeof e.length){t&&(e=t);var n=0,o=function(){};return{s:o,n:function(){return n>=e.length?{done:!0}:{done:!1,value:e[n++]}},e:function(e){throw e},f:o}}throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method.")}var a,u=!0,i=!1;return{s:function(){t=e[Symbol.iterator]()},n:function(){var e=t.next();return u=e.done,e},e:function(e){i=!0,a=e},f:function(){try{u||null==t.return||t.return()}finally{if(i)throw a}}}}function _unsupportedIterableToArray(e,r){if(e){if("string"==typeof e)return _arrayLikeToArray(e,r);var t=Object.prototype.toString.call(e).slice(8,-1);return"Object"===t&&e.constructor&&(t=e.constructor.name),"Map"===t||"Set"===t?Array.from(e):"Arguments"===t||/^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(t)?_arrayLikeToArray(e,r):void 0}}function _arrayLikeToArray(e,r){(null==r||r>e.length)&&(r=e.length);for(var t=0,n=new Array(r);t<r;t++)n[t]=e[t];return n}function ownKeys(e,r){var t=Object.keys(e);if(Object.getOwnPropertySymbols){var n=Object.getOwnPropertySymbols(e);r&&(n=n.filter((function(r){return Object.getOwnPropertyDescriptor(e,r).enumerable}))),t.push.apply(t,n)}return t}function _objectSpread(e){for(var r=1;r<arguments.length;r++){var t=null!=arguments[r]?arguments[r]:{};r%2?ownKeys(Object(t),!0).forEach((function(r){_defineProperty(e,r,t[r])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(t)):ownKeys(Object(t)).forEach((function(r){Object.defineProperty(e,r,Object.getOwnPropertyDescriptor(t,r))}))}return e}function _defineProperty(e,r,t){return r in e?Object.defineProperty(e,r,{value:t,enumerable:!0,configurable:!0,writable:!0}):e[r]=t,e}function _classCallCheck(e,r){if(!(e instanceof r))throw new TypeError("Cannot call a class as a function")}function _defineProperties(e,r){for(var t=0;t<r.length;t++){var n=r[t];n.enumerable=n.enumerable||!1,n.configurable=!0,"value"in n&&(n.writable=!0),Object.defineProperty(e,n.key,n)}}function _createClass(e,r,t){return r&&_defineProperties(e.prototype,r),t&&_defineProperties(e,t),e}function asyncGeneratorStep(e,r,t,n,o,a,u){try{var i=e[a](u),c=i.value}catch(e){return void t(e)}i.done?r(c):Promise.resolve(c).then(n,o)}function _asyncToGenerator(e){return function(){var r=this,t=arguments;return new Promise((function(n,o){var a=e.apply(r,t);function u(e){asyncGeneratorStep(a,n,o,u,i,"next",e)}function i(e){asyncGeneratorStep(a,n,o,u,i,"throw",e)}u(void 0)}))}}function _typeof(e){return(_typeof="function"==typeof Symbol&&"symbol"==typeof Symbol.iterator?function(e){return typeof e}:function(e){return e&&"function"==typeof Symbol&&e.constructor===Symbol&&e!==Symbol.prototype?"symbol":typeof e})(e)}!function(e,r){"object"===("undefined"==typeof exports?"undefined":_typeof(exports))&&"undefined"!=typeof module?r(exports):"function"==typeof define&&define.amd?define(["exports"],r):r((e="undefined"!=typeof globalThis?globalThis:e||self).__TAURI__={})}(this,(function(e){"use strict";var r=function(e){var r,t=Object.prototype,n=t.hasOwnProperty,o="function"==typeof Symbol?Symbol:{},a=o.iterator||"@@iterator",u=o.asyncIterator||"@@asyncIterator",i=o.toStringTag||"@@toStringTag";function c(e,r,t){return Object.defineProperty(e,r,{value:t,enumerable:!0,configurable:!0,writable:!0}),e[r]}try{c({},"")}catch(e){c=function(e,r,t){return e[r]=t}}function s(e,r,t,n){var o=r&&r.prototype instanceof d?r:d,a=Object.create(o.prototype),u=new O(n||[]);return a._invoke=function(e,r,t){var n=f;return function(o,a){if(n===h)throw new Error("Generator is already running");if(n===m){if("throw"===o)throw a;return j()}for(t.method=o,t.arg=a;;){var u=t.delegate;if(u){var i=T(u,t);if(i){if(i===y)continue;return i}}if("next"===t.method)t.sent=t._sent=t.arg;else if("throw"===t.method){if(n===f)throw n=m,t.arg;t.dispatchException(t.arg)}else"return"===t.method&&t.abrupt("return",t.arg);n=h;var c=p(e,r,t);if("normal"===c.type){if(n=t.done?m:l,c.arg===y)continue;return{value:c.arg,done:t.done}}"throw"===c.type&&(n=m,t.method="throw",t.arg=c.arg)}}}(e,t,u),a}function p(e,r,t){try{return{type:"normal",arg:e.call(r,t)}}catch(e){return{type:"throw",arg:e}}}e.wrap=s;var f="suspendedStart",l="suspendedYield",h="executing",m="completed",y={};function d(){}function g(){}function _(){}var v={};v[a]=function(){return this};var w=Object.getPrototypeOf,b=w&&w(w(M([])));b&&b!==t&&n.call(b,a)&&(v=b);var R=_.prototype=d.prototype=Object.create(v);function k(e){["next","throw","return"].forEach((function(r){c(e,r,(function(e){return this._invoke(r,e)}))}))}function x(e,r){function t(o,a,u,i){var c=p(e[o],e,a);if("throw"!==c.type){var s=c.arg,f=s.value;return f&&"object"===_typeof(f)&&n.call(f,"__await")?r.resolve(f.__await).then((function(e){t("next",e,u,i)}),(function(e){t("throw",e,u,i)})):r.resolve(f).then((function(e){s.value=e,u(s)}),(function(e){return t("throw",e,u,i)}))}i(c.arg)}var o;this._invoke=function(e,n){function a(){return new r((function(r,o){t(e,n,r,o)}))}return o=o?o.then(a,a):a()}}function T(e,t){var n=e.iterator[t.method];if(n===r){if(t.delegate=null,"throw"===t.method){if(e.iterator.return&&(t.method="return",t.arg=r,T(e,t),"throw"===t.method))return y;t.method="throw",t.arg=new TypeError("The iterator does not provide a 'throw' method")}return y}var o=p(n,e.iterator,t.arg);if("throw"===o.type)return t.method="throw",t.arg=o.arg,t.delegate=null,y;var a=o.arg;return a?a.done?(t[e.resultName]=a.value,t.next=e.nextLoc,"return"!==t.method&&(t.method="next",t.arg=r),t.delegate=null,y):a:(t.method="throw",t.arg=new TypeError("iterator result is not an object"),t.delegate=null,y)}function G(e){var r={tryLoc:e[0]};1 in e&&(r.catchLoc=e[1]),2 in e&&(r.finallyLoc=e[2],r.afterLoc=e[3]),this.tryEntries.push(r)}function P(e){var r=e.completion||{};r.type="normal",delete r.arg,e.completion=r}function O(e){this.tryEntries=[{tryLoc:"root"}],e.forEach(G,this),this.reset(!0)}function M(e){if(e){var t=e[a];if(t)return t.call(e);if("function"==typeof e.next)return e;if(!isNaN(e.length)){var o=-1,u=function t(){for(;++o<e.length;)if(n.call(e,o))return t.value=e[o],t.done=!1,t;return t.value=r,t.done=!0,t};return u.next=u}}return{next:j}}function j(){return{value:r,done:!0}}return g.prototype=R.constructor=_,_.constructor=g,g.displayName=c(_,i,"GeneratorFunction"),e.isGeneratorFunction=function(e){var r="function"==typeof e&&e.constructor;return!!r&&(r===g||"GeneratorFunction"===(r.displayName||r.name))},e.mark=function(e){return Object.setPrototypeOf?Object.setPrototypeOf(e,_):(e.__proto__=_,c(e,i,"GeneratorFunction")),e.prototype=Object.create(R),e},e.awrap=function(e){return{__await:e}},k(x.prototype),x.prototype[u]=function(){return this},e.AsyncIterator=x,e.async=function(r,t,n,o,a){void 0===a&&(a=Promise);var u=new x(s(r,t,n,o),a);return e.isGeneratorFunction(t)?u:u.next().then((function(e){return e.done?e.value:u.next()}))},k(R),c(R,i,"Generator"),R[a]=function(){return this},R.toString=function(){return"[object Generator]"},e.keys=function(e){var r=[];for(var t in e)r.push(t);return r.reverse(),function t(){for(;r.length;){var n=r.pop();if(n in e)return t.value=n,t.done=!1,t}return t.done=!0,t}},e.values=M,O.prototype={constructor:O,reset:function(e){if(this.prev=0,this.next=0,this.sent=this._sent=r,this.done=!1,this.delegate=null,this.method="next",this.arg=r,this.tryEntries.forEach(P),!e)for(var t in this)"t"===t.charAt(0)&&n.call(this,t)&&!isNaN(+t.slice(1))&&(this[t]=r)},stop:function(){this.done=!0;var e=this.tryEntries[0].completion;if("throw"===e.type)throw e.arg;return this.rval},dispatchException:function(e){if(this.done)throw e;var t=this;function o(n,o){return i.type="throw",i.arg=e,t.next=n,o&&(t.method="next",t.arg=r),!!o}for(var a=this.tryEntries.length-1;a>=0;--a){var u=this.tryEntries[a],i=u.completion;if("root"===u.tryLoc)return o("end");if(u.tryLoc<=this.prev){var c=n.call(u,"catchLoc"),s=n.call(u,"finallyLoc");if(c&&s){if(this.prev<u.catchLoc)return o(u.catchLoc,!0);if(this.prev<u.finallyLoc)return o(u.finallyLoc)}else if(c){if(this.prev<u.catchLoc)return o(u.catchLoc,!0)}else{if(!s)throw new Error("try statement without catch or finally");if(this.prev<u.finallyLoc)return o(u.finallyLoc)}}}},abrupt:function(e,r){for(var t=this.tryEntries.length-1;t>=0;--t){var o=this.tryEntries[t];if(o.tryLoc<=this.prev&&n.call(o,"finallyLoc")&&this.prev<o.finallyLoc){var a=o;break}}a&&("break"===e||"continue"===e)&&a.tryLoc<=r&&r<=a.finallyLoc&&(a=null);var u=a?a.completion:{};return u.type=e,u.arg=r,a?(this.method="next",this.next=a.finallyLoc,y):this.complete(u)},complete:function(e,r){if("throw"===e.type)throw e.arg;return"break"===e.type||"continue"===e.type?this.next=e.arg:"return"===e.type?(this.rval=this.arg=e.arg,this.method="return",this.next="end"):"normal"===e.type&&r&&(this.next=r),y},finish:function(e){for(var r=this.tryEntries.length-1;r>=0;--r){var t=this.tryEntries[r];if(t.finallyLoc===e)return this.complete(t.completion,t.afterLoc),P(t),y}},catch:function(e){for(var r=this.tryEntries.length-1;r>=0;--r){var t=this.tryEntries[r];if(t.tryLoc===e){var n=t.completion;if("throw"===n.type){var o=n.arg;P(t)}return o}}throw new Error("illegal catch attempt")},delegateYield:function(e,t,n){return this.delegate={iterator:M(e),resultName:t,nextLoc:n},"next"===this.method&&(this.arg=r),y}},e}("object"===("undefined"==typeof module?"undefined":_typeof(module))?module.exports:{});try{regeneratorRuntime=r}catch(e){Function("r","regeneratorRuntime = r")(r)}function t(e){for(var r=void 0,t=e[0],n=1;n<e.length;){var o=e[n],a=e[n+1];if(n+=2,("optionalAccess"===o||"optionalCall"===o)&&null==t)return;"access"===o||"optionalAccess"===o?(r=t,t=a(t)):"call"!==o&&"optionalCall"!==o||(t=a((function(){for(var e,n=arguments.length,o=new Array(n),a=0;a<n;a++)o[a]=arguments[a];return(e=t).call.apply(e,[r].concat(o))})),r=void 0)}return t}function n(){return Math.floor(65536*(1+Math.random())).toString(16).substring(1)}function o(){return n()+n()+"-"+n()+"-"+n()+"-"+n()+"-"+n()+n()+n()}function a(e){var r=arguments.length>1&&void 0!==arguments[1]&&arguments[1],n=o();return Object.defineProperty(window,n,{value:function(o){return r&&Reflect.deleteProperty(window,n),t([e,"optionalCall",function(e){return e(o)}])},writable:!1,configurable:!0}),n}function u(e){return i.apply(this,arguments)}function i(){return(i=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",new Promise((function(e,t){var n=a((function(r){e(r),Reflect.deleteProperty(window,o)}),!0),o=a((function(e){t(e),Reflect.deleteProperty(window,n)}),!0);window.__TAURI_INVOKE_HANDLER__(_objectSpread({callback:n,error:o},r))})));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var c=Object.freeze({__proto__:null,transformCallback:a,invoke:u});function s(){return(s=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Cli",message:{cmd:"cliMatches"}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var p=Object.freeze({__proto__:null,getMatches:function(){return s.apply(this,arguments)}});function f(){return(f=_asyncToGenerator(regeneratorRuntime.mark((function e(){var r,t=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(r=t.length>0&&void 0!==t[0]?t[0]:{})&&Object.freeze(r),e.abrupt("return",u({__tauriModule:"Dialog",mainThread:!0,message:{cmd:"openDialog",options:r}}));case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function l(){return(l=_asyncToGenerator(regeneratorRuntime.mark((function e(){var r,t=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(r=t.length>0&&void 0!==t[0]?t[0]:{})&&Object.freeze(r),e.abrupt("return",u({__tauriModule:"Dialog",mainThread:!0,message:{cmd:"saveDialog",options:r}}));case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var h=Object.freeze({__proto__:null,open:function(){return f.apply(this,arguments)},save:function(){return l.apply(this,arguments)}});function m(e,r,t){return y.apply(this,arguments)}function y(){return(y=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t,n){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Event",message:{cmd:"listen",event:r,handler:a(t,n),once:n}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function d(e,r){return g.apply(this,arguments)}function g(){return(g=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",m(r,t,!1));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function _(e,r){return v.apply(this,arguments)}function v(){return(v=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",m(r,t,!0));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function w(e,r,t){return b.apply(this,arguments)}function b(){return(b=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t,n){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({__tauriModule:"Event",message:{cmd:"emit",event:r,windowLabel:t,payload:n}});case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function R(){return(R=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",w(r,void 0,t));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var k,x=Object.freeze({__proto__:null,emit:function(e,r){return R.apply(this,arguments)},listen:d,once:_});function T(){return(T=_asyncToGenerator(regeneratorRuntime.mark((function e(r){var t,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return t=n.length>1&&void 0!==n[1]?n[1]:{},e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"readTextFile",path:r,options:t}}));case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function G(){return(G=_asyncToGenerator(regeneratorRuntime.mark((function e(r){var t,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return t=n.length>1&&void 0!==n[1]?n[1]:{},e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"readBinaryFile",path:r,options:t}}));case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function P(){return(P=_asyncToGenerator(regeneratorRuntime.mark((function e(r){var t,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(t=n.length>1&&void 0!==n[1]?n[1]:{})&&Object.freeze(t),"object"===_typeof(r)&&Object.freeze(r),e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"writeFile",path:r.path,contents:r.contents,options:t}}));case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}!function(e){e[e.Audio=1]="Audio";e[e.Cache=2]="Cache";e[e.Config=3]="Config";e[e.Data=4]="Data";e[e.LocalData=5]="LocalData";e[e.Desktop=6]="Desktop";e[e.Document=7]="Document";e[e.Download=8]="Download";e[e.Executable=9]="Executable";e[e.Font=10]="Font";e[e.Home=11]="Home";e[e.Picture=12]="Picture";e[e.Public=13]="Public";e[e.Runtime=14]="Runtime";e[e.Template=15]="Template";e[e.Video=16]="Video";e[e.Resource=17]="Resource";e[e.App=18]="App"}(k||(k={}));var O=65536;function M(e){var r=function(e){if(e.length<O)return String.fromCharCode.apply(null,Array.from(e));for(var r="",t=e.length,n=0;n<t;n++){var o=e.subarray(n*O,(n+1)*O);r+=String.fromCharCode.apply(null,Array.from(o))}return r}(new Uint8Array(e));return btoa(r)}function j(){return(j=_asyncToGenerator(regeneratorRuntime.mark((function e(r){var t,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(t=n.length>1&&void 0!==n[1]?n[1]:{})&&Object.freeze(t),"object"===_typeof(r)&&Object.freeze(r),e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"writeBinaryFile",path:r.path,contents:M(r.contents),options:t}}));case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function F(){return(F=_asyncToGenerator(regeneratorRuntime.mark((function e(r){var t,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return t=n.length>1&&void 0!==n[1]?n[1]:{},e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"readDir",path:r,options:t}}));case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function D(){return(D=_asyncToGenerator(regeneratorRuntime.mark((function e(r){var t,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return t=n.length>1&&void 0!==n[1]?n[1]:{},e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"createDir",path:r,options:t}}));case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function S(){return(S=_asyncToGenerator(regeneratorRuntime.mark((function e(r){var t,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return t=n.length>1&&void 0!==n[1]?n[1]:{},e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"removeDir",path:r,options:t}}));case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function C(){return(C=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){var n,o=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return n=o.length>2&&void 0!==o[2]?o[2]:{},e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"copyFile",source:r,destination:t,options:n}}));case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function E(){return(E=_asyncToGenerator(regeneratorRuntime.mark((function e(r){var t,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return t=n.length>1&&void 0!==n[1]?n[1]:{},e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"removeFile",path:r,options:t}}));case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function A(){return(A=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){var n,o=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return n=o.length>2&&void 0!==o[2]?o[2]:{},e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"renameFile",oldPath:r,newPath:t,options:n}}));case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var L=Object.freeze({__proto__:null,get BaseDirectory(){return k},get Dir(){return k},readTextFile:function(e){return T.apply(this,arguments)},readBinaryFile:function(e){return G.apply(this,arguments)},writeFile:function(e){return P.apply(this,arguments)},writeBinaryFile:function(e){return j.apply(this,arguments)},readDir:function(e){return F.apply(this,arguments)},createDir:function(e){return D.apply(this,arguments)},removeDir:function(e){return S.apply(this,arguments)},copyFile:function(e,r){return C.apply(this,arguments)},removeFile:function(e){return E.apply(this,arguments)},renameFile:function(e,r){return A.apply(this,arguments)}});function z(){return(z=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.App}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function W(){return(W=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Audio}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function N(){return(N=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Cache}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function I(){return(I=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Config}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function H(){return(H=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Data}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function q(){return(q=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Desktop}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function B(){return(B=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Document}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function U(){return(U=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Download}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function K(){return(K=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Executable}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function V(){return(V=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Font}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Y(){return(Y=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Home}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function J(){return(J=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.LocalData}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function X(){return(X=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Picture}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function $(){return($=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Public}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Q(){return(Q=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Resource}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Z(){return(Z=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Runtime}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function ee(){return(ee=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Template}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function re(){return(re=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:"",directory:k.Video}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function te(){return(te=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Fs",message:{cmd:"resolvePath",path:r,directory:t}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var ne,oe=Object.freeze({__proto__:null,appDir:function(){return z.apply(this,arguments)},audioDir:function(){return W.apply(this,arguments)},cacheDir:function(){return N.apply(this,arguments)},configDir:function(){return I.apply(this,arguments)},dataDir:function(){return H.apply(this,arguments)},desktopDir:function(){return q.apply(this,arguments)},documentDir:function(){return B.apply(this,arguments)},downloadDir:function(){return U.apply(this,arguments)},executableDir:function(){return K.apply(this,arguments)},fontDir:function(){return V.apply(this,arguments)},homeDir:function(){return Y.apply(this,arguments)},localDataDir:function(){return J.apply(this,arguments)},pictureDir:function(){return X.apply(this,arguments)},publicDir:function(){return $.apply(this,arguments)},resourceDir:function(){return Q.apply(this,arguments)},runtimeDir:function(){return Z.apply(this,arguments)},templateDir:function(){return ee.apply(this,arguments)},videoDir:function(){return re.apply(this,arguments)},resolvePath:function(e,r){return te.apply(this,arguments)}});function ae(e,r){return null!=e?e:r()}function ue(e){for(var r=void 0,t=e[0],n=1;n<e.length;){var o=e[n],a=e[n+1];if(n+=2,("optionalAccess"===o||"optionalCall"===o)&&null==t)return;"access"===o||"optionalAccess"===o?(r=t,t=a(t)):"call"!==o&&"optionalCall"!==o||(t=a((function(){for(var e,n=arguments.length,o=new Array(n),a=0;a<n;a++)o[a]=arguments[a];return(e=t).call.apply(e,[r].concat(o))})),r=void 0)}return t}!function(e){e[e.JSON=1]="JSON";e[e.Text=2]="Text";e[e.Binary=3]="Binary"}(ne||(ne={}));var ie=function(){function e(r,t){_classCallCheck(this,e),this.type=r,this.payload=t}return _createClass(e,null,[{key:"form",value:function(r){return new e("Form",r)}},{key:"json",value:function(r){return new e("Json",r)}},{key:"text",value:function(r){return new e("Text",r)}},{key:"bytes",value:function(r){return new e("Bytes",r)}}]),e}(),ce=function(){function e(r){_classCallCheck(this,e),this.id=r}var r,t,n,o,a,i,c;return _createClass(e,[{key:"drop",value:(c=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Http",message:{cmd:"dropClient",client:this.id}}));case 1:case"end":return e.stop()}}),e,this)}))),function(){return c.apply(this,arguments)})},{key:"request",value:(i=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Http",message:{cmd:"httpRequest",client:this.id,options:r}}));case 1:case"end":return e.stop()}}),e,this)}))),function(e){return i.apply(this,arguments)})},{key:"get",value:(a=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",this.request(_objectSpread({method:"GET",url:r},t)));case 1:case"end":return e.stop()}}),e,this)}))),function(e,r){return a.apply(this,arguments)})},{key:"post",value:(o=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t,n){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",this.request(_objectSpread({method:"POST",url:r,body:t},n)));case 1:case"end":return e.stop()}}),e,this)}))),function(e,r,t){return o.apply(this,arguments)})},{key:"put",value:(n=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t,n){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",this.request(_objectSpread({method:"PUT",url:r,body:t},n)));case 1:case"end":return e.stop()}}),e,this)}))),function(e,r,t){return n.apply(this,arguments)})},{key:"patch",value:(t=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",this.request(_objectSpread({method:"PATCH",url:r},t)));case 1:case"end":return e.stop()}}),e,this)}))),function(e,r){return t.apply(this,arguments)})},{key:"delete",value:(r=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",this.request(_objectSpread({method:"DELETE",url:r},t)));case 1:case"end":return e.stop()}}),e,this)}))),function(e,t){return r.apply(this,arguments)})}]),e}();function se(e){return pe.apply(this,arguments)}function pe(){return(pe=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Http",message:{cmd:"createClient",options:r}}).then((function(e){return new ce(e)})));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var fe=null;function le(){return(le=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:if(null!==fe){e.next=4;break}return e.next=3,se();case 3:fe=e.sent;case 4:return e.abrupt("return",fe.request(_objectSpread({url:r,method:ae(ue([t,"optionalAccess",function(e){return e.method}]),(function(){return"GET"}))},t)));case 5:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var he=Object.freeze({__proto__:null,get ResponseType(){return ne},Body:ie,Client:ce,getClient:se,fetch:function(e,r){return le.apply(this,arguments)}});function me(){return(me=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(t)&&Object.freeze(t),e.abrupt("return",u({__tauriModule:"Shell",message:{cmd:"execute",command:r,args:"string"==typeof t?[t]:t}}));case 2:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function ye(){return(ye=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Shell",message:{cmd:"open",uri:r}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var de=Object.freeze({__proto__:null,execute:function(e,r){return me.apply(this,arguments)},open:function(e){return ye.apply(this,arguments)}});function ge(){return window.__TAURI__.__windows}var _e=["tauri://created","tauri://error"],ve=function(){function e(r){_classCallCheck(this,e),this.label=r,this.listeners={}}var r,t,n;return _createClass(e,[{key:"listen",value:(n=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:if(!this._handleTauriEvent(r,t)){e.next=2;break}return e.abrupt("return",Promise.resolve());case 2:return e.abrupt("return",d(r,t));case 3:case"end":return e.stop()}}),e,this)}))),function(e,r){return n.apply(this,arguments)})},{key:"once",value:(t=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:if(!this._handleTauriEvent(r,t)){e.next=2;break}return e.abrupt("return",Promise.resolve());case 2:return e.abrupt("return",_(r,t));case 3:case"end":return e.stop()}}),e,this)}))),function(e,r){return t.apply(this,arguments)})},{key:"emit",value:(r=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){var n,o;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:if(!_e.includes(r)){e.next=4;break}n=_createForOfIteratorHelper(this.listeners[r]||[]);try{for(n.s();!(o=n.n()).done;)(0,o.value)({type:r,payload:t})}catch(e){n.e(e)}finally{n.f()}return e.abrupt("return",Promise.resolve());case 4:return e.abrupt("return",w(r,this.label,t));case 5:case"end":return e.stop()}}),e,this)}))),function(e,t){return r.apply(this,arguments)})},{key:"_handleTauriEvent",value:function(e,r){return!!_e.includes(e)&&(e in this.listeners?this.listeners[e].push(r):this.listeners[e]=[r],!0)}},{key:"_emitTauriEvent",value:function(e){}}]),e}(),we=function(e){_inherits(t,e);var r=_createSuper(t);function t(e){var n,o=arguments.length>1&&void 0!==arguments[1]?arguments[1]:{};return _classCallCheck(this,t),n=r.call(this,e),u({__tauriModule:"Window",message:{cmd:"createWebview",options:_objectSpread({label:e},o)}}).then(_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",n.emit("tauri://created"));case 1:case"end":return e.stop()}}),e)})))).catch(function(){var e=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",n.emit("tauri://error",r));case 1:case"end":return e.stop()}}),e)})));return function(r){return e.apply(this,arguments)}}()),n}return _createClass(t,null,[{key:"getByLabel",value:function(e){return ge().some((function(r){return r.label===e}))?new ve(e):null}}]),t}(ve),be=new(function(){function e(){_classCallCheck(this,e)}var r,t,n,o,a,i,c,s,p,f,l,h,m,y,d,g,_,v,w,b,R;return _createClass(e,[{key:"setResizable",value:(R=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setResizable",resizable:r}}));case 1:case"end":return e.stop()}}),e)}))),function(e){return R.apply(this,arguments)})},{key:"setTitle",value:(b=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setTitle",title:r}}));case 1:case"end":return e.stop()}}),e)}))),function(e){return b.apply(this,arguments)})},{key:"maximize",value:(w=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"maximize"}}));case 1:case"end":return e.stop()}}),e)}))),function(){return w.apply(this,arguments)})},{key:"unmaximize",value:(v=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"unmaximize"}}));case 1:case"end":return e.stop()}}),e)}))),function(){return v.apply(this,arguments)})},{key:"minimize",value:(_=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"minimize"}}));case 1:case"end":return e.stop()}}),e)}))),function(){return _.apply(this,arguments)})},{key:"unminimize",value:(g=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"unminimize"}}));case 1:case"end":return e.stop()}}),e)}))),function(){return g.apply(this,arguments)})},{key:"show",value:(d=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"show"}}));case 1:case"end":return e.stop()}}),e)}))),function(){return d.apply(this,arguments)})},{key:"hide",value:(y=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"hide"}}));case 1:case"end":return e.stop()}}),e)}))),function(){return y.apply(this,arguments)})},{key:"setTransparent",value:(m=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setTransparent",transparent:r}}));case 1:case"end":return e.stop()}}),e)}))),function(e){return m.apply(this,arguments)})},{key:"setDecorations",value:(h=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setDecorations",decorations:r}}));case 1:case"end":return e.stop()}}),e)}))),function(e){return h.apply(this,arguments)})},{key:"setAlwaysOnTop",value:(l=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setAlwaysOnTop",alwaysOnTop:r}}));case 1:case"end":return e.stop()}}),e)}))),function(e){return l.apply(this,arguments)})},{key:"setWidth",value:(f=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setWidth",width:r}}));case 1:case"end":return e.stop()}}),e)}))),function(e){return f.apply(this,arguments)})},{key:"setHeight",value:(p=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setHeight",height:r}}));case 1:case"end":return e.stop()}}),e)}))),function(e){return p.apply(this,arguments)})},{key:"resize",value:(s=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"resize",width:r,height:t}}));case 1:case"end":return e.stop()}}),e)}))),function(e,r){return s.apply(this,arguments)})},{key:"setMinSize",value:(c=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setMinSize",minWidth:r,minHeight:t}}));case 1:case"end":return e.stop()}}),e)}))),function(e,r){return c.apply(this,arguments)})},{key:"setMaxSize",value:(i=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setMaxSize",maxWidth:r,maxHeight:t}}));case 1:case"end":return e.stop()}}),e)}))),function(e,r){return i.apply(this,arguments)})},{key:"setX",value:(a=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setX",x:r}}));case 1:case"end":return e.stop()}}),e)}))),function(e){return a.apply(this,arguments)})},{key:"setY",value:(o=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setY",y:r}}));case 1:case"end":return e.stop()}}),e)}))),function(e){return o.apply(this,arguments)})},{key:"setPosition",value:(n=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setPosition",x:r,y:t}}));case 1:case"end":return e.stop()}}),e)}))),function(e,r){return n.apply(this,arguments)})},{key:"setFullscreen",value:(t=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setFullscreen",fullscreen:r}}));case 1:case"end":return e.stop()}}),e)}))),function(e){return t.apply(this,arguments)})},{key:"setIcon",value:(r=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"Window",message:{cmd:"setIcon",icon:r}}));case 1:case"end":return e.stop()}}),e)}))),function(e){return r.apply(this,arguments)})}]),e}()),Re=Object.freeze({__proto__:null,WebviewWindow:we,getCurrent:function(){return new ve(window.__TAURI__.__currentWindow.label)},getAll:ge,manager:be});function ke(){return(ke=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:if("default"===window.Notification.permission){e.next=2;break}return e.abrupt("return",Promise.resolve("granted"===window.Notification.permission));case 2:return e.abrupt("return",u({__tauriModule:"Notification",message:{cmd:"isNotificationPermissionGranted"}}));case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function xe(){return(xe=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",window.Notification.requestPermission());case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var Te=Object.freeze({__proto__:null,sendNotification:function(e){"string"==typeof e?new window.Notification(e):new window.Notification(e.title,e)},requestPermission:function(){return xe.apply(this,arguments)},isPermissionGranted:function(){return ke.apply(this,arguments)}});function Ge(){return(Ge=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"GlobalShortcut",message:{cmd:"register",shortcut:r,handler:a(t)}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Pe(){return(Pe=_asyncToGenerator(regeneratorRuntime.mark((function e(r,t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"GlobalShortcut",message:{cmd:"registerAll",shortcuts:r,handler:a(t)}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Oe(){return(Oe=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"GlobalShortcut",message:{cmd:"isRegistered",shortcut:r}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Me(){return(Me=_asyncToGenerator(regeneratorRuntime.mark((function e(r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"GlobalShortcut",message:{cmd:"unregister",shortcut:r}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function je(){return(je=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.abrupt("return",u({__tauriModule:"GlobalShortcut",message:{cmd:"unregisterAll"}}));case 1:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var Fe=Object.freeze({__proto__:null,register:function(e,r){return Ge.apply(this,arguments)},registerAll:function(e,r){return Pe.apply(this,arguments)},isRegistered:function(e){return Oe.apply(this,arguments)},unregister:function(e){return Me.apply(this,arguments)},unregisterAll:function(){return je.apply(this,arguments)}});e.cli=p,e.dialog=h,e.event=x,e.fs=L,e.globalShortcut=Fe,e.http=he,e.notification=Te,e.path=oe,e.shell=de,e.tauri=c,e.window=Re,Object.defineProperty(e,"__esModule",{value:!0})}));


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
          window.__TAURI_INVOKE_HANDLER__(
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

  window.__TAURI__.invoke('tauri', {
    __tauriModule: 'Event',
    message: {
      cmd: 'listen',
      event: 'tauri://window-created',
      handler: window.__TAURI__.transformCallback(function (event) {
        if (event.payload) {
          var windowLabel = event.payload.label
          window.__TAURI__.__windows.push({ label: windowLabel })
        }
      })
    }
  })

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
