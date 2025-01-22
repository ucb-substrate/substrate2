import CodeBlock from "@theme/CodeBlock";
const Diff = require("diff");
const magicComments = [
  "begin-code-snippet",
  "end-code-snippet",
  "begin-code-replace",
  "end-code-replace",
];

function trimLeadingWS(str) {
  /*
    Get the initial indentation
    But ignore new line characters
  */
  var matcher = /^[\r\n]?(\s+)/;
  if (matcher.test(str)) {
    /*
      Replace the initial whitespace 
      globally and over multiple lines
    */
    return str.replace(new RegExp("^" + str.match(matcher)[1], "gm"), "");
  } else {
    // Regex doesn't match so return the original string
    return str;
  }
}

function getCommentSnippet(trimmed, comment) {
  let re = new RegExp(`^// ${comment} ([a-zA-Z_-]*)`);
  let caps = re.exec(trimmed);

  return caps ? caps[1] : null;
}

function isMagicComment(trimmed, comment, snippet) {
  if (comment) {
    let suffix = "";
    if (snippet) {
      if (
        trimmed === `// ${comment} ${snippet}` ||
        trimmed === `# ${comment} ${snippet}`
      ) {
        return snippet;
      }
    } else {
      return getCommentSnippet(trimmed, comment);
    }
  } else {
    for (const comment of magicComments) {
      let snippet = getCommentSnippet(trimmed, comment);
      if (snippet) {
        return snippet;
      }
    }
  }
  return null;
}

function getSnippet(content, snippet, replacements = {}) {
  console.log(replacements);
  var inSnippet = false;
  var inReplace = null;
  var commentSnippet;
  var selected = "";
  for (const line of content.split("\n")) {
    console.log(inReplace);
    const trimmed = line.trim();
    if (isMagicComment(trimmed, "begin-code-snippet", snippet)) {
      inSnippet = true;
    } else if (isMagicComment(trimmed, "end-code-snippet", snippet)) {
      return trimLeadingWS(selected);
    } else if (
      !inReplace &&
      (commentSnippet = isMagicComment(trimmed, "begin-code-replace"))
    ) {
      if (commentSnippet in replacements) {
        inReplace = commentSnippet;
      }
      console.log(inReplace);
    } else if (
      inReplace &&
      isMagicComment(trimmed, "end-code-replace") === inReplace
    ) {
      let ws = line.match(/^\s*/);
      selected += `${ws}${replacements[inReplace]}\n`;
      inReplace = null;
    } else if (inSnippet && !inReplace && !isMagicComment(trimmed)) {
      selected += `${line}\n`;
    }
  }

  throw new Error(`Code snippet '${snippet}' not found`);
}

function generateDiff(source, target) {
  var diff = Diff.diffLines(source, target);
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

function CodeSnippet({
  children,
  snippet,
  language,
  title,
  showLineNumbers,
  diffSnippet,
  replacements,
}) {
  let target = getSnippet(children, snippet, replacements);
  let copy;
  if (diffSnippet) {
    let source = getSnippet(children, diffSnippet, replacements);
    copy = target;
    target = generateDiff(source, target);
  }
  return (
    <div>
      <CodeBlock
        language={language}
        title={title}
        showLineNumbers={showLineNumbers}
        copy={copy}
      >
        {target}
      </CodeBlock>
    </div>
  );
}

export default CodeSnippet;
