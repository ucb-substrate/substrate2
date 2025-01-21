import CodeSnippet from "@site/src/components/CodeSnippet";
import { isRelease } from "@site/src/utils/versions.js";
const siteConfig = require("@site/site-config.json");

function DependenciesSnippet({
  children,
  version,
  snippet,
  language,
  title,
  showLineNumbers,
}) {
  // If this is for the current code, can simply use a git dependency.
  if (!isRelease(version)) {
    children = children.replace(
      /version = .*, registry = "substrate", path = ".*"/g,
      `git = "https://github.com/ucb-substrate/substrate2.git", branch = "${siteConfig.branch}"`,
    );
  }
  return (
    <CodeSnippet
      language={language}
      title={title}
      snippet={snippet}
      showLineNumbers={showLineNumbers}
    >
      {children}
    </CodeSnippet>
  );
}

export default DependenciesSnippet;
