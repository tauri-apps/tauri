function ownKeys(e,t){var r=Object.keys(e);if(Object.getOwnPropertySymbols){var n=Object.getOwnPropertySymbols(e);t&&(n=n.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),r.push.apply(r,n)}return r}function _objectSpread(e){for(var t=1;t<arguments.length;t++){var r=null!=arguments[t]?arguments[t]:{};t%2?ownKeys(Object(r),!0).forEach((function(t){_defineProperty(e,t,r[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(r)):ownKeys(Object(r)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(r,t))}))}return e}function _defineProperty(e,t,r){return t in e?Object.defineProperty(e,t,{value:r,enumerable:!0,configurable:!0,writable:!0}):e[t]=r,e}function asyncGeneratorStep(e,t,r,n,o,a,i){try{var u=e[a](i),c=u.value}catch(e){return void r(e)}u.done?t(c):Promise.resolve(c).then(n,o)}function _asyncToGenerator(e){return function(){var t=this,r=arguments;return new Promise((function(n,o){var a=e.apply(t,r);function i(e){asyncGeneratorStep(a,n,o,i,u,"next",e)}function u(e){asyncGeneratorStep(a,n,o,i,u,"throw",e)}i(void 0)}))}}function _typeof(e){return(_typeof="function"==typeof Symbol&&"symbol"==typeof Symbol.iterator?function(e){return typeof e}:function(e){return e&&"function"==typeof Symbol&&e.constructor===Symbol&&e!==Symbol.prototype?"symbol":typeof e})(e)}!function(e,t){"object"===("undefined"==typeof exports?"undefined":_typeof(exports))&&"undefined"!=typeof module?t(exports):"function"==typeof define&&define.amd?define(["exports"],t):t((e="undefined"!=typeof globalThis?globalThis:e||self).__TAURI__={})}(this,(function(e){"use strict";var t=function(e){var t,r=Object.prototype,n=r.hasOwnProperty,o="function"==typeof Symbol?Symbol:{},a=o.iterator||"@@iterator",i=o.asyncIterator||"@@asyncIterator",u=o.toStringTag||"@@toStringTag";function c(e,t,r){return Object.defineProperty(e,t,{value:r,enumerable:!0,configurable:!0,writable:!0}),e[t]}try{c({},"")}catch(e){c=function(e,t,r){return e[t]=r}}function s(e,t,r,n){var o=t&&t.prototype instanceof y?t:y,a=Object.create(o.prototype),i=new j(n||[]);return a._invoke=function(e,t,r){var n=f;return function(o,a){if(n===m)throw new Error("Generator is already running");if(n===h){if("throw"===o)throw a;return O()}for(r.method=o,r.arg=a;;){var i=r.delegate;if(i){var u=P(i,r);if(u){if(u===d)continue;return u}}if("next"===r.method)r.sent=r._sent=r.arg;else if("throw"===r.method){if(n===f)throw n=h,r.arg;r.dispatchException(r.arg)}else"return"===r.method&&r.abrupt("return",r.arg);n=m;var c=p(e,t,r);if("normal"===c.type){if(n=r.done?h:l,c.arg===d)continue;return{value:c.arg,done:r.done}}"throw"===c.type&&(n=h,r.method="throw",r.arg=c.arg)}}}(e,r,i),a}function p(e,t,r){try{return{type:"normal",arg:e.call(t,r)}}catch(e){return{type:"throw",arg:e}}}e.wrap=s;var f="suspendedStart",l="suspendedYield",m="executing",h="completed",d={};function y(){}function g(){}function v(){}var w={};w[a]=function(){return this};var b=Object.getPrototypeOf,x=b&&b(b(F([])));x&&x!==r&&n.call(x,a)&&(w=x);var _=v.prototype=y.prototype=Object.create(w);function R(e){["next","throw","return"].forEach((function(t){c(e,t,(function(e){return this._invoke(t,e)}))}))}function T(e,t){function r(o,a,i,u){var c=p(e[o],e,a);if("throw"!==c.type){var s=c.arg,f=s.value;return f&&"object"===_typeof(f)&&n.call(f,"__await")?t.resolve(f.__await).then((function(e){r("next",e,i,u)}),(function(e){r("throw",e,i,u)})):t.resolve(f).then((function(e){s.value=e,i(s)}),(function(e){return r("throw",e,i,u)}))}u(c.arg)}var o;this._invoke=function(e,n){function a(){return new t((function(t,o){r(e,n,t,o)}))}return o=o?o.then(a,a):a()}}function P(e,r){var n=e.iterator[r.method];if(n===t){if(r.delegate=null,"throw"===r.method){if(e.iterator.return&&(r.method="return",r.arg=t,P(e,r),"throw"===r.method))return d;r.method="throw",r.arg=new TypeError("The iterator does not provide a 'throw' method")}return d}var o=p(n,e.iterator,r.arg);if("throw"===o.type)return r.method="throw",r.arg=o.arg,r.delegate=null,d;var a=o.arg;return a?a.done?(r[e.resultName]=a.value,r.next=e.nextLoc,"return"!==r.method&&(r.method="next",r.arg=t),r.delegate=null,d):a:(r.method="throw",r.arg=new TypeError("iterator result is not an object"),r.delegate=null,d)}function k(e){var t={tryLoc:e[0]};1 in e&&(t.catchLoc=e[1]),2 in e&&(t.finallyLoc=e[2],t.afterLoc=e[3]),this.tryEntries.push(t)}function G(e){var t=e.completion||{};t.type="normal",delete t.arg,e.completion=t}function j(e){this.tryEntries=[{tryLoc:"root"}],e.forEach(k,this),this.reset(!0)}function F(e){if(e){var r=e[a];if(r)return r.call(e);if("function"==typeof e.next)return e;if(!isNaN(e.length)){var o=-1,i=function r(){for(;++o<e.length;)if(n.call(e,o))return r.value=e[o],r.done=!1,r;return r.value=t,r.done=!0,r};return i.next=i}}return{next:O}}function O(){return{value:t,done:!0}}return g.prototype=_.constructor=v,v.constructor=g,g.displayName=c(v,u,"GeneratorFunction"),e.isGeneratorFunction=function(e){var t="function"==typeof e&&e.constructor;return!!t&&(t===g||"GeneratorFunction"===(t.displayName||t.name))},e.mark=function(e){return Object.setPrototypeOf?Object.setPrototypeOf(e,v):(e.__proto__=v,c(e,u,"GeneratorFunction")),e.prototype=Object.create(_),e},e.awrap=function(e){return{__await:e}},R(T.prototype),T.prototype[i]=function(){return this},e.AsyncIterator=T,e.async=function(t,r,n,o,a){void 0===a&&(a=Promise);var i=new T(s(t,r,n,o),a);return e.isGeneratorFunction(r)?i:i.next().then((function(e){return e.done?e.value:i.next()}))},R(_),c(_,u,"Generator"),_[a]=function(){return this},_.toString=function(){return"[object Generator]"},e.keys=function(e){var t=[];for(var r in e)t.push(r);return t.reverse(),function r(){for(;t.length;){var n=t.pop();if(n in e)return r.value=n,r.done=!1,r}return r.done=!0,r}},e.values=F,j.prototype={constructor:j,reset:function(e){if(this.prev=0,this.next=0,this.sent=this._sent=t,this.done=!1,this.delegate=null,this.method="next",this.arg=t,this.tryEntries.forEach(G),!e)for(var r in this)"t"===r.charAt(0)&&n.call(this,r)&&!isNaN(+r.slice(1))&&(this[r]=t)},stop:function(){this.done=!0;var e=this.tryEntries[0].completion;if("throw"===e.type)throw e.arg;return this.rval},dispatchException:function(e){if(this.done)throw e;var r=this;function o(n,o){return u.type="throw",u.arg=e,r.next=n,o&&(r.method="next",r.arg=t),!!o}for(var a=this.tryEntries.length-1;a>=0;--a){var i=this.tryEntries[a],u=i.completion;if("root"===i.tryLoc)return o("end");if(i.tryLoc<=this.prev){var c=n.call(i,"catchLoc"),s=n.call(i,"finallyLoc");if(c&&s){if(this.prev<i.catchLoc)return o(i.catchLoc,!0);if(this.prev<i.finallyLoc)return o(i.finallyLoc)}else if(c){if(this.prev<i.catchLoc)return o(i.catchLoc,!0)}else{if(!s)throw new Error("try statement without catch or finally");if(this.prev<i.finallyLoc)return o(i.finallyLoc)}}}},abrupt:function(e,t){for(var r=this.tryEntries.length-1;r>=0;--r){var o=this.tryEntries[r];if(o.tryLoc<=this.prev&&n.call(o,"finallyLoc")&&this.prev<o.finallyLoc){var a=o;break}}a&&("break"===e||"continue"===e)&&a.tryLoc<=t&&t<=a.finallyLoc&&(a=null);var i=a?a.completion:{};return i.type=e,i.arg=t,a?(this.method="next",this.next=a.finallyLoc,d):this.complete(i)},complete:function(e,t){if("throw"===e.type)throw e.arg;return"break"===e.type||"continue"===e.type?this.next=e.arg:"return"===e.type?(this.rval=this.arg=e.arg,this.method="return",this.next="end"):"normal"===e.type&&t&&(this.next=t),d},finish:function(e){for(var t=this.tryEntries.length-1;t>=0;--t){var r=this.tryEntries[t];if(r.finallyLoc===e)return this.complete(r.completion,r.afterLoc),G(r),d}},catch:function(e){for(var t=this.tryEntries.length-1;t>=0;--t){var r=this.tryEntries[t];if(r.tryLoc===e){var n=r.completion;if("throw"===n.type){var o=n.arg;G(r)}return o}}throw new Error("illegal catch attempt")},delegateYield:function(e,r,n){return this.delegate={iterator:F(e),resultName:r,nextLoc:n},"next"===this.method&&(this.arg=t),d}},e}("object"===("undefined"==typeof module?"undefined":_typeof(module))?module.exports:{});try{regeneratorRuntime=t}catch(e){Function("r","regeneratorRuntime = r")(t)}function r(e){for(var t=void 0,r=e[0],n=1;n<e.length;){var o=e[n],a=e[n+1];if(n+=2,("optionalAccess"===o||"optionalCall"===o)&&null==r)return;"access"===o||"optionalAccess"===o?(t=r,r=a(r)):"call"!==o&&"optionalCall"!==o||(r=a((function(){for(var e,n=arguments.length,o=new Array(n),a=0;a<n;a++)o[a]=arguments[a];return(e=r).call.apply(e,[t].concat(o))})),t=void 0)}return r}function n(){return Math.floor(65536*(1+Math.random())).toString(16).substring(1)}function o(){return n()+n()+"-"+n()+"-"+n()+"-"+n()+"-"+n()+n()+n()}function a(e){window.__TAURI_INVOKE_HANDLER__(JSON.stringify(e))}function i(e){var t=arguments.length>1&&void 0!==arguments[1]&&arguments[1],n=o();return Object.defineProperty(window,n,{value:function(o){return t&&Reflect.deleteProperty(window,n),r([e,"optionalCall",function(e){return e(o)}])},writable:!1,configurable:!0}),n}function u(e){return c.apply(this,arguments)}function c(){return(c=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,new Promise((function(e,r){var n=i((function(t){e(t),Reflect.deleteProperty(window,o)}),!0),o=i((function(e){r(e),Reflect.deleteProperty(window,n)}),!0);a(_objectSpread({callback:n,error:o},t))}));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var s=Object.freeze({__proto__:null,invoke:a,transformCallback:i,promisified:u});function p(){return(p=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Cli",message:{cmd:"cliMatches"}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var f=Object.freeze({__proto__:null,getMatches:function(){return p.apply(this,arguments)}});function l(){return(l=_asyncToGenerator(regeneratorRuntime.mark((function e(){var t,r=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(t=r.length>0&&void 0!==r[0]?r[0]:{})&&Object.freeze(t),e.next=4,u({module:"Dialog",message:{cmd:"openDialog",options:t}});case 4:return e.abrupt("return",e.sent);case 5:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function m(){return(m=_asyncToGenerator(regeneratorRuntime.mark((function e(){var t,r=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(t=r.length>0&&void 0!==r[0]?r[0]:{})&&Object.freeze(t),e.next=4,u({module:"Dialog",message:{cmd:"saveDialog",options:t}});case 4:return e.abrupt("return",e.sent);case 5:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var h=Object.freeze({__proto__:null,open:function(){return l.apply(this,arguments)},save:function(){return m.apply(this,arguments)}});var d,y=Object.freeze({__proto__:null,listen:function(e,t){var r=arguments.length>2&&void 0!==arguments[2]&&arguments[2];a({module:"Event",message:{cmd:"listen",event:e,handler:i(t,r),once:r}})},emit:function(e,t){a({module:"Event",message:{cmd:"emit",event:e,payload:t}})}});function g(){return(g=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return r=n.length>1&&void 0!==n[1]?n[1]:{},e.next=3,u({module:"Fs",message:{cmd:"readTextFile",path:t,options:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function v(){return(v=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return r=n.length>1&&void 0!==n[1]?n[1]:{},e.next=3,u({module:"Fs",message:{cmd:"readBinaryFile",path:t,options:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function w(){return(w=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(r=n.length>1&&void 0!==n[1]?n[1]:{})&&Object.freeze(r),"object"===_typeof(t)&&Object.freeze(t),e.next=5,u({module:"Fs",message:{cmd:"writeFile",path:t.path,contents:t.contents,options:r}});case 5:return e.abrupt("return",e.sent);case 6:case"end":return e.stop()}}),e)})))).apply(this,arguments)}!function(e){e[e.Audio=1]="Audio";e[e.Cache=2]="Cache";e[e.Config=3]="Config";e[e.Data=4]="Data";e[e.LocalData=5]="LocalData";e[e.Desktop=6]="Desktop";e[e.Document=7]="Document";e[e.Download=8]="Download";e[e.Executable=9]="Executable";e[e.Font=10]="Font";e[e.Home=11]="Home";e[e.Picture=12]="Picture";e[e.Public=13]="Public";e[e.Runtime=14]="Runtime";e[e.Template=15]="Template";e[e.Video=16]="Video";e[e.Resource=17]="Resource";e[e.App=18]="App"}(d||(d={}));var b=65536;function x(e){var t=function(e){if(e.length<b)return String.fromCharCode.apply(null,Array.from(e));for(var t="",r=e.length,n=0;n<r;n++){var o=e.subarray(n*b,(n+1)*b);t+=String.fromCharCode.apply(null,Array.from(o))}return t}(new Uint8Array(e));return btoa(t)}function _(){return(_=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(r=n.length>1&&void 0!==n[1]?n[1]:{})&&Object.freeze(r),"object"===_typeof(t)&&Object.freeze(t),e.next=5,u({module:"Fs",message:{cmd:"writeBinaryFile",path:t.path,contents:x(t.contents),options:r}});case 5:return e.abrupt("return",e.sent);case 6:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function R(){return(R=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return r=n.length>1&&void 0!==n[1]?n[1]:{},e.next=3,u({module:"Fs",message:{cmd:"readDir",path:t,options:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function T(){return(T=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return r=n.length>1&&void 0!==n[1]?n[1]:{},e.next=3,u({module:"Fs",message:{cmd:"createDir",path:t,options:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function P(){return(P=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return r=n.length>1&&void 0!==n[1]?n[1]:{},e.next=3,u({module:"Fs",message:{cmd:"removeDir",path:t,options:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function k(){return(k=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){var n,o=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return n=o.length>2&&void 0!==o[2]?o[2]:{},e.next=3,u({module:"Fs",message:{cmd:"copyFile",source:t,destination:r,options:n}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function G(){return(G=_asyncToGenerator(regeneratorRuntime.mark((function e(t){var r,n=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return r=n.length>1&&void 0!==n[1]?n[1]:{},e.next=3,u({module:"Fs",message:{cmd:"removeFile",path:t,options:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function j(){return(j=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){var n,o=arguments;return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return n=o.length>2&&void 0!==o[2]?o[2]:{},e.next=3,u({module:"Fs",message:{cmd:"renameFile",oldPath:t,newPath:r,options:n}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var F=Object.freeze({__proto__:null,get BaseDirectory(){return d},get Dir(){return d},readTextFile:function(e){return g.apply(this,arguments)},readBinaryFile:function(e){return v.apply(this,arguments)},writeFile:function(e){return w.apply(this,arguments)},writeBinaryFile:function(e){return _.apply(this,arguments)},readDir:function(e){return R.apply(this,arguments)},createDir:function(e){return T.apply(this,arguments)},removeDir:function(e){return P.apply(this,arguments)},copyFile:function(e,t){return k.apply(this,arguments)},removeFile:function(e){return G.apply(this,arguments)},renameFile:function(e,t){return j.apply(this,arguments)}});function O(){return(O=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.App}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function D(){return(D=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Audio}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function S(){return(S=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Cache}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function z(){return(z=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Config}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function E(){return(E=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Data}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function L(){return(L=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Desktop}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function W(){return(W=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Document}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function A(){return(A=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Download}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function N(){return(N=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Executable}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function C(){return(C=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Font}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function H(){return(H=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Home}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function M(){return(M=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.LocalData}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function B(){return(B=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Picture}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function I(){return(I=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Public}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function q(){return(q=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Resource}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function K(){return(K=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Runtime}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function U(){return(U=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Template}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function V(){return(V=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:"",directory:d.Video}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function Y(){return(Y=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Fs",message:{cmd:"resolvePath",path:t,directory:r}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var J,X,Q=Object.freeze({__proto__:null,appDir:function(){return O.apply(this,arguments)},audioDir:function(){return D.apply(this,arguments)},cacheDir:function(){return S.apply(this,arguments)},configDir:function(){return z.apply(this,arguments)},dataDir:function(){return E.apply(this,arguments)},desktopDir:function(){return L.apply(this,arguments)},documentDir:function(){return W.apply(this,arguments)},downloadDir:function(){return A.apply(this,arguments)},executableDir:function(){return N.apply(this,arguments)},fontDir:function(){return C.apply(this,arguments)},homeDir:function(){return H.apply(this,arguments)},localDataDir:function(){return M.apply(this,arguments)},pictureDir:function(){return B.apply(this,arguments)},publicDir:function(){return I.apply(this,arguments)},resourceDir:function(){return q.apply(this,arguments)},runtimeDir:function(){return K.apply(this,arguments)},templateDir:function(){return U.apply(this,arguments)},videoDir:function(){return V.apply(this,arguments)},resolvePath:function(e,t){return Y.apply(this,arguments)}});function Z(e){return $.apply(this,arguments)}function $(){return($=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"Http",message:{cmd:"httpRequest",options:t}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function ee(){return(ee=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,Z(_objectSpread({method:"GET",url:t},r));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function te(){return(te=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r,n){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,Z(_objectSpread({method:"POST",url:t,body:r},n));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function re(){return(re=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r,n){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,Z(_objectSpread({method:"PUT",url:t,body:r},n));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function ne(){return(ne=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,Z(_objectSpread({method:"PATCH",url:t},r));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function oe(){return(oe=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,Z(_objectSpread({method:"DELETE",url:t},r));case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}!function(e){e[e.JSON=1]="JSON";e[e.Text=2]="Text";e[e.Binary=3]="Binary"}(J||(J={})),function(e){e[e.Form=1]="Form";e[e.File=2]="File";e[e.Auto=3]="Auto"}(X||(X={}));var ae=Object.freeze({__proto__:null,get ResponseType(){return J},get BodyType(){return X},request:Z,get:function(e,t){return ee.apply(this,arguments)},post:function(e,t,r){return te.apply(this,arguments)},put:function(e,t,r){return re.apply(this,arguments)},patch:function(e,t){return ne.apply(this,arguments)},httpDelete:function(e,t){return oe.apply(this,arguments)}});function ie(){return(ie=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return"object"===_typeof(r)&&Object.freeze(r),e.next=3,u({module:"Shell",message:{cmd:"execute",command:t,args:"string"==typeof r?[r]:r}});case 3:return e.abrupt("return",e.sent);case 4:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var ue=Object.freeze({__proto__:null,execute:function(e,t){return ie.apply(this,arguments)},open:function(e){a({module:"Shell",message:{cmd:"open",uri:e}})}});var ce=Object.freeze({__proto__:null,setResizable:function(e){a({module:"Window",message:{cmd:"setResizable",resizable:e}})},setTitle:function(e){a({module:"Window",message:{cmd:"setTitle",title:e}})},maximize:function(){a({module:"Window",message:{cmd:"maximize"}})},unmaximize:function(){a({module:"Window",message:{cmd:"unmaximize"}})},minimize:function(){a({module:"Window",message:{cmd:"minimize"}})},unminimize:function(){a({module:"Window",message:{cmd:"unminimize"}})},show:function(){a({module:"Window",message:{cmd:"show"}})},hide:function(){a({module:"Window",message:{cmd:"hide"}})},setTransparent:function(e){a({module:"Window",message:{cmd:"setTransparent",transparent:e}})},setDecorations:function(e){a({module:"Window",message:{cmd:"setDecorations",decorations:e}})},setAlwaysOnTop:function(e){a({module:"Window",message:{cmd:"setAlwaysOnTop",alwaysOnTop:e}})},setWidth:function(e){a({module:"Window",message:{cmd:"setWidth",width:e}})},setHeight:function(e){a({module:"Window",message:{cmd:"setHeight",height:e}})},resize:function(e,t){a({module:"Window",message:{cmd:"resize",width:e,height:t}})},setMinSize:function(e,t){a({module:"Window",message:{cmd:"setMinSize",minWidth:e,minHeight:t}})},setMaxSize:function(e,t){a({module:"Window",message:{cmd:"setMaxSize",maxWidth:e,maxHeight:t}})},setX:function(e){a({module:"Window",message:{cmd:"setX",x:e}})},setY:function(e){a({module:"Window",message:{cmd:"setY",y:e}})},setPosition:function(e,t){a({module:"Window",message:{cmd:"setPosition",x:e,y:t}})},setFullscreen:function(e){a({module:"Window",message:{cmd:"setFullscreen",fullscreen:e}})},setIcon:function(e){a({module:"Window",message:{cmd:"setIcon",icon:e}})}});function se(){return(se=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:if("default"===window.Notification.permission){e.next=4;break}return e.next=3,Promise.resolve("granted"===window.Notification.permission);case 3:return e.abrupt("return",e.sent);case 4:return e.next=6,u({module:"Notification",message:{cmd:"isNotificationPermissionGranted"}});case 6:return e.abrupt("return",e.sent);case 7:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function pe(){return(pe=_asyncToGenerator(regeneratorRuntime.mark((function e(){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,window.Notification.requestPermission();case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var fe=Object.freeze({__proto__:null,sendNotification:function(e){"string"==typeof e?new window.Notification(e):new window.Notification(e.title,e)},requestPermission:function(){return pe.apply(this,arguments)},isPermissionGranted:function(){return se.apply(this,arguments)}});function le(){return(le=_asyncToGenerator(regeneratorRuntime.mark((function e(t,r){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"GlobalShortcut",message:{cmd:"register",shortcut:t,handler:i(r)}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}function me(){return(me=_asyncToGenerator(regeneratorRuntime.mark((function e(t){return regeneratorRuntime.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.next=2,u({module:"GlobalShortcut",message:{cmd:"unregister",shortcut:t}});case 2:return e.abrupt("return",e.sent);case 3:case"end":return e.stop()}}),e)})))).apply(this,arguments)}var he=Object.freeze({__proto__:null,registerShortcut:function(e,t){return le.apply(this,arguments)},unregisterShortcut:function(e){return me.apply(this,arguments)}});e.cli=f,e.dialog=h,e.event=y,e.fs=F,e.globalShortcut=he,e.http=ae,e.notification=fe,e.path=Q,e.shell=ue,e.tauri=s,e.window=ce,Object.defineProperty(e,"__esModule",{value:!0})}));


// polyfills
if (!String.prototype.startsWith) {
  String.prototype.startsWith = function (searchString, position) {
    position = position || 0
    return this.substr(position, searchString.length) === searchString
  }
}

;(function () {
  function s4() {
    return Math.floor((1 + Math.random()) * 0x10000)
      .toString(16)
      .substring(1)
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
    )
  }

  function ownKeys(object, enumerableOnly) {
    var keys = Object.keys(object)
    if (Object.getOwnPropertySymbols) {
      var symbols = Object.getOwnPropertySymbols(object)
      if (enumerableOnly)
        symbols = symbols.filter(function (sym) {
          return Object.getOwnPropertyDescriptor(object, sym).enumerable
        })
      keys.push.apply(keys, symbols)
    }
    return keys
  }

  function _objectSpread(target) {
    for (var i = 1; i < arguments.length; i++) {
      var source = arguments[i] != null ? arguments[i] : {}
      if (i % 2) {
        ownKeys(source, true).forEach(function (key) {
          _defineProperty(target, key, source[key])
        })
      } else if (Object.getOwnPropertyDescriptors) {
        Object.defineProperties(
          target,
          Object.getOwnPropertyDescriptors(source)
        )
      } else {
        ownKeys(source).forEach(function (key) {
          Object.defineProperty(
            target,
            key,
            Object.getOwnPropertyDescriptor(source, key)
          )
        })
      }
    }
    return target
  }

  function _defineProperty(obj, key, value) {
    if (key in obj) {
      Object.defineProperty(obj, key, {
        value: value,
        enumerable: true,
        configurable: true,
        writable: true
      })
    } else {
      obj[key] = value
    }
    return obj
  }

  if (!window.__TAURI__) {
    window.__TAURI__ = {}
  }

  window.__TAURI__.transformCallback = function transformCallback(
    callback,
    once
  ) {
    var identifier = uid()

    window[identifier] = function (result) {
      if (once) {
        delete window[identifier]
      }

      return callback && callback(result)
    }

    return identifier
  }

  window.__TAURI__.promisified = function promisified(args) {
    var _this = this

    return new Promise(function (resolve, reject) {
      var callback = _this.transformCallback(function (r) {
        resolve(r)
        delete window[error]
      }, true)
      var error = _this.transformCallback(function (e) {
        reject(e)
        delete window[callback]
      }, true)

      if (window.__TAURI_INVOKE_HANDLER__) {
        window.__TAURI_INVOKE_HANDLER__(
          JSON.stringify(_objectSpread(
            {
              callback: callback,
              error: error
            },
            args
          ))
        )
      } else {
        window.addEventListener('DOMContentLoaded', function () {
          window.__TAURI_INVOKE_HANDLER__(
            JSON.stringify(_objectSpread(
              {
                callback: callback,
                error: error
              },
              args
            ))
          )
        })
      }
    })
  }

  // open <a href="..."> links with the Tauri API
  function __openLinks() {
    document.querySelector('body').addEventListener(
      'click',
      function (e) {
        var target = e.target
        while (target != null) {
          if (
            target.matches ? target.matches('a') : target.msMatchesSelector('a')
          ) {
            if (
              target.href &&
              target.href.startsWith('http') &&
              target.target === '_blank'
            ) {
              window.__TAURI_INVOKE_HANDLER__(JSON.stringify({
                module: 'Shell',
                message: {
                  cmd: 'open',
                  uri: target.href
                }
              }))
              e.preventDefault()
            }
            break
          }
          target = target.parentElement
        }
      },
      true
    )
  }

  if (
    document.readyState === 'complete' ||
    document.readyState === 'interactive'
  ) {
    __openLinks()
  } else {
    window.addEventListener(
      'DOMContentLoaded',
      function () {
        __openLinks()
      },
      true
    )
  }

  let permissionSettable = false
  let permissionValue = 'default'

  function isPermissionGranted() {
    if (window.Notification.permission !== 'default') {
      return Promise.resolve(window.Notification.permission === 'granted')
    }
    return window.__TAURI__.promisified({
      module: 'Notification',
      message: {
        cmd: 'isNotificationPermissionGranted'
      }
    })
  }

  function setNotificationPermission(value) {
    permissionSettable = true
    window.Notification.permission = value
    permissionSettable = false
  }

  function requestPermission() {
    return window.__TAURI__
      .promisified({
        module: 'Notification',
        message: {
          cmd: 'requestNotificationPermission'
        }
      })
      .then(function (permission) {
        setNotificationPermission(permission)
        return permission
      })
  }

  function sendNotification(options) {
    if (typeof options === 'object') {
      Object.freeze(options)
    }

    isPermissionGranted().then(function (permission) {
      if (permission) {
        return window.__TAURI__.promisified({
          module: 'Notification',
          message: {
            cmd: 'notification',
            options:
              typeof options === 'string'
                ? {
                    title: options
                  }
                : options
          }
        })
      }
    })
  }

  window.Notification = function (title, options) {
    var opts = options || {}
    sendNotification(
      Object.assign(opts, {
        title: title
      })
    )
  }

  window.Notification.requestPermission = requestPermission

  Object.defineProperty(window.Notification, 'permission', {
    enumerable: true,
    get: function () {
      return permissionValue
    },
    set: function (v) {
      if (!permissionSettable) {
        throw new Error('Readonly property')
      }
      permissionValue = v
    }
  })

  isPermissionGranted().then(function (response) {
    if (response === null) {
      setNotificationPermission('default')
    } else {
      setNotificationPermission(response ? 'granted' : 'denied')
    }
  })

  window.alert = function (message) {
    window.__TAURI_INVOKE_HANDLER__(JSON.stringify({
      module: 'Dialog',
      message: {
        cmd: 'messageDialog',
        message: message
      }
    }))
  }

  window.confirm = function (message) {
    return window.__TAURI__.promisified({
      module: 'Dialog',
      message: {
        cmd: 'askDialog',
        message: message
      }
    })
  }
})()