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

/***/ "./src/api/dependency-manager/cargo-crates.ts":
/*!****************************************************!*\
  !*** ./src/api/dependency-manager/cargo-crates.ts ***!
  \****************************************************/
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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.update = exports.install = void 0;
var spawn_1 = __webpack_require__(/*! ./../../helpers/spawn */ "./src/helpers/spawn.ts");
var types_1 = __webpack_require__(/*! ./types */ "./src/api/dependency-manager/types.ts");
var util_1 = __webpack_require__(/*! ./util */ "./src/api/dependency-manager/util.ts");
var logger_1 = __importDefault(__webpack_require__(/*! ../../helpers/logger */ "./src/helpers/logger.ts"));
var app_paths_1 = __webpack_require__(/*! ../../helpers/app-paths */ "./src/helpers/app-paths.ts");
var fs_1 = __webpack_require__(/*! fs */ "fs");
var toml_1 = __importDefault(__webpack_require__(/*! @tauri-apps/toml */ "@tauri-apps/toml"));
var inquirer_1 = __importDefault(__webpack_require__(/*! inquirer */ "inquirer"));
var log = logger_1.default('dependency:crates');
var dependencies = ['tauri'];
function readToml(tomlPath) {
    if (fs_1.existsSync(tomlPath)) {
        var manifest = fs_1.readFileSync(tomlPath).toString();
        return toml_1.default.parse(manifest);
    }
    return null;
}
function dependencyDefinition(version) {
    return { version: version.substring(0, version.lastIndexOf('.')) };
}
function manageDependencies(managementType) {
    return __awaiter(this, void 0, void 0, function () {
        var installedDeps, updatedDeps, result, manifest, lockPath, lock, _loop_1, _i, dependencies_1, dependency;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    installedDeps = [];
                    updatedDeps = [];
                    result = new Map();
                    manifest = readToml(app_paths_1.resolve.tauri('Cargo.toml'));
                    if (manifest === null) {
                        log('Cargo.toml not found. Skipping crates check...');
                        return [2 /*return*/, result];
                    }
                    lockPath = app_paths_1.resolve.tauri('Cargo.lock');
                    if (!fs_1.existsSync(lockPath)) {
                        spawn_1.spawnSync('cargo', ['generate-lockfile'], app_paths_1.tauriDir);
                    }
                    lock = readToml(lockPath);
                    _loop_1 = function (dependency) {
                        var lockPackages, manifestDep, currentVersion, latestVersion, latestVersion, inquired;
                        return __generator(this, function (_b) {
                            switch (_b.label) {
                                case 0:
                                    lockPackages = lock
                                        ? lock.package.filter(function (pkg) { return pkg.name === dependency; })
                                        : [];
                                    manifestDep = manifest.dependencies[dependency];
                                    currentVersion = lockPackages.length === 1
                                        ? lockPackages[0].version
                                        : typeof manifestDep === 'string'
                                            ? manifestDep
                                            : manifestDep === null || manifestDep === void 0 ? void 0 : manifestDep.version;
                                    if (!(currentVersion === undefined)) return [3 /*break*/, 1];
                                    log("Installing " + dependency + "...");
                                    latestVersion = util_1.getCrateLatestVersion(dependency);
                                    if (latestVersion !== null) {
                                        // eslint-disable-next-line security/detect-object-injection
                                        manifest.dependencies[dependency] = dependencyDefinition(latestVersion);
                                    }
                                    installedDeps.push(dependency);
                                    return [3 /*break*/, 6];
                                case 1:
                                    if (!(managementType === types_1.ManagementType.Update)) return [3 /*break*/, 5];
                                    latestVersion = util_1.getCrateLatestVersion(dependency);
                                    if (!(latestVersion !== null && util_1.semverLt(currentVersion, latestVersion))) return [3 /*break*/, 3];
                                    return [4 /*yield*/, inquirer_1.default.prompt([
                                            {
                                                type: 'confirm',
                                                name: 'answer',
                                                message: "[CRATES] \"" + dependency + "\" latest version is " + latestVersion + ". Do you want to update?",
                                                default: false
                                            }
                                        ])];
                                case 2:
                                    inquired = (_b.sent());
                                    if (inquired.answer) {
                                        log("Updating " + dependency + "...");
                                        // eslint-disable-next-line security/detect-object-injection
                                        manifest.dependencies[dependency] =
                                            dependencyDefinition(latestVersion);
                                        updatedDeps.push(dependency);
                                    }
                                    return [3 /*break*/, 4];
                                case 3:
                                    log("\"" + dependency + "\" is up to date");
                                    _b.label = 4;
                                case 4: return [3 /*break*/, 6];
                                case 5:
                                    log("\"" + dependency + "\" is already installed");
                                    _b.label = 6;
                                case 6: return [2 /*return*/];
                            }
                        });
                    };
                    _i = 0, dependencies_1 = dependencies;
                    _a.label = 1;
                case 1:
                    if (!(_i < dependencies_1.length)) return [3 /*break*/, 4];
                    dependency = dependencies_1[_i];
                    return [5 /*yield**/, _loop_1(dependency)];
                case 2:
                    _a.sent();
                    _a.label = 3;
                case 3:
                    _i++;
                    return [3 /*break*/, 1];
                case 4:
                    if (installedDeps.length || updatedDeps.length) {
                        fs_1.writeFileSync(app_paths_1.resolve.tauri('Cargo.toml'), toml_1.default.stringify(manifest));
                    }
                    if (updatedDeps.length) {
                        if (!fs_1.existsSync(app_paths_1.resolve.tauri('Cargo.lock'))) {
                            spawn_1.spawnSync('cargo', ['generate-lockfile'], app_paths_1.tauriDir);
                        }
                        spawn_1.spawnSync('cargo', __spreadArray([
                            'update',
                            '--aggressive'
                        ], updatedDeps.reduce(function (initialValue, dep) { return __spreadArray(__spreadArray([], initialValue), ['-p', dep]); }, [])), app_paths_1.tauriDir);
                    }
                    result.set(types_1.ManagementType.Install, installedDeps);
                    result.set(types_1.ManagementType.Update, updatedDeps);
                    return [2 /*return*/, result];
            }
        });
    });
}
function install() {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, manageDependencies(types_1.ManagementType.Install)];
                case 1: return [2 /*return*/, _a.sent()];
            }
        });
    });
}
exports.install = install;
function update() {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, manageDependencies(types_1.ManagementType.Update)];
                case 1: return [2 /*return*/, _a.sent()];
            }
        });
    });
}
exports.update = update;


