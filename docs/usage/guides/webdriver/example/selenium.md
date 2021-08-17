---
title: Selenium
---
import Alert from '@theme/Alert'
import Tabs from '@theme/Tabs'
import TabItem from '@theme/TabItem'

<Alert title="Example Application" type="info" icon="info-alt">

This [Selenium] guide expects you to have already gone through the [example Application setup] in order to follow
step-by-step. The general information may still be useful otherwise.
</Alert>

This WebDriver testing example will use [Selenium] and a popular Node.js testing suite. It is expected to already have
Node.js installed, along with `npm` or `yarn` although the [finished example project] uses `yarn`.

## Create a Directory for the Tests

Let's start off by creating a space in our project to write these tests. We are going to be using a nested directory for
this example project as we will later also go over other frameworks, but typically you will only need to use one. Create
the directory we will use with `mkdir -p webdriver/selenium`. The rest of this guide will assume you are inside the
`webdriver/selenium` directory.

## Initializing a Selenium Project

We will be using a pre-existing `package.json` to bootstrap this test suite because we have already chosen specific
dependencies to use and want to showcase a simple working solution. The bottom of this section has a collapsed
guide on how to set it up from scratch.

`package.json`:
```json
{
  "name": "selenium",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "test": "mocha"
  },
  "dependencies": {
    "chai": "^4.3.4",
    "mocha": "^9.0.3",
    "selenium-webdriver": "^4.0.0-beta.4"
  }
}
```

We have a script which runs [Mocha] as a test framework exposed as the `test` command. We also have various dependencies
that we will be using to run the tests. [Mocha] as the testing framework, [Chai] as the assertion library, and
[`selenium-webdriver`] which is the Node.js [Selenium] package.

<details><summary>Click me if you want to see how to set a project up from scratch</summary>

If you wanted to install the dependencies from scratch, just run the following command.

<Tabs groupId="package-manager"
defaultValue="yarn"
values={[
{label: 'npm', value: 'npm'}, {label: 'Yarn', value: 'yarn'},
]}>
<TabItem value="npm">

```sh
npm install mocha chai selenium-webdriver
```

</TabItem>

<TabItem value="yarn">

```sh
yarn add mocha chai selenium-webdriver
```

</TabItem>
</Tabs>

I suggest also adding a `"test": "mocha"` item in the `package.json` `"scripts"` key so that running mocha can be called
simply with

<Tabs groupId="package-manager"
defaultValue="yarn"
values={[
{label: 'npm', value: 'npm'}, {label: 'Yarn', value: 'yarn'},
]}>
<TabItem value="npm">

```sh
npm test
```

</TabItem>

<TabItem value="yarn">

```sh
yarn test
```

</TabItem>
</Tabs>

</details>

## Testing

Unlike the [WebdriverIO Test Suite](webdriverio#config), Selenium does not come out of the box with a Test Suite and
leaves it up to the developer to build those out. We chose [Mocha] which is pretty neutral, and not related to WebDrivers
at all, so our script will need to do a bit of work to set up everything for us in the right order. [Mocha] expects a
testing file at `test/test.js` by default, so let's create that file now.

`test/test.js`:
```js
const os = require("os");
const path = require("path");
const { expect } = require("chai");
const { spawn, spawnSync } = require("child_process");
const { Builder, By, Capabilities } = require("selenium-webdriver");

// create the path to the expected application binary
const application = path.resolve(
  __dirname,
  "..",
  "..",
  "..",
  "target",
  "release",
  "hello-tauri-webdriver"
);

// keep track of the webdriver instance we create
let driver;

// keep track of the tauri-driver process we start
let tauriDriver;

before(async function() {
  // set timeout to 2 minutes to allow the program to build if it needs to
  this.timeout(120000)

  // ensure the program has been built
  spawnSync("cargo", ["build", "--release"]);

  // start tauri-driver
  tauriDriver = spawn(
    path.resolve(os.homedir(), ".cargo", "bin", "tauri-driver"),
    [],
    { stdio: [null, process.stdout, process.stderr] }
  );

  const capabilities = new Capabilities();
  capabilities.set("tauri:options", { application });
  capabilities.setBrowserName("wry");

  // start the webdriver client
  driver = await new Builder()
    .withCapabilities(capabilities)
    .usingServer("http://localhost:4444/")
    .build();
});

after(async function() {
  // stop the webdriver session
  await driver.quit();

  // kill the tauri-driver process
  tauriDriver.kill();
});

describe("Hello Tauri", () => {
  it("should be cordial", async () => {
    const text = await driver.findElement(By.css("body > h1")).getText();
    expect(text).to.match(/^[hH]ello/);
  });

  it("should be excited", async () => {
    const text = await driver.findElement(By.css("body > h1")).getText();
    expect(text).to.match(/!$/);
  });

  it("should be easy on the eyes", async () => {
    // selenium returns color css values as rgb(r, g, b)
    const text = await driver.findElement(By.css("body")).getCssValue("background-color");

    const rgb = text.match(/^rgb\((?<r>\d+), (?<g>\d+), (?<b>\d+)\)$/).groups;
    expect(rgb).to.have.all.keys('r','g','b');

    const luma =  0.2126 * rgb.r + 0.7152 * rgb.g  + 0.0722 * rgb.b ;
    expect(luma).to.be.lessThan(100)
  });
});
```

If you are familiar with JS testing frameworks, `describe`, `it`, and `expect` should look familiar. We also have
semi-complex `before()` and `after()` callbacks to setup and teardown mocha. Lines that are not the tests themselves
have comments explaining what the setup and teardown code is doing. If you were familiar with the Spec file from the
[WebdriverIO example](webdriverio#spec), you will notice a lot more code that isn't tests, as we have to set up a few
more WebDriver related items.

## Running the Test Suite

Now that we are all set up with our dependencies and our test script, lets run it!

<Tabs groupId="package-manager"
defaultValue="yarn"
values={[
{label: 'npm', value: 'npm'}, {label: 'Yarn', value: 'yarn'},
]}>
<TabItem value="npm">

```sh
npm test
```

</TabItem>

<TabItem value="yarn">

```sh
yarn test
```

</TabItem>
</Tabs>

We should see output the following output:

```text
➜  selenium git:(main) ✗ yarn test
yarn run v1.22.11
$ mocha


  Hello Tauri
    ✔ should be cordial (120ms)
    ✔ should be excited
    ✔ should be easy on the eyes


  3 passing (588ms)

Done in 0.93s.
```

We can see that our `Hello Tauri` sweet we created with `decribe` had all 3 items we created with `it` pass their
tests!

With [Selenium] and some hooking up to a test suite, we just enabled e2e testing without modifying our Tauri
application at all!


[Selenium]: https://selenium.dev/
[finished example project]: https://github.com/chippers/hello_tauri
[example Application setup]: setup
[Mocha]: https://mochajs.org/
[Chai]: https://www.chaijs.com/
[`selenium-webdriver`]: https://www.npmjs.com/package/selenium-webdriver
