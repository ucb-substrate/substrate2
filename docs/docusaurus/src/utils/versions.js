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

export function getGitHubUrl(branch) {
    return `https://github.com/ucb-sustrate/substrate2/tree/${branch}`;
}

export function isRelease(version) {
    return version != siteConfig.branch;
}