/***/ }),

/***/ "./src/api/dependency-manager/index.ts":
/*!*********************************************!*\
  !*** ./src/api/dependency-manager/index.ts ***!
  \*********************************************/
/***/ (function(module, exports, __webpack_require__) {

"use strict";

// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
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
var logger_1 = __importDefault(__webpack_require__(/*! ../../helpers/logger */ "./src/helpers/logger.ts"));
var rust = __importStar(__webpack_require__(/*! ./rust */ "./src/api/dependency-manager/rust.ts"));
var cargoCrates = __importStar(__webpack_require__(/*! ./cargo-crates */ "./src/api/dependency-manager/cargo-crates.ts"));
var npmPackages = __importStar(__webpack_require__(/*! ./npm-packages */ "./src/api/dependency-manager/npm-packages.ts"));
var log = logger_1.default('dependency:manager');
module.exports = {
    installDependencies: function () {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        log('Installing missing dependencies...');
                        return [4 /*yield*/, rust.install()];
                    case 1:
                        _a.sent();
                        return [4 /*yield*/, cargoCrates.install()];
                    case 2:
                        _a.sent();
                        return [4 /*yield*/, npmPackages.install()];
                    case 3:
                        _a.sent();
                        return [2 /*return*/];
                }
            });
        });
    },
    updateDependencies: function () {
        return __awaiter(this, void 0, void 0, function () {
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        log('Updating dependencies...');
                        return [4 /*yield*/, rust.update()];
                    case 1:
                        _a.sent();
                        return [4 /*yield*/, cargoCrates.update()];
                    case 2:
                        _a.sent();
                        return [4 /*yield*/, npmPackages.update()];
                    case 3:
                        _a.sent();
                        return [2 /*return*/];
                }
            });
        });
    }
};


/***/ }),

