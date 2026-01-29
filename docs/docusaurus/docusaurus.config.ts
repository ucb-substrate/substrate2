import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";
import * as fs from "fs";
import {
  getExamplesPath,
  getApiDocsUrl,
  getGitHubUrl,
} from "./src/utils/versions";
const siteConfig = require("./site-config.json");
const isMain = siteConfig.branch == "main";
const editUrl = `${getGitHubUrl(siteConfig.branch)}/docs/docusaurus`;

const config: Config = {
  title: "Substrate Labs",
  tagline: "21st century electronic design automation tools, written in Rust.",
  favicon: "img/substrate_logo_blue.png",

  // Set the production url of your site here
  url: "https://docs.substratelabs.io",
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: isMain ? "/" : `/branch/${siteConfig.branch}/`,

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: "substrate-labs", // Usually your GitHub org/user name.
  projectName: "substrate", // Usually your repo name.

  onBrokenLinks: "throw",

  // Even if you don't use internalization, you can use this field to set useful
  // metadata like html lang. For example, if your site is Chinese, you may want
  // to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  onBrokenLinks: "ignore",

  plugins: [
    "./src/plugins/substrate-source-assets",
    [
      "@docusaurus/plugin-content-docs",
      {
        id: "dev",
        sidebarPath: require.resolve("./sidebars.js"),
        path: "dev",
        routeBasePath: "dev",
        // ... other options
      },
    ],
  ],

  presets: [
    [
      "classic",
      /** @type {import('@docusaurus/preset-classic').Options} */
      {
        docs: {
          ...(!isMain && {
            onlyIncludeVersions: ["current"],
            lastVersion: "current",
          }),
          exclude: ['**/assets/**'],
          sidebarPath: require.resolve("./sidebars.js"),
          versions: {
            current: {
              label: siteConfig.branch,
              path: isMain ? siteConfig.branch : "",
              banner: "unreleased",
            },
          },
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl: editUrl,
        },
        blog: false // replace with `isMain` once blog is ready
          ? {
              // Please change this to your repo.
              // Remove this to remove the "edit this page" links.
              editUrl: editUrl,
            }
          : false,
        theme: {
          customCss: require.resolve("./src/css/custom.css"),
        },
      },
    ],
  ],

  markdown: {
    format: "mdx",
    hooks: {
        onBrokenMarkdownLinks: "throw",
    },
    preprocessor: ({ filePath, fileContent }) => {
      let version;
      let match = /versioned_docs\/version-([a-zA-Z0-9_-]*)\//.exec(filePath);
      if (match) {
        version = match[1];
      } else {
        version = siteConfig.branch;
      }

      // begin-code-snippet global-vars
      let vars = [
        ["VERSION", version],
        ["EXAMPLES", getExamplesPath(version)],
        ["API", getApiDocsUrl(version)],
        ["GITHUB_URL", getGitHubUrl(siteConfig.branch)],
      ];
      // end-code-snippet global-vars

      for (const [key, value] of vars) {
        fileContent = fileContent.replaceAll(`{{${key}}}`, value);
        fileContent = fileContent.replace(
          new RegExp(`{{(>*)>${key}}}`, "g"),
          `{{$1${key}}}`,
        );
      }
      return fileContent;
    },
  },

  themeConfig: {
    // Replace with your project's social card
    image: "img/substrate_logo.png",
    navbar: {
      title: "Substrate Labs",
      logo: {
        alt: "Substrate Labs Logo",
        src: "img/substrate_logo.png",
        srcDark: "img/substrate_logo_dark.png",
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "tutorialSidebar",
          position: "left",
          label: "Docs",
        },
        {
          type: "docSidebar",
          sidebarId: "tutorialSidebar",
          docsPluginId: "dev",
          position: "left",
          label: "Developers",
        },
        ...(isMain
          ? [
              // { to: "blog", label: "Blog", position: "left" }, // Uncomment once blog is ready
              {
                type: "docsVersionDropdown",
                position: "right",
              },
            ]
          : [
              {
                type: "docsVersion",
                position: "right",
              },
            ]),
        {
          type: "custom-apiLink",
          position: "right",
        },
        {
          href: getGitHubUrl(siteConfig.branch),
          label: "GitHub",
          position: "right",
        },
      ],
    },
    footer: {
      style: "dark",
      links: [],
      copyright: `Copyright © ${new Date().getFullYear()} Substrate Labs. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.oneLight,
      darkTheme: prismThemes.palenight, // nightowl
      magicComments: [
        {
          className: "code-block-diff-add-line",
          line: "diff-add",
        },
        {
          className: "code-block-diff-remove-line",
          line: "diff-remove",
        },
      ],
      additionalLanguages: ["rust", "toml"],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
