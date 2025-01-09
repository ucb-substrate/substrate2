import CodeSnippet from '@site/src/components/CodeSnippet';
import {isRelease} from '@site/src/utils/versions.js';
const siteConfig = require('@site/site-config.json');

function DependenciesSnippet({children, docsConfig, snippet, language, title, showLineNumbers}) {
    console.log(children);
  // If this is for the current code, can simply use a git dependency.
  if (!isRelease(docsConfig)) {
      children = children.replace(/registry = "substrate", path = ".*"/g, `git = "https://github.com/ucb-substrate/substrate2", branch = "${siteConfig.branch}"`);
  }
    console.log(children);
  return (
      <CodeSnippet language={language} title={title} snippet={snippet} showLineNumbers={showLineNumbers}>{children}</CodeSnippet>
  );
}

export default DependenciesSnippet;