/***/ "./src/api/dependency-manager/managers/index.ts":
/*!******************************************************!*\
  !*** ./src/api/dependency-manager/managers/index.ts ***!
  \******************************************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {

"use strict";

var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
__exportStar(__webpack_require__(/*! ./yarn-manager */ "./src/api/dependency-manager/managers/yarn-manager.ts"), exports);
__exportStar(__webpack_require__(/*! ./npm-manager */ "./src/api/dependency-manager/managers/npm-manager.ts"), exports);
__exportStar(__webpack_require__(/*! ./pnpm-manager */ "./src/api/dependency-manager/managers/pnpm-manager.ts"), exports);
__exportStar(__webpack_require__(/*! ./types */ "./src/api/dependency-manager/managers/types.ts"), exports);


/***/ }),

/***/ "./src/api/dependency-manager/managers/npm-manager.ts":
/*!************************************************************!*\
  !*** ./src/api/dependency-manager/managers/npm-manager.ts ***!
  \************************************************************/
/***/ ((__unused_webpack_module, exports, __webpack_require__) => {

"use strict";

Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.NpmManager = void 0;
var cross_spawn_1 = __webpack_require__(/*! cross-spawn */ "cross-spawn");
var spawn_1 = __webpack_require__(/*! ../../../helpers/spawn */ "./src/helpers/spawn.ts");
var app_paths_1 = __webpack_require__(/*! ../../../helpers/app-paths */ "./src/helpers/app-paths.ts");
var NpmManager = /** @class */ (function () {
    function NpmManager() {
        this.type = 'npm';
    }
    NpmManager.prototype.installPackage = function (packageName) {
        spawn_1.spawnSync('npm', ['install', packageName], app_paths_1.appDir);
    };
    NpmManager.prototype.installDevPackage = function (packageName) {
        spawn_1.spawnSync('npm', ['install', packageName, '--save-dev'], app_paths_1.appDir);
    };
    NpmManager.prototype.updatePackage = function (packageName) {
        spawn_1.spawnSync('npm', ['install', packageName + "@latest"], app_paths_1.appDir);
    };
    NpmManager.prototype.getPackageVersion = function (packageName) {
        var child = cross_spawn_1.sync('npm', ['list', packageName, 'version', '--depth', '0'], {
            cwd: app_paths_1.appDir
        });
        var output = String(child.output[1]);
        // eslint-disable-next-line security/detect-non-literal-regexp
        var matches = new RegExp(packageName + '@(\\S+)', 'g').exec(output);
        if (matches === null || matches === void 0 ? void 0 : matches[1]) {
            return matches[1];
        }
        else {
            return null;
        }
    };
    NpmManager.prototype.getLatestVersion = function (packageName) {
        var child = cross_spawn_1.sync('npm', ['show', packageName, 'version'], {
            cwd: app_paths_1.appDir
        });
        return String(child.output[1]).replace('\n', '');
    };
    return NpmManager;
}());
exports.NpmManager = NpmManager;


/***/ }),

/***/ "./src/api/dependency-manager/managers/pnpm-manager.ts":
/*!*************************************************************!*\
  !*** ./src/api/dependency-manager/managers/pnpm-manager.ts ***!
  \*************************************************************/
