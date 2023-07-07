import CodeBlock from "@theme/CodeBlock";
import React from 'react';

function CodeSnippet({children, snippet, language, title, showLineNumbers}) {
  var inSnippet = false;
  var selected = "";
  for (const line of children.split('\n')) {
    const trimmed = line.trim();
    if (trimmed === `// begin-code-snippet ${snippet}`) {
      inSnippet = true;
    } else if (trimmed === `// end-code-snippet ${snippet}`) {
      return (<div><CodeBlock language={language} title={title} showLineNumbers={showLineNumbers}>{selected}</CodeBlock></div>);
    } else if (inSnippet) {
      selected += `${line}\n`;
    }
  }

  throw new Error(`Code snippet '${snippet}' not found`);
}

export default CodeSnippet;
