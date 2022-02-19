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
/******/ 	var __webpack_modules__ = ({

/***/ "../cli.rs/Cargo.toml":
/*!****************************!*\
  !*** ../cli.rs/Cargo.toml ***!
  \****************************/
/***/ ((module) => {

module.exports    = {
	"workspace": {},
	"package": {
		"name": "tauri-cli",
		"version": "1.0.0-beta.5",
		"authors": [
			"Tauri Programme within The Commons Conservancy"
		],
		"edition": "2018",
		"categories": [
			"gui",
			"web-programming"
		],
		"license": "Apache-2.0 OR MIT",
		"homepage": "https://tauri.studio",
		"repository": "https://github.com/tauri-apps/tauri",
		"description": "Command line interface for building Tauri apps",
		"include": [
			"src/",
			"/templates",
			"MergeModules/",
			"*.json",
			"*.rs"
		]
	},
	"bin": [
		{
			"name": "cargo-tauri",
			"path": "src/main.rs"
		}
	],
	"dependencies": {
		"clap": {
			"version": "3.0.0-beta.2",
			"features": [
				"yaml"
			]
		},
		"anyhow": "1.0",
		"tauri-bundler": {
			"version": "1.0.0-beta.3",
			"path": "../bundler"
		},
		"colored": "2.0",
		"once_cell": "1.8",
		"serde": {
			"version": "1.0",
			"features": [
				"derive"
			]
		},
		"serde_json": "1.0",
		"serde_with": "1.9",
		"notify": "4.0",
		"shared_child": "0.3",
		"toml_edit": "0.2",
		"json-patch": "0.2",
		"schemars": "0.8",
		"toml": "0.5",
		"valico": "3.6",
		"handlebars": "4.1",
		"include_dir": "0.6",
		"minisign": "0.6",
		"base64": "0.13.0",
		"ureq": "2.1",
		"os_info": "3.0",
		"semver": "1.0",
		"regex": "1.5",
		"lazy_static": "1",
		"libc": "0.2",
		"terminal_size": "0.1",
		"unicode-width": "0.1",
		"tempfile": "3",
		"zeroize": "1.3"
	},
	"target": {
		"cfg(windows)": {
			"dependencies": {
				"winapi": {
					"version": "0.3",
					"features": [
						"winbase",
						"winuser",
						"consoleapi",
						"processenv",
						"wincon"
					]
				},
				"encode_unicode": "0.3"
			}
		},
		"cfg(target_os = \"linux\")": {
			"dependencies": {
				"heck": "0.3"
			}
		}
	},
	"build-dependencies": {
		"schemars": "0.8",
		"serde": {
			"version": "1.0",
			"features": [
				"derive"
			]
		},
		"serde_json": "1.0",
		"serde_with": "1.9"
	}
}

/***/ }),

/***/ "./src/api/cli.ts":
/*!************************!*\
  !*** ./src/api/cli.ts ***!
  \************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {

"use strict";

// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.build = exports.dev = exports.init = void 0;
var rust_cli_1 = __webpack_require__(/*! ../helpers/rust-cli */ "./src/helpers/rust-cli.ts");
function toKebabCase(value) {
    return value
        .replace(/([a-z])([A-Z])/g, '$1-$2')
        .replace(/\s+/g, '-')
        .toLowerCase();
}
function runCliCommand(command, args) {
    return __awaiter(this, void 0, void 0, function () {
        var argsArray, _i, _a, _b, argName, argValue;
        return __generator(this, function (_c) {
            switch (_c.label) {
                case 0:
                    argsArray = [];
                    for (_i = 0, _a = Object.entries(args !== null && args !== void 0 ? args : {}); _i < _a.length; _i++) {
                        _b = _a[_i], argName = _b[0], argValue = _b[1];
                        if (argValue === false) {
                            continue;
                        }
                        argsArray.push("--" + toKebabCase(argName));
                        if (argValue === true) {
                            continue;
                        }
                        argsArray.push(typeof argValue === 'string' ? argValue : JSON.stringify(argValue));
                    }
                    return [4 /*yield*/, rust_cli_1.runOnRustCli(command, argsArray)];
                case 1: return [2 /*return*/, _c.sent()];
            }
        });
    });
}
var init = function (args) { return __awaiter(void 0, void 0, void 0, function () { return __generator(this, function (_a) {
    switch (_a.label) {
        case 0: return [4 /*yield*/, runCliCommand('init', args)];
        case 1: return [2 /*return*/, _a.sent()];
    }
}); }); };
exports.init = init;
var dev = function (args) { return __awaiter(void 0, void 0, void 0, function () { return __generator(this, function (_a) {
    switch (_a.label) {
        case 0: return [4 /*yield*/, runCliCommand('dev', args)];
        case 1: return [2 /*return*/, _a.sent()];
    }
}); }); };
exports.dev = dev;
var build = function (args) { return __awaiter(void 0, void 0, void 0, function () { return __generator(this, function (_a) {
    switch (_a.label) {
        case 0: return [4 /*yield*/, runCliCommand('build', args)];
        case 1: return [2 /*return*/, _a.sent()];
    }
}); }); };
exports.build = build;


/***/ }),