/***/ ((__unused_webpack_module, exports, __webpack_require__) => {

"use strict";

Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.PnpmManager = void 0;
var cross_spawn_1 = __webpack_require__(/*! cross-spawn */ "cross-spawn");
var spawn_1 = __webpack_require__(/*! ../../../helpers/spawn */ "./src/helpers/spawn.ts");
var app_paths_1 = __webpack_require__(/*! ../../../helpers/app-paths */ "./src/helpers/app-paths.ts");
var PnpmManager = /** @class */ (function () {
    function PnpmManager() {
        this.type = 'pnpm';
    }
    PnpmManager.prototype.installPackage = function (packageName) {
        spawn_1.spawnSync('pnpm', ['add', packageName], app_paths_1.appDir);
    };
    PnpmManager.prototype.installDevPackage = function (packageName) {
        spawn_1.spawnSync('pnpm', ['add', packageName, '--save-dev'], app_paths_1.appDir);
    };
    PnpmManager.prototype.updatePackage = function (packageName) {
        spawn_1.spawnSync('pnpm', ['add', packageName + "@latest"], app_paths_1.appDir);
    };
    PnpmManager.prototype.getPackageVersion = function (packageName) {
        var child = cross_spawn_1.sync('pnpm', ['list', packageName, 'version', '--depth', '0'], {
            cwd: app_paths_1.appDir
        });
        var output = String(child.output[1]);
        // eslint-disable-next-line security/detect-non-literal-regexp
        var matches = new RegExp(packageName + ' (\\S+)', 'g').exec(output);
        if (matches === null || matches === void 0 ? void 0 : matches[1]) {
            return matches[1];
        }
        else {
            return null;
        }
    };
    PnpmManager.prototype.getLatestVersion = function (packageName) {
        var child = cross_spawn_1.sync('pnpm', ['info', packageName, 'version'], {
            cwd: app_paths_1.appDir
        });
        return String(child.output[1]).replace('\n', '');
    };
    return PnpmManager;
}());
exports.PnpmManager = PnpmManager;


/***/ }),

/***/ "./src/api/dependency-manager/managers/types.ts":
/*!******************************************************!*\
  !*** ./src/api/dependency-manager/managers/types.ts ***!
  \******************************************************/
/***/ ((__unused_webpack_module, exports) => {

"use strict";

Object.defineProperty(exports, "__esModule", ({ value: true }));


/***/ }),

/***/ "./src/api/dependency-manager/managers/yarn-manager.ts":
/*!*************************************************************!*\
  !*** ./src/api/dependency-manager/managers/yarn-manager.ts ***!
  \*************************************************************/
/***/ ((__unused_webpack_module, exports, __webpack_require__) => {

"use strict";

Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.YarnManager = void 0;
var cross_spawn_1 = __webpack_require__(/*! cross-spawn */ "cross-spawn");
var spawn_1 = __webpack_require__(/*! ../../../helpers/spawn */ "./src/helpers/spawn.ts");
var app_paths_1 = __webpack_require__(/*! ../../../helpers/app-paths */ "./src/helpers/app-paths.ts");
var YarnManager = /** @class */ (function () {
    function YarnManager() {
        this.type = 'yarn';
    }
    YarnManager.prototype.installPackage = function (packageName) {
        spawn_1.spawnSync('yarn', ['add', packageName], app_paths_1.appDir);
    };
    YarnManager.prototype.installDevPackage = function (packageName) {
        spawn_1.spawnSync('yarn', ['add', packageName, '--dev'], app_paths_1.appDir);
    };
    YarnManager.prototype.updatePackage = function (packageName) {
        spawn_1.spawnSync('yarn', ['upgrade', packageName, '--latest'], app_paths_1.appDir);
    };
    YarnManager.prototype.getPackageVersion = function (packageName) {
        var child = cross_spawn_1.sync('yarn', ['list', '--pattern', packageName, '--depth', '0'], { cwd: app_paths_1.appDir });
        var output = String(child.output[1]);
        // eslint-disable-next-line security/detect-non-literal-regexp
        var matches = new RegExp(packageName + '@(\\S+)', 'g').exec(output);
        if (matches === null || matches === void 0 ? void 0 : matches[1]) {
            return matches[1];
        }
        else {
            return null;
        }
    };
    YarnManager.prototype.getLatestVersion = function (packageName) {
        var child = cross_spawn_1.sync('yarn', ['info', packageName, 'version', '--json'], { cwd: app_paths_1.appDir });
        var output = String(child.output[1]);
        var packageJson = JSON.parse(output);
        return packageJson.data;
    };
    return YarnManager;
}());
exports.YarnManager = YarnManager;


/***/ }),

