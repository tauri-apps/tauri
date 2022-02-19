(function webpackUniversalModuleDefinition(root, factory) {
	if(typeof exports === 'object' && typeof module === 'object')
		module.exports = factory();
	else if(typeof define === 'function' && define.amd)
		define([], factory);
	else if(typeof exports === 'object')
		exports["tauri"] = factory();
	else
		root["tauri"] = factory();
})(this, function() {
return /******/ (() => { // webpackBootstrap
/******/ 	"use strict";
/******/ 	var __webpack_modules__ = ({

/***/ "./src/helpers/logger.ts":
/*!*******************************!*\
  !*** ./src/helpers/logger.ts ***!
  \*******************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {


// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
var chalk_1 = __importDefault(__webpack_require__(/*! chalk */ "chalk"));
var ms_1 = __importDefault(__webpack_require__(/*! ms */ "ms"));
var prevTime;
exports.default = (function (banner, color) {
    if (color === void 0) { color = chalk_1.default.green; }
    return function (msg) {
        var curr = +new Date();
        var diff = curr - (prevTime || curr);
        prevTime = curr;
        if (msg) {
            console.log(
            // TODO: proper typings for color and banner
            // eslint-disable-next-line @typescript-eslint/restrict-template-expressions, @typescript-eslint/no-unsafe-call
            " " + color(String(banner)) + " " + msg + " " + chalk_1.default.green("+" + ms_1.default(diff)));
        }
        else {
            console.log();
        }
    };
});


/***/ }),

/***/ "./src/helpers/spawn.ts":
/*!******************************!*\
  !*** ./src/helpers/spawn.ts ***!
  \******************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {


// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.spawnSync = exports.spawn = void 0;
var cross_spawn_1 = __importDefault(__webpack_require__(/*! cross-spawn */ "cross-spawn"));
var logger_1 = __importDefault(__webpack_require__(/*! ./logger */ "./src/helpers/logger.ts"));
var chalk_1 = __importDefault(__webpack_require__(/*! chalk */ "chalk"));
var log = logger_1.default('app:spawn');
var warn = logger_1.default('app:spawn', chalk_1.default.red);
/*
  Returns pid, takes onClose
 */
var spawn = function (cmd, params, cwd, onClose) {
    var _a;
    log("Running \"" + cmd + " " + params.join(' ') + "\"");
    log();
    // TODO: move to execa?
    var runner = cross_spawn_1.default(cmd, params, {
        stdio: 'inherit',
        cwd: cwd,
        env: process.env
    });
    runner.on('close', function (code) {
        var _a;
        log();
        if (code) {
            // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
            log("Command \"" + cmd + "\" failed with exit code: " + code);
        }
        // eslint-disable-next-line @typescript-eslint/prefer-optional-chain
        onClose && onClose(code !== null && code !== void 0 ? code : 0, (_a = runner.pid) !== null && _a !== void 0 ? _a : 0);
    });
    return (_a = runner.pid) !== null && _a !== void 0 ? _a : 0;
};
exports.spawn = spawn;
/*
  Returns nothing, takes onFail
 */
var spawnSync = function (cmd, params, cwd, onFail) {
    log("[sync] Running \"" + cmd + " " + params.join(' ') + "\"");
    log();
    var runner = cross_spawn_1.default.sync(cmd, params, {
        stdio: 'inherit',
        cwd: cwd
    });
    // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
    if (runner.status || runner.error) {
        warn();
        // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
        warn("\u26A0\uFE0F  Command \"" + cmd + "\" failed with exit code: " + runner.status);
        if (runner.status === null) {
            warn("\u26A0\uFE0F  Please globally install \"" + cmd + "\"");
        }
        // eslint-disable-next-line @typescript-eslint/prefer-optional-chain
        onFail && onFail();
        process.exit(1);
    }
};
exports.spawnSync = spawnSync;


/***/ }),

/***/ "chalk":
/*!************************!*\
  !*** external "chalk" ***!
  \************************/
/***/ ((module) => {

module.exports = require("chalk");;

/***/ }),

/***/ "cross-spawn":
/*!******************************!*\
  !*** external "cross-spawn" ***!
  \******************************/
/***/ ((module) => {

module.exports = require("cross-spawn");;

/***/ }),

/***/ "ms":
/*!*********************!*\
  !*** external "ms" ***!
  \*********************/
/***/ ((module) => {

module.exports = require("ms");;

/***/ })

/******/ 	});
/************************************************************************/
/******/ 	// The module cache
/******/ 	var __webpack_module_cache__ = {};
/******/ 	
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/ 		// Check if module is in cache
/******/ 		var cachedModule = __webpack_module_cache__[moduleId];
/******/ 		if (cachedModule !== undefined) {
/******/ 			return cachedModule.exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = __webpack_module_cache__[moduleId] = {
/******/ 			// no module.id needed
/******/ 			// no module.loaded needed
/******/ 			exports: {}
/******/ 		};
/******/ 	
/******/ 		// Execute the module function
/******/ 		__webpack_modules__[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/ 	
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/ 	
/************************************************************************/
/******/ 	
/******/ 	// startup
/******/ 	// Load entry module and return exports
/******/ 	// This entry module is referenced by other modules so it can't be inlined
/******/ 	var __webpack_exports__ = __webpack_require__("./src/helpers/spawn.ts");
/******/ 	
/******/ 	return __webpack_exports__;
/******/ })()
;
});
//# sourceMappingURL=spawn.js.map