const siteConfig = require('../../site-config.json');

export function getExamplesPath(version) {
    if (!isRelease(version)) {
        version = "latest";
    }
    return `@substrate/examples/${version}`;
}

export function getApiDocsUrl(version) {
    return `https://api.substratelabs.io/${version}`;
}

export function isRelease(version) {
    return version != siteConfig.branch;
}