/***/ "./src/api/dependency-manager/npm-packages.ts":
/*!****************************************************!*\
  !*** ./src/api/dependency-manager/npm-packages.ts ***!
  \****************************************************/
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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.update = exports.installTheseDev = exports.installThese = exports.install = void 0;
var types_1 = __webpack_require__(/*! ./types */ "./src/api/dependency-manager/types.ts");
var util_1 = __webpack_require__(/*! ./util */ "./src/api/dependency-manager/util.ts");
var logger_1 = __importDefault(__webpack_require__(/*! ../../helpers/logger */ "./src/helpers/logger.ts"));
var app_paths_1 = __webpack_require__(/*! ../../helpers/app-paths */ "./src/helpers/app-paths.ts");
var inquirer_1 = __importDefault(__webpack_require__(/*! inquirer */ "inquirer"));
var fs_1 = __webpack_require__(/*! fs */ "fs");
var cross_spawn_1 = __webpack_require__(/*! cross-spawn */ "cross-spawn");
var log = logger_1.default('dependency:npm-packages');
function manageDependencies(managementType, dependencies) {
    var _a, _b, _c;
    return __awaiter(this, void 0, void 0, function () {
        var installedDeps, updatedDeps, npmChild, yarnChild, pnpmChild, _i, dependencies_1, dependency, currentVersion, packageManager, prefix, inquired, latestVersion, inquired, result;
        return __generator(this, function (_d) {
            switch (_d.label) {
                case 0:
                    installedDeps = [];
                    updatedDeps = [];
                    npmChild = cross_spawn_1.sync('npm', ['--version']);
                    yarnChild = cross_spawn_1.sync('yarn', ['--version']);
                    pnpmChild = cross_spawn_1.sync('pnpm', ['--version']);
                    if (((_a = npmChild.status) !== null && _a !== void 0 ? _a : npmChild.error) &&
                        ((_b = yarnChild.status) !== null && _b !== void 0 ? _b : yarnChild.error) &&
                        ((_c = pnpmChild.status) !== null && _c !== void 0 ? _c : pnpmChild.error)) {
                        throw new Error('must have installed one of the following package managers `npm`, `yarn`, `pnpm` to manage dependenices');
                    }
                    if (!fs_1.existsSync(app_paths_1.resolve.app('package.json'))) return [3 /*break*/, 10];
                    _i = 0, dependencies_1 = dependencies;
                    _d.label = 1;
                case 1:
                    if (!(_i < dependencies_1.length)) return [3 /*break*/, 10];
                    dependency = dependencies_1[_i];
                    currentVersion = util_1.getNpmPackageVersion(dependency);
                    packageManager = util_1.getManager().type.toUpperCase();
                    if (!(currentVersion === null)) return [3 /*break*/, 4];
                    log("Installing " + dependency + "...");
                    if (!(managementType === types_1.ManagementType.Install ||
                        managementType === types_1.ManagementType.InstallDev)) return [3 /*break*/, 3];
                    prefix = managementType === types_1.ManagementType.InstallDev
                        ? ' as dev-dependency'
                        : '';
                    return [4 /*yield*/, inquirer_1.default.prompt([
                            {
                                type: 'confirm',
                                name: 'answer',
                                message: "[" + packageManager + "]: \"Do you want to install " + dependency + prefix + "?\"",
                                default: false
                            }
                        ])];
                case 2:
                    inquired = _d.sent();
                    if (inquired.answer) {
                        if (managementType === types_1.ManagementType.Install) {
                            util_1.installNpmPackage(dependency);
                        }
                        else if (managementType === types_1.ManagementType.InstallDev) {
                            util_1.installNpmDevPackage(dependency);
                        }
                        installedDeps.push(dependency);
                    }
                    _d.label = 3;
                case 3: return [3 /*break*/, 9];
                case 4:
                    if (!(managementType === types_1.ManagementType.Update)) return [3 /*break*/, 8];
                    latestVersion = util_1.getNpmLatestVersion(dependency);
                    if (!util_1.semverLt(currentVersion, latestVersion)) return [3 /*break*/, 6];
                    return [4 /*yield*/, inquirer_1.default.prompt([
                            {
                                type: 'confirm',
                                name: 'answer',
                                message: "[" + packageManager + "]: \"" + dependency + "\" latest version is " + latestVersion + ". Do you want to update?",
                                default: false
                            }
                        ])];
                case 5:
                    inquired = _d.sent();
                    if (inquired.answer) {
                        log("Updating " + dependency + "...");
                        util_1.updateNpmPackage(dependency);
                        updatedDeps.push(dependency);
                    }
                    return [3 /*break*/, 7];
                case 6:
                    log("\"" + dependency + "\" is up to date");
                    _d.label = 7;
                case 7: return [3 /*break*/, 9];
                case 8:
                    log("\"" + dependency + "\" is already installed");
                    _d.label = 9;
                case 9:
                    _i++;
                    return [3 /*break*/, 1];
                case 10:
                    result = new Map();
                    result.set(types_1.ManagementType.Install, installedDeps);
                    result.set(types_1.ManagementType.Update, updatedDeps);
                    return [2 /*return*/, result];
            }
        });
    });
}
var dependencies = ['@tauri-apps/api', '@tauri-apps/cli'];
function install() {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, manageDependencies(types_1.ManagementType.Install, dependencies)];
                case 1: return [2 /*return*/, _a.sent()];
            }
        });
    });
}
exports.install = install;
function installThese(dependencies) {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, manageDependencies(types_1.ManagementType.Install, dependencies)];
                case 1: return [2 /*return*/, _a.sent()];
            }
        });
    });
}
exports.installThese = installThese;
function installTheseDev(dependencies) {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, manageDependencies(types_1.ManagementType.InstallDev, dependencies)];
                case 1: return [2 /*return*/, _a.sent()];
            }
        });
    });
}
exports.installTheseDev = installTheseDev;
function update() {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, manageDependencies(types_1.ManagementType.Update, dependencies)];
                case 1: return [2 /*return*/, _a.sent()];
            }
        });
    });
}
exports.update = update;


