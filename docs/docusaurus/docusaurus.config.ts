import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import * as fs from 'fs';
import toml from '@iarna/toml';

const siteConfig = toml.parse(fs.readFileSync('../site-config.toml', 'utf-8'));

const config: Config = {
  title: 'Substrate Labs',
  tagline: '21st century electronic design automation tools, written in Rust.',
  favicon: 'img/substrate_logo_blue.png',

  // Set the production url of your site here
  url: 'https://docs.substratelabs.io',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: siteConfig.docusaurus.base_url,

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'substrate-labs', // Usually your GitHub org/user name.
  projectName: 'substrate', // Usually your repo name.

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'throw',

  // Even if you don't use internalization, you can use this field to set useful
  // metadata like html lang. For example, if your site is Chinese, you may want
  // to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  onBrokenLinks: 'ignore',

  plugins: ['./src/plugins/substrate-source-assets'],

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          versions: {
            current: {
              label: 'main',
              banner: siteConfig.docusaurus.banner,
            },
          },
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl: siteConfig.docusaurus.edit_url,
        },
        blog: {
          showReadingTime: true,
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl: siteConfig.docusaurus.edit_url
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],

  themeConfig: {
      // Replace with your project's social card
      image: 'img/substrate_logo.png',
      navbar: {
        title: 'Substrate Labs',
        logo: {
          alt: 'Substrate Labs Logo',
          src: 'img/substrate_logo.png',
          srcDark: 'img/substrate_logo_dark.png',
        },
        items: [
          {
            type: 'docsVersion',
            to: "/docs",
            position: 'left',
            label: 'Documentation',
          },
          {
            href: 'https://api.substratelabs.io/substrate/',
            label: 'API',
            position: 'left',
          },
          {href: '/blog', autoAddBaseUrl: false, label: 'Blog', position: 'left'},
          { type: 'docsVersionDropdown', position: 'right' },
          // {
          //   type: 'dropdown',
          //   label: siteConfig.docusaurus.current_version,
          //   position: 'right',
          //   items: [
          //     {
          //       label: 'release',
          //       href: '/docs',
          //       autoAddBaseUrl: false,
          //     },
          //     // ... more items
          //   ].concat(
          //     siteConfig.site.versions.map((version) => {
          //     return {
          //       label: version,
          //       href: `/${version}/docs`,
          //       autoAddBaseUrl: false,
          //     }}))
          //   ,
          // },
          {
            href: 'https://github.com/substrate-labs/substrate2',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [],
        copyright: `Copyright © ${new Date().getFullYear()} Substrate Labs. Built with Docusaurus.`,
      },
    prism: {
      theme: prismThemes.oneLight,
      darkTheme: prismThemes.palenight, // nightowl
      additionalLanguages: ['rust', 'toml'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