/***/ "./src/helpers/download-binary.ts":
/*!****************************************!*\
  !*** ./src/helpers/download-binary.ts ***!
  \****************************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {

"use strict";

var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.downloadRustup = exports.downloadCli = void 0;
var stream_1 = __importDefault(__webpack_require__(/*! stream */ "stream"));
var util_1 = __webpack_require__(/*! util */ "util");
var fs_1 = __importDefault(__webpack_require__(/*! fs */ "fs"));
var got_1 = __importDefault(__webpack_require__(/*! got */ "got"));
var path_1 = __importDefault(__webpack_require__(/*! path */ "path"));
var pipeline = util_1.promisify(stream_1.default.pipeline);
// Webpack reads the file at build-time, so this becomes a static var
// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-var-requires, @typescript-eslint/no-unsafe-member-access
var tauriCliManifest = __webpack_require__(/*! ../../../cli.rs/Cargo.toml */ "../cli.rs/Cargo.toml");
var downloads = {};
function downloadBinaryRelease(tag, asset, outPath) {
    return __awaiter(this, void 0, void 0, function () {
        var url, removeDownloadedCliIfNeeded;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    url = "https://github.com/tauri-apps/binary-releases/releases/download/" + tag + "/" + asset;
                    removeDownloadedCliIfNeeded = function () {
                        try {
                            if (!(url in downloads)) {
                                // eslint-disable-next-line security/detect-non-literal-fs-filename
                                fs_1.default.unlinkSync(outPath);
                            }
                        }
                        finally {
                            process.exit();
                        }
                    };
                    // on exit, we remove the `tauri-cli` file if the download didn't complete
                    process.on('exit', removeDownloadedCliIfNeeded);
                    process.on('SIGINT', removeDownloadedCliIfNeeded);
                    process.on('SIGTERM', removeDownloadedCliIfNeeded);
                    process.on('SIGHUP', removeDownloadedCliIfNeeded);
                    process.on('SIGBREAK', removeDownloadedCliIfNeeded);
                    // TODO: Check hash of download
                    // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access, security/detect-non-literal-fs-filename
                    return [4 /*yield*/, pipeline(got_1.default.stream(url), fs_1.default.createWriteStream(outPath)).catch(function (e) {
                            try {
                                // eslint-disable-next-line security/detect-non-literal-fs-filename
                                fs_1.default.unlinkSync(outPath);
                            }
                            catch (_a) { }
                            throw e;
                        })
                        // eslint-disable-next-line security/detect-object-injection
                    ];
                case 1:
                    // TODO: Check hash of download
                    // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access, security/detect-non-literal-fs-filename
                    _a.sent();
                    // eslint-disable-next-line security/detect-object-injection
                    downloads[url] = true;
                    // eslint-disable-next-line security/detect-non-literal-fs-filename
                    fs_1.default.chmodSync(outPath, 448);
                    console.log('Download Complete');
                    return [2 /*return*/];
            }
        });
    });
}
function downloadCli() {
    return __awaiter(this, void 0, void 0, function () {
        var version, platform, extension, outPath;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    version = tauriCliManifest.package.version;
                    platform = process.platform;
                    if (platform === 'win32') {
                        platform = 'windows';
                    }
                    else if (platform === 'linux') {
                        platform = 'linux';
                    }
                    else if (platform === 'darwin') {
                        platform = 'macos';
                    }
                    else {
                        throw Error('Unsupported platform');
                    }
                    extension = platform === 'windows' ? '.exe' : '';
                    outPath = path_1.default.join(__dirname, "../../bin/tauri-cli" + extension);
                    console.log('Downloading Rust CLI...');
                    return [4 /*yield*/, downloadBinaryRelease("tauri-cli-v" + version, "tauri-cli_" + platform + extension, outPath)];
                case 1:
                    _a.sent();
                    return [2 /*return*/];
            }
        });
    });
}
exports.downloadCli = downloadCli;
function downloadRustup() {
    return __awaiter(this, void 0, void 0, function () {
        var assetName;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    assetName = process.platform === 'win32' ? 'rustup-init.exe' : 'rustup-init.sh';
                    console.log('Downloading Rustup...');
                    return [4 /*yield*/, downloadBinaryRelease('rustup', assetName, path_1.default.join(__dirname, "../../bin/" + assetName))];
                case 1: return [2 /*return*/, _a.sent()];
            }
        });
    });
}
exports.downloadRustup = downloadRustup;


/***/ }),

