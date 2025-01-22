---
sidebar_position: 1
---

import CodeSnippet from '@site/src/components/CodeSnippet';
import docusaurusConfig from '!!raw-loader!@site/docusaurus.config.ts';
export const context = require(`@substrate/substrate/src/context.rs?snippet`);

# Organization

Substrate documentation consists of [API docs],
[user docs]({{GITHUB_URL}}/docs/docusaurus/docs), and [developer docs]({{GITHUB_URL}}/docs/docusaurus/dev).

The API docs simply uses `cargo doc` to generate docs for the root cargo workspace as static HTML.

The user docs and developer docs are Docusaurus docs. The Docusaurus website is configured
using [`docusaurus.config.ts`] and [`site-config.json`]. The behavior of the website differs
depending on whether the `branch` key in [`site-config.json`] is `main` or a different branch:

- On the `main` branch, the blog is present and release documentation is available.
- On other branches, only documentation corresponding to the latest code is available.

The [`site-config.json`] should not be modified in the code base, but is automatically
updated when deploying docs from a branch.

User docs are versioned, while developer docs are not. To make it easier to reference
correctly-versioned files, the [`docusaurus.config.ts`] provides several global variables:

<CodeSnippet language="ts" title="docusaurus.config.ts" snippet="global-vars">{docusaurusConfig}</CodeSnippet>

In the user docs, code snippets should always reference code in the appropriately-versioned [`examples/*` folder].
This can be done using the `EXAMPLES` global variable:

```jsx
import CodeSnippet from "@site/src/components/CodeSnippet";
export const inverterMod = require(
  `{{>EXAMPLES}}/sky130_inverter/src/lib.rs?snippet`,
);

<CodeSnippet language="rust" title="src/lib.rs" snippet="imports">
  {inverterMod}
</CodeSnippet>;
```

The developer docs should always reference the latest code. All of the global variables will reference
the correctly-versioned docs when referenced in the developer docs. The following can be used
to reference the actual code base:

```jsx
import CodeSnippet from "@site/src/components/CodeSnippet";
export const context = require(`@substrate/substrate/src/context.rs?snippet`);

<CodeSnippet language="rust" title="substrate/src/context.rs" snippet="context">
  {context}
</CodeSnippet>;
```

The resulting code snippet would look like this:

<CodeSnippet language="rust" title="substrate/src/context.rs" snippet="context">{context}</CodeSnippet>

[API docs]: {{GITHUB_URL}}/docs/api
[user docs]: {{GITHUB_URL}}/docs/docusaurus/docs
[developer docs]: {{GITHUB_URL}}/docs/docusaurus/dev
[`docusaurus.config.ts`]: {{GITHUB_URL}}/docs/docusaurus/docusaurus.config.ts
[`site-config.json`]: {{GITHUB_URL}}/docs/docusaurus/site-config.json
[`examples/*` folder]: {{GITHUB_URL}}/examples
