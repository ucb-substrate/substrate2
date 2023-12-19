import CodeBlock from "@theme/CodeBlock";
import React from 'react';

function trimLeadingWS(str) {
  /*
    Get the initial indentation
    But ignore new line characters
  */
  var matcher = /^[\r\n]?(\s+)/;
  if(matcher.test(str)) {
    /*
      Replace the initial whitespace 
      globally and over multiple lines
    */
    return str.replace(new RegExp("^" + str.match(matcher)[1], "gm"), "");
  } else {
    // Regex doesn't match so return the original string
    return str;
  }
};


function CodeSnippet({children, snippet, language, title, showLineNumbers}) {
  var inSnippet = false;
  var selected = "";
  for (const line of children.split('\n')) {
    const trimmed = line.trim();
    if (trimmed === `// begin-code-snippet ${snippet}` || trimmed === `# begin-code-snippet ${snippet}`) {
      inSnippet = true;
    } else if (trimmed === `// end-code-snippet ${snippet}` || trimmed === `# end-code-snippet ${snippet}`) {
      return (<div><CodeBlock language={language} title={title} showLineNumbers={showLineNumbers}>{trimLeadingWS(selected)}</CodeBlock></div>);
    } else if (inSnippet) {
      selected += `${line}\n`;
    }
  }

  throw new Error(`Code snippet '${snippet}' not found`);
}

export default CodeSnippet;
