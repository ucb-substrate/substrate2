export function getExamplesPath(version) {
    return `examples/${version}`;
}

export function getApiDocsLink(version, path) {
    return `https://api.substratelabs.io/${version}/${path}`;
}

export function isRelease(version) {
    return version != "latest";
}
