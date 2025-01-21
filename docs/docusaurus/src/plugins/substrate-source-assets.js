const path = require('path');
// A JavaScript function that returns an object.
// `context` is provided by Docusaurus. Example: siteConfig can be accessed from context.
// `opts` is the user-defined options.
async function substrateSourceAssets(context, opts) {
  return {
    // A compulsory field used as the namespace for directories to cache
    // the intermediate data for each plugin.
    // If you're writing your own local plugin, you will want it to
    // be unique in order not to potentially conflict with imported plugins.
    // A good way will be to add your own project name within.
    name: 'substrate-labs-substrate-source-assets',

    configureWebpack(config, isServer, utils, content) {
      // Modify internal webpack config. If returned value is an Object, it
      // will be merged into the final config using webpack-merge;
      // If the returned value is a function, it will receive the config as the 1st argument and an isServer flag as the 2nd argument.
      return {
        module: {
          rules: [
            {
              resourceQuery: /snippet/,
              type: 'asset/source',
            }
          ]
        },
        resolve: {
          alias: {
            "@substrate": path.resolve(__dirname, '../../../../'),
          }
        }
      };
    },

    getPathsToWatch() {
      // Paths to watch.
    },

  };
}

module.exports = substrateSourceAssets;
