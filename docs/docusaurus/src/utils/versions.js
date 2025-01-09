export function isRelease(docsConfig) {
    return docsConfig.examples_path != "examples/latest";
}