/***/ }),

/***/ "./src/api/dependency-manager/rust.ts":
/*!********************************************!*\
  !*** ./src/api/dependency-manager/rust.ts ***!
  \********************************************/
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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.update = exports.install = void 0;
var types_1 = __webpack_require__(/*! ./types */ "./src/api/dependency-manager/types.ts");
var spawn_1 = __webpack_require__(/*! ../../helpers/spawn */ "./src/helpers/spawn.ts");
var get_script_version_1 = __importDefault(__webpack_require__(/*! ../../helpers/get-script-version */ "./src/helpers/get-script-version.ts"));
var download_binary_1 = __webpack_require__(/*! ../../helpers/download-binary */ "./src/helpers/download-binary.ts");
var logger_1 = __importDefault(__webpack_require__(/*! ../../helpers/logger */ "./src/helpers/logger.ts"));
var fs_1 = __webpack_require__(/*! fs */ "fs");
var path_1 = __webpack_require__(/*! path */ "path");
var os_1 = __webpack_require__(/*! os */ "os");
var https_1 = __importDefault(__webpack_require__(/*! https */ "https"));
var log = logger_1.default('dependency:rust');
// eslint-disable-next-line @typescript-eslint/no-unused-vars
function download(url, dest) {
    return __awaiter(this, void 0, void 0, function () {
        var file;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    file = fs_1.createWriteStream(dest);
                    return [4 /*yield*/, new Promise(function (resolve, reject) {
                            https_1.default
                                .get(url, function (response) {
                                response.pipe(file);
                                file.on('finish', function () {
                                    file.close();
                                    resolve();
                                });
                            })
                                .on('error', function (err) {
                                fs_1.unlinkSync(dest);
                                reject(err.message);
                            });
                        })];
                case 1: return [2 /*return*/, _a.sent()];
            }
        });
    });
}
function installRustup() {
    return __awaiter(this, void 0, void 0, function () {
        var assetName, rustupPath;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    assetName = os_1.platform() === 'win32' ? 'rustup-init.exe' : 'rustup-init.sh';
                    rustupPath = path_1.resolve(__dirname, "../../bin/" + assetName);
                    if (!!fs_1.existsSync(rustupPath)) return [3 /*break*/, 2];
                    return [4 /*yield*/, download_binary_1.downloadRustup()];
                case 1:
                    _a.sent();
                    _a.label = 2;
                case 2:
                    if (os_1.platform() === 'win32') {
                        return [2 /*return*/, spawn_1.spawnSync('powershell', ['-NoProfile', rustupPath], process.cwd())];
                    }
                    return [2 /*return*/, spawn_1.spawnSync('/bin/sh', [rustupPath], process.cwd())];
            }
        });
    });
}
function manageDependencies(managementType) {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    if (!(get_script_version_1.default('rustup') === null)) return [3 /*break*/, 2];
                    log('Installing rustup...');
                    return [4 /*yield*/, installRustup()];
                case 1:
                    _a.sent();
                    _a.label = 2;
                case 2:
                    if (managementType === types_1.ManagementType.Update) {
                        spawn_1.spawnSync('rustup', ['update'], process.cwd());
                    }
                    return [2 /*return*/];
            }
        });
    });
}
function install() {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, manageDependencies(types_1.ManagementType.Install)];
                case 1: return [2 /*return*/, _a.sent()];
            }
        });
    });
}
exports.install = install;
function update() {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, manageDependencies(types_1.ManagementType.Update)];
                case 1: return [2 /*return*/, _a.sent()];
            }
        });
    });
}
exports.update = update;


