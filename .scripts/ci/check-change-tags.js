#!/usr/bin/env node

// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const fs = require('fs')
const path = require('path')
const { execFileSync } = require('child_process')
const ignorePackages = [
  'tauri-macros',
  'tauri-codegen',
  'tauri-runtime',
  'tauri-runtime-wry',
  'tauri-driver'
]

const covectorConfig = JSON.parse(
  fs.readFileSync('.changes/config.json', 'utf8')
)
const tags = Object.keys(covectorConfig.changeTags)

const missingTagsFiles = {}
const unknownTagsFiles = {}

function checkChangeFiles(changeFiles) {
  for (const file of changeFiles) {
    const content = fs.readFileSync(file, 'utf8')
    const [frontMatter] = /^---[\s\S.]*---\n/i.exec(content)
    const packages = frontMatter
      .split('\n')
      .filter((l) => !(l === '---' || !l))
      .map((l) => l.replace(/('|")/g, '').split(':'))

    for (const [package, _, tag] of packages) {
      if (!tag) {
        if (ignorePackages.includes(package)) continue

        if (!missingTagsFiles[file]) missingTagsFiles[file] = []
        missingTagsFiles[file].push(package)
      } else if (!tags.includes(tag)) {
        if (!unknownTagsFiles[file]) unknownTagsFiles[file] = []
        unknownTagsFiles[file].push({ package, tag })
      }
    }
  }
  const missingTagsEntries = Object.entries(missingTagsFiles)
  const unknownTagsEntries = Object.entries(unknownTagsFiles)
  if (missingTagsEntries.length > 0 || unknownTagsEntries.length > 0) {
    for (const [file, packages] of missingTagsEntries) {
      for (const package of packages) {
        console.error(
          `Package \`${package}\` is missing a change tag in ${file} `
        )
      }
    }

    for (const [file, packages] of unknownTagsEntries) {
      for (const { package, tag } of packages) {
        console.error(
          `Package \`${package}\` has an unknown change tag ${tag} in ${file} `
        )
      }
    }

    process.exit(1)
  }
}

// in prerelease mode covector lists the consumed change files in pre.json
// instead of deleting them
const releasedChangeFiles = (() => {
  const preFile = '.changes/pre.json'
  if (!fs.existsSync(preFile)) return new Set()
  const changes = JSON.parse(fs.readFileSync(preFile, 'utf8')).changes || []
  return new Set(changes.map((f) => path.posix.normalize(f)))
})()

function isPendingChangeFile(file) {
  return (
    !file.toLowerCase().endsWith('readme.md')
    && !releasedChangeFiles.has(path.posix.normalize(file))
  )
}

function pendingChangeFiles() {
  return fs
    .readdirSync('.changes')
    .filter((f) => f.endsWith('.md'))
    .map((p) => path.posix.join('.changes', p))
    .filter(isPendingChangeFile)
}

function git(...args) {
  return execFileSync('git', args, { encoding: 'utf8' }).trim()
}

function changeFilePackages(file) {
  const content = fs.readFileSync(file, 'utf8')
  const match = /^---[\s\S.]*?---\n/i.exec(content)
  if (!match) return []
  return match[0]
    .split('\n')
    .filter((l) => !(l === '---' || !l))
    .map((l) => l.replace(/('|")/g, '').split(':')[0].trim())
}

function packageVersion(pkg) {
  if (pkg.manager === 'javascript') {
    return JSON.parse(
      fs.readFileSync(path.join(pkg.path, 'package.json'), 'utf8')
    ).version
  }
  const manifest = fs.readFileSync(path.join(pkg.path, 'Cargo.toml'), 'utf8')
  return /^version\s*=\s*"([^"]+)"/m.exec(manifest)?.[1]
}

// files that changing does not warrant a release
function isIgnoredFile(file) {
  return file.endsWith('CHANGELOG.md') || /(^|\/)(tests|benches)\//.test(file)
}

function changedFiles(from, pkgPath) {
  const out = git('diff', '--name-only', from, '--', pkgPath)
  return out ? out.split('\n').filter((f) => !isIgnoredFile(f)) : []
}

// covector bumps a package when one of its dependencies is bumped
function withDependents(packages, bumped) {
  const covered = new Set(bumped)
  let added = true
  while (added) {
    added = false
    for (const [name, pkg] of Object.entries(packages)) {
      if (
        !covered.has(name)
        && (pkg.dependencies || []).some((d) => covered.has(d))
      ) {
        covered.add(name)
        added = true
      }
    }
  }
  return covered
}

// Checks that every package with changes has a bump planned in a change file.
// With a base ref (PR mode), checks the changes introduced since the merge base
// against the change files added or modified in the same range.
// Without it, checks the changes since each package's latest release tag
// against all pending change files.
function checkMissingBumps(baseRef) {
  const packages = Object.fromEntries(
    Object.entries(covectorConfig.packages).filter(([, p]) => p.path)
  )

  let changeFiles
  if (baseRef) {
    const out = git(
      'diff',
      '--name-only',
      '--diff-filter=AM',
      `${baseRef}...HEAD`,
      '--',
      '.changes/*.md'
    )
    changeFiles = out ? out.split('\n') : []
  } else {
    changeFiles = pendingChangeFiles()
  }
  changeFiles = changeFiles.filter(isPendingChangeFile)

  const bumped = new Set(changeFiles.flatMap(changeFilePackages))
  const covered = withDependents(packages, bumped)

  const missing = []
  for (const [name, pkg] of Object.entries(packages)) {
    let from
    if (baseRef) {
      from = `${baseRef}...HEAD`
    } else {
      const tag = `${name}-v${packageVersion(pkg)}`
      if (!git('tag', '--list', tag)) {
        console.warn(`Skipping \`${name}\`: release tag ${tag} not found`)
        continue
      }
      from = `${tag}..HEAD`
    }

    const files = changedFiles(from, pkg.path)
    if (files.length > 0 && !covered.has(name)) {
      missing.push({ name, from, files })
    }
  }

  if (missing.length > 0) {
    for (const { name, from, files } of missing) {
      console.error(
        `Package \`${name}\` has changes (${from}) but no planned bump in .changes:`
      )
      for (const file of files.slice(0, 10)) console.error(`  ${file}`)
      if (files.length > 10)
        console.error(`  ... and ${files.length - 10} more`)
      console.error('  commits:')
      // `base...HEAD` in git log would also list the base branch commits
      const range = from.replace('...', '..')
      const commits = git(
        'log',
        '--no-merges',
        '--format=%h %s',
        range,
        '--',
        ...files
      )
      for (const commit of commits.split('\n')) console.error(`    ${commit}`)
    }
    console.error(
      '\nAdd a change file, see https://github.com/jbolda/covector/tree/main/covector#readme'
    )
    process.exit(1)
  }
}

const [_bin, _script, ...files] = process.argv

if (files[0] === '--missing-bumps') {
  checkMissingBumps(files[1])
} else if (files.length > 0) {
  checkChangeFiles(files.filter(isPendingChangeFile))
} else {
  checkChangeFiles(pendingChangeFiles())
}
