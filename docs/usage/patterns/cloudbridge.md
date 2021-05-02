---
title: Cloudbridge
---

import Rater from '@theme/Rater'
import useBaseUrl from '@docusaurus/useBaseUrl'

<div className="row">
  <div className="col col--4">
    <table>
      <tr>
        <td>Ease of Use</td>
        <td><Rater value="1"/></td>
      </tr>
      <tr>
        <td>Extensibility</td>
        <td><Rater value="5"/></td>
      </tr>
      <tr>
        <td>Performance</td>
        <td><Rater value="3"/></td>
      </tr>
      <tr>
        <td>Security</td>
        <td><Rater value="2"/></td>
      </tr>
    </table>
  </div>
  <div className="col col--4 pattern-logo">
    <img src={useBaseUrl('img/patterns/Cloudbridge.png')} alt="Cloudbridge" />
  </div>
    <div className="col col--4">
    Pros:
    <ul>
      <li>All available features</li>
      <li>No Rust skills required</li>
    </ul>
    Cons:
    <ul>
      <li>Largest bundle size</li>
      <li>Hard to separate concerns</li>
    </ul>
  </div>
</div>

## Description

The Cloudbridge recipe combines the flexibility of a localhost and the security of the bridge. With so many features, it can be easy to get lost.

## Diagram

import Mermaid, { colors } from '@theme/Mermaid'

<Mermaid chart={`graph TD
      H==>F2
      H==>D2
      D2-->F2
      F2-->D2
      B-->D
      D-->B
      E2-->D
      D-->E2
      subgraph WEBVIEW
      F2
      E2
      end
      subgraph SERVER
      D2
      E-->D2
      end
      subgraph RUST
      A==>H
      A-->B
      B-.-C
      end
      A[Binary]
      B{Rust Broker}
      C[Subprocess]
      D(( API BRIDGE ))
      E{JS Broker}
      D2(( localhost ))
      E[bundled resources]
      E2{JS Broker}
      F2[Window]
      H{Bootstrap}
      style D fill:#ccc,stroke:#333,stroke-width:4px,color:white
      style RUST fill:${colors.orange.light},stroke:${colors.orange.dark},stroke-width:4px
      style WEBVIEW fill:${colors.blue.light},stroke:${colors.blue.dark},stroke-width:4px
      style SERVER fill:#49A24A,stroke:#2B6063,stroke-width:4px
      `} />


## Configuration

Here's what you need to add to your tauri.conf.json file:
```json
"tauri": {
  "allowlist": {
    "all": true                   // enable entire API
  }
}
```
