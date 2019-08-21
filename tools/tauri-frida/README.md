# Tauri-Frida

Coming soon, a complete [Frida](https://frida.re) harness for testing, debugging, decompiling and recompiling App binaries. While it is made explicitly for Tauri Apps, it will probably prove useful for any binary in existence - no matter the source or original compiler.

## Post Binary Analysis - The Last Mile
Normal tests have one huge deficiency: They are generally run in artificially constructed environments that are mere reflections of the reality of your application. Even e2e tests, while closest to reality - just aren't the same as the real thing. Enter **Post Binary Analysis** (PBA).

PBA is a novel technique for DEVSEC that helps evaluate and harden the binary of your project - exactly as it is in reality - after all of the building and packaging has been done. Commonly known as reverse-engineering or decompiling, Tauri-Frida brings you a super-charged tool to investigate your binaries. As a matter of fact, Frida will quite often be used by Security Researchers during their investigation of your App. This is why we are making it available to you, so you can get the same insights into your binary that they will use to penetrate it.

## Status of Tauri-Frida
We are currently in the evaluation and architectural-planning phase of this project, and you can expect things to grow and change. Here is a list of features that we expect to be able to deliver:

- [ ] Automatic Install of Frida
- [ ] Portable Binary including Frida Headers and Libs
- [ ] Binary Hooking at Runtime
- [ ] Static Analysis
- [ ] Pointer Evaluation
- [ ] Chaos Experimentation
- [ ] Report Generation
- [ ] Binary Pruning
- [ ] Binary Injection
- [ ] Matryoschkasumming
- [ ] Recompilation

Operating System Availability
- [ ] MacOS
- [ ] Windows
- [ ] GNU/Linux

## Installation
Frida requires python and runs an Windows, MacOS and GNU/Linux.

```
$ pip install frida-tools
```

Or you can grab binaries from Frida's GitHub [releases](https://github.com/frida/frida/releases) page. 
We plan on automating the installation of Frida, 

## Binary Hooking at Runtime
> TODO

## Static Analysis
> TODO

## Pointer Evaluation
> TODO

## Chaos Experimentation
- Interface Jacking
- Value Spraying
- Fuzzing
- Spoofing
- Disk Change
- Latency
- Process Kill
- CPU Throttle

## Report Generation

## Binary Pruning
> TODO

## Binary Injection
> TODO

## Matryoschkasumming
> TODO

## Recompilation
> TODO

## Resources

### Inspirations
- https://github.com/nowsecure/frida-uikit
- https://github.com/frida/cryptoshark
- https://github.com/dweinstein/awesome-frida

### Notes
Some of the documentation on this page recycled from [frida.re](https://frida.re/docs/hacking/)

### License
(c) 2019 Daniel Thompson-Yvetot and Quasar Tauri Team Contributors

MIT