/***/ }),

/***/ "./src/api/dependency-manager/types.ts":
/*!*********************************************!*\
  !*** ./src/api/dependency-manager/types.ts ***!
  \*********************************************/
/***/ ((__unused_webpack_module, exports) => {

"use strict";

// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.ManagementType = void 0;
var ManagementType;
(function (ManagementType) {
    ManagementType[ManagementType["Install"] = 0] = "Install";
    ManagementType[ManagementType["InstallDev"] = 1] = "InstallDev";
    ManagementType[ManagementType["Update"] = 2] = "Update";
})(ManagementType = exports.ManagementType || (exports.ManagementType = {}));


/***/ }),

/***/ "./src/api/dependency-manager/util.ts":
/*!********************************************!*\
  !*** ./src/api/dependency-manager/util.ts ***!
  \********************************************/
/***/ ((__unused_webpack_module, exports, __webpack_require__) => {

"use strict";

// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.semverLt = exports.padVersion = exports.updateNpmPackage = exports.installNpmDevPackage = exports.installNpmPackage = exports.getNpmPackageVersion = exports.getNpmLatestVersion = exports.getCrateLatestVersion = exports.getManager = void 0;
var cross_spawn_1 = __webpack_require__(/*! cross-spawn */ "cross-spawn");
var app_paths_1 = __webpack_require__(/*! ../../helpers/app-paths */ "./src/helpers/app-paths.ts");
var fs_1 = __webpack_require__(/*! fs */ "fs");
// import semver from 'semver'
var managers_1 = __webpack_require__(/*! ./managers */ "./src/api/dependency-manager/managers/index.ts");
var getManager = function () {
    if (fs_1.existsSync(app_paths_1.resolve.app('yarn.lock'))) {
        return new managers_1.YarnManager();
    }
    else if (fs_1.existsSync(app_paths_1.resolve.app('pnpm-lock.yaml'))) {
        return new managers_1.PnpmManager();
    }
    else {
        return new managers_1.NpmManager();
    }
};
exports.getManager = getManager;
function getCrateLatestVersion(crateName) {
    var child = cross_spawn_1.sync('cargo', ['search', crateName, '--limit', '1']);
    var output = String(child.output[1]);
    // eslint-disable-next-line security/detect-non-literal-regexp
    var matches = new RegExp(crateName + ' = "(\\S+)"', 'g').exec(output);
    if (matches === null || matches === void 0 ? void 0 : matches[1]) {
        return matches[1];
    }
    else {
        return null;
    }
}
exports.getCrateLatestVersion = getCrateLatestVersion;
function getNpmLatestVersion(packageName) {
    return getManager().getLatestVersion(packageName);
}
exports.getNpmLatestVersion = getNpmLatestVersion;
function getNpmPackageVersion(packageName) {
    return getManager().getPackageVersion(packageName);
}
exports.getNpmPackageVersion = getNpmPackageVersion;
function installNpmPackage(packageName) {
    return getManager().installPackage(packageName);
}
exports.installNpmPackage = installNpmPackage;
function installNpmDevPackage(packageName) {
    return getManager().installDevPackage(packageName);
}
exports.installNpmDevPackage = installNpmDevPackage;
function updateNpmPackage(packageName) {
    return getManager().updatePackage(packageName);
}
exports.updateNpmPackage = updateNpmPackage;
function padVersion(version) {
    var _a;
    var count = ((_a = version.match(/\./g)) !== null && _a !== void 0 ? _a : []).length;
    while (count < 2) {
        count++;
        version += '.0';
    }
    return version;
}
exports.padVersion = padVersion;
function semverLt(first, second) {
    return first !== second;
    // TODO: When version 1.0.0 is released this code should work again
    // return semver.lt(padVersion(first), padVersion(second))
}
exports.semverLt = semverLt;


/***/ }),

