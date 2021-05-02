---
title: Lockdown
---

import Rater from '@theme/Rater'
import useBaseUrl from '@docusaurus/useBaseUrl'

<div className="row">
  <div className="col col--4">
    <table>
      <tr>
        <td>Ease of Use</td>
        <td><Rater value="2"/></td>
      </tr>
      <tr>
        <td>Extensibility</td>
        <td><Rater value="4"/></td>
      </tr>
      <tr>
        <td>Performance</td>
        <td><Rater value="5"/></td>
      </tr>
      <tr>
        <td>Security</td>
        <td><Rater value="5" color="#fff04d"/></td>
      </tr>
    </table>
  </div>
  <div className="col col--4 pattern-logo">
    <img src={useBaseUrl('img/patterns/Lockdown.png')} alt="Lockdown" />
  </div>
  <div className="col col--4">
    Pros:
    <ul>
      <li>Highest security rating</li>
      <li>Elegant and powerful</li>
    </ul>
    Cons:
    <ul>
      <li>Rust skills required</li>
      <li>No remote resources</li>
    </ul>
  </div>
</div>


## Description

The Lockdown recipe is a minimal usage of the [Bridge pattern](/docs/usage/patterns/bridge), which only allows interaction between Rust and the Window via expiring JS Promise Closures that are injected into the Window by Rust and nulled as part of the callback.

## Diagram

import Mermaid, { colors } from '@theme/Mermaid'

<Mermaid chart={`graph TD
      H==>F
      G-.->B
      B-->G
      subgraph WEBVIEW
      G-->F
      end
      subgraph RUST
      A-->B
      A==>H
      end
      A[Binary]
      B[API:Event]
      F[Window]
      G((Promise Closure))
      H{Bootstrap}
      style RUST fill:${colors.orange.light},stroke:${colors.orange.dark},stroke-width:4px
      style WEBVIEW fill:${colors.blue.light},stroke:${colors.blue.dark},stroke-width:4px`} />


## Configuration

Here's what you need to add to your tauri.conf.json file:
```json
"tauri": {
  "allowlist": {}                  // all API endpoints are default false
}
```