/***/ "./src/helpers/logger.ts":
/*!*******************************!*\
  !*** ./src/helpers/logger.ts ***!
  \*******************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {

"use strict";

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

/***/ "./src/helpers/rust-cli.ts":
/*!*********************************!*\
  !*** ./src/helpers/rust-cli.ts ***!
  \*********************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {

"use strict";

// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var __spreadArray = (this && this.__spreadArray) || function (to, from) {
    for (var i = 0, il = from.length, j = to.length; i < il; i++, j++)
        to[j] = from[i];
    return to;
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.runOnRustCli = void 0;
var fs_1 = __webpack_require__(/*! fs */ "fs");
var path_1 = __webpack_require__(/*! path */ "path");
var spawn_1 = __webpack_require__(/*! ./spawn */ "./src/helpers/spawn.ts");
var download_binary_1 = __webpack_require__(/*! ./download-binary */ "./src/helpers/download-binary.ts");
var currentTauriCliVersion = function () {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
    var tauriCliManifest = 
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    __webpack_require__(/*! ../../../cli.rs/Cargo.toml */ "../cli.rs/Cargo.toml");
    return tauriCliManifest.package.version;
};
function runOnRustCli(command, args) {
    return __awaiter(this, void 0, void 0, function () {
        var targetPath, targetCliPath, resolveCb, rejectCb, pid, promise, onClose, cliPath, localCliPath;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    targetPath = path_1.resolve(__dirname, '../..');
                    targetCliPath = path_1.join(targetPath, 'bin/tauri-cli' + (process.platform === 'win32' ? '.exe' : ''));
                    promise = new Promise(function (resolve, reject) {
                        resolveCb = resolve;
                        rejectCb = function () { return reject(new Error()); };
                    });
                    onClose = function (code, pid) {
                        if (code === 0) {
                            resolveCb();
                        }
                        else {
                            rejectCb();
                        }
                    };
                    if (!fs_1.existsSync(targetCliPath)) return [3 /*break*/, 1];
                    pid = spawn_1.spawn(targetCliPath, __spreadArray(['tauri', command], args), process.cwd(), onClose);
                    return [3 /*break*/, 4];
                case 1:
                    if (true) return [3 /*break*/, 3];
                    return [4 /*yield*/, download_binary_1.downloadCli()];
                case 2:
                    _a.sent();
                    pid = spawn_1.spawn(targetCliPath, __spreadArray(['tauri', command], args), process.cwd(), onClose);
                    return [3 /*break*/, 4];
                case 3:
                    if (fs_1.existsSync(path_1.resolve(targetPath, '../bundler'))) {
                        cliPath = path_1.resolve(targetPath, '../cli.rs');
                        spawn_1.spawnSync('cargo', ['build', '--release'], cliPath);
                        localCliPath = path_1.resolve(targetPath, '../cli.rs/target/release/cargo-tauri');
                        pid = spawn_1.spawn(localCliPath, __spreadArray(['tauri', command], args), process.cwd(), onClose);
                    }
                    else {
                        spawn_1.spawnSync('cargo', [
                            'install',
                            '--root',
                            targetPath,
                            'tauri-cli',
                            '--version',
                            currentTauriCliVersion()
                        ], process.cwd());
                        pid = spawn_1.spawn(targetCliPath, __spreadArray(['tauri', command], args), process.cwd(), onClose);
                    }
                    _a.label = 4;
                case 4: return [2 /*return*/, { pid: pid, promise: promise }];
            }
        });
    });
}
exports.runOnRustCli = runOnRustCli;


/***/ }),

/***/ "./src/helpers/spawn.ts":
/*!******************************!*\
  !*** ./src/helpers/spawn.ts ***!
  \******************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {

"use strict";

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

"use strict";
module.exports = require("chalk");;

/***/ }),

/***/ "cross-spawn":
/*!******************************!*\
  !*** external "cross-spawn" ***!
  \******************************/
/***/ ((module) => {

"use strict";
module.exports = require("cross-spawn");;

/***/ }),

/***/ "fs":
/*!*********************!*\
  !*** external "fs" ***!
  \*********************/
/***/ ((module) => {

"use strict";
module.exports = require("fs");;

/***/ }),

/***/ "got":
/*!**********************!*\
  !*** external "got" ***!
  \**********************/
/***/ ((module) => {

"use strict";
module.exports = require("got");;

/***/ }),

/***/ "ms":
/*!*********************!*\
  !*** external "ms" ***!
  \*********************/
/***/ ((module) => {

"use strict";
module.exports = require("ms");;

/***/ }),

/***/ "path":
/*!***********************!*\
  !*** external "path" ***!
  \***********************/
/***/ ((module) => {

"use strict";
module.exports = require("path");;

/***/ }),

/***/ "stream":
/*!*************************!*\
  !*** external "stream" ***!
  \*************************/
/***/ ((module) => {

"use strict";
module.exports = require("stream");;

/***/ }),

/***/ "util":
/*!***********************!*\
  !*** external "util" ***!
  \***********************/
/***/ ((module) => {

"use strict";
module.exports = require("util");;

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
/******/ 	var __webpack_exports__ = __webpack_require__("./src/api/cli.ts");
/******/ 	
/******/ 	return __webpack_exports__;
/******/ })()
;
});
//# sourceMappingURL=cli.js.map