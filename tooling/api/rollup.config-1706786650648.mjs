import { defineConfig } from 'rollup';
import typescript from '@rollup/plugin-typescript';
import terser from '@rollup/plugin-terser';
import fg from 'fast-glob';
import { join, basename } from 'path';
import { copyFileSync, opendirSync, rmSync } from 'fs';
import { fileURLToPath } from 'url';

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// cleanup dist dir
const __dirname = fileURLToPath(new URL('.', 'file:///home/lucas/projects/tauri/tauri/tooling/api/rollup.config.ts'));
cleanDir(join(__dirname, './dist'));
const modules = fg.sync(['!./src/*.d.ts', './src/*.ts']);
var rollup_config = defineConfig([
    {
        input: Object.fromEntries(modules.map((p) => [basename(p, '.ts'), p])),
        output: [
            {
                format: 'esm',
                dir: './dist',
                preserveModules: true,
                preserveModulesRoot: 'src',
                entryFileNames: (chunkInfo) => {
                    if (chunkInfo.name.includes('node_modules')) {
                        return chunkInfo.name.replace('node_modules', 'external') + '.js';
                    }
                    return '[name].js';
                }
            },
            {
                format: 'cjs',
                dir: './dist',
                preserveModules: true,
                preserveModulesRoot: 'src',
                entryFileNames: (chunkInfo) => {
                    if (chunkInfo.name.includes('node_modules')) {
                        return chunkInfo.name.replace('node_modules', 'external') + '.cjs';
                    }
                    return '[name].cjs';
                }
            }
        ],
        plugins: [
            typescript({
                declaration: true,
                declarationDir: './dist',
                rootDir: 'src'
            }),
            makeFlatPackageInDist()
        ],
        onwarn
    },
    {
        input: 'src/index.ts',
        output: {
            format: 'iife',
            name: '__TAURI_IIFE__',
            footer: 'window.__TAURI__ = __TAURI_IIFE__',
            file: '../../core/tauri/scripts/bundle.global.js'
        },
        plugins: [typescript(), terser()],
        onwarn
    }
]);
function onwarn(warning) {
    // deny warnings by default
    throw Object.assign(new Error(), warning);
}
function makeFlatPackageInDist() {
    return {
        name: 'makeFlatPackageInDist',
        writeBundle() {
            // copy necessary files like `CHANGELOG.md` , `README.md` and Licenses to `./dist`
            fg.sync('(LICENSE*|*.md|package.json)').forEach((f) => copyFileSync(f, `dist/${f}`));
        }
    };
}
function cleanDir(path) {
    let dir;
    try {
        dir = opendirSync(path);
    }
    catch (err) {
        switch (err.code) {
            case 'ENOENT':
                return; // Noop when directory don't exists.
            case 'ENOTDIR':
                throw new Error(`'${path}' is not a directory.`);
            default:
                throw err;
        }
    }
    let file = dir.readSync();
    while (file) {
        const filePath = join(path, file.name);
        rmSync(filePath, { recursive: true });
        file = dir.readSync();
    }
    dir.closeSync();
}

export { rollup_config as default };