/***/ "./src/helpers/app-paths.ts":
/*!**********************************!*\
  !*** ./src/helpers/app-paths.ts ***!
  \**********************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {

"use strict";

// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.resolve = exports.tauriDir = exports.appDir = void 0;
var fs_1 = __webpack_require__(/*! fs */ "fs");
var path_1 = __webpack_require__(/*! path */ "path");
var logger_1 = __importDefault(__webpack_require__(/*! ./logger */ "./src/helpers/logger.ts"));
var chalk_1 = __importDefault(__webpack_require__(/*! chalk */ "chalk"));
var warn = logger_1.default('tauri', chalk_1.default.red);
function resolvePath(basePath, dir) {
    return dir && path_1.isAbsolute(dir) ? dir : path_1.resolve(basePath, dir);
}
var getAppDir = function () {
    var dir = process.cwd();
    var count = 0;
    // only go up three folders max
    while (dir.length > 0 && !dir.endsWith(path_1.sep) && count <= 2) {
        if (fs_1.existsSync(path_1.join(dir, 'src-tauri', 'tauri.conf.json'))) {
            return dir;
        }
        count++;
        dir = path_1.normalize(path_1.join(dir, '..'));
    }
    warn("Couldn't find recognize the current folder as a part of a Tauri project");
    process.exit(1);
};
var appDir = getAppDir();
exports.appDir = appDir;
var tauriDir = path_1.resolve(appDir, 'src-tauri');
exports.tauriDir = tauriDir;
var resolveDir = {
    app: function (dir) { return resolvePath(appDir, dir); },
    tauri: function (dir) { return resolvePath(tauriDir, dir); }
};
exports.resolve = resolveDir;


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

/***/ "./src/helpers/get-script-version.ts":
/*!*******************************************!*\
  !*** ./src/helpers/get-script-version.ts ***!
  \*******************************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {

"use strict";

// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
var __spreadArray = (this && this.__spreadArray) || function (to, from) {
    for (var i = 0, il = from.length, j = to.length; i < il; i++, j++)
        to[j] = from[i];
    return to;
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
var cross_spawn_1 = __webpack_require__(/*! cross-spawn */ "cross-spawn");
function getVersion(command, args) {
    if (args === void 0) { args = []; }
    try {
        var child = cross_spawn_1.sync(command, __spreadArray(__spreadArray([], args), ['--version']));
        if (child.status === 0) {
            var output = String(child.output[1]);
            return output.replace(/\n/g, '');
        }
        return null;
    }
    catch (err) {
        return null;
    }
}
exports.default = getVersion;


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

/***/ "@tauri-apps/toml":
/*!***********************************!*\
  !*** external "@tauri-apps/toml" ***!
  \***********************************/
/***/ ((module) => {

"use strict";
module.exports = require("@tauri-apps/toml");;

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

/***/ "https":
/*!************************!*\
  !*** external "https" ***!
  \************************/
/***/ ((module) => {

"use strict";
module.exports = require("https");;

/***/ }),

/***/ "inquirer":
/*!***************************!*\
  !*** external "inquirer" ***!
  \***************************/
/***/ ((module) => {

"use strict";
module.exports = require("inquirer");;

/***/ }),

/***/ "ms":
/*!*********************!*\
  !*** external "ms" ***!
  \*********************/
/***/ ((module) => {

"use strict";
module.exports = require("ms");;

/***/ }),

/***/ "os":
/*!*********************!*\
  !*** external "os" ***!
  \*********************/
/***/ ((module) => {

"use strict";
module.exports = require("os");;

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
/******/ 	var __webpack_exports__ = __webpack_require__("./src/api/dependency-manager/index.ts");
/******/ 	
/******/ 	return __webpack_exports__;
/******/ })()
;
});
//# sourceMappingURL=dependency-manager.js.map