import CodeBlock from "@theme/CodeBlock";
const Diff = require("diff");

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

function getSnippet(content, snippet) {
  var inSnippet = false;
  var selected = "";
  for (const line of content.split('\n')) {
    const trimmed = line.trim();
    if (trimmed === `// begin-code-snippet ${snippet}` || trimmed === `# begin-code-snippet ${snippet}`) {
      inSnippet = true;
    } else if (trimmed === `// end-code-snippet ${snippet}` || trimmed === `# end-code-snippet ${snippet}`) {
      return trimLeadingWS(selected);
    } else if (inSnippet) {
      selected += `${line}\n`;
    }
  }

  throw new Error(`Code snippet '${snippet}' not found`);
}

function generateDiff(source, target) {
    var diff = Diff.diffLines(source, target, {ignoreWhitespace: true});
    var final = "";
    diff.forEach((part) => {
        // green for additions, red for deletions
        var prefix = "";
        if (part.added) {
            prefix = "// diff-add\n";
        } else if (part.removed) {
            prefix = "// diff-remove\n";
        }

        for (const line of part.value.split("\n").slice(0, -1)) {
            final += `${prefix}${line}\n`;
        }
    });
    return final;
}


function CodeSnippet({children, snippet, language, title, showLineNumbers, diffSnippet}) {
    let target = getSnippet(children, snippet);
  if (diffSnippet) {
      let source = getSnippet(children, diffSnippet);
      target = generateDiff(source, target);
  }
  return (<div><CodeBlock language={language} title={title} showLineNumbers={showLineNumbers}>{target}</CodeBlock></div>);
}

export default CodeSnippet;
