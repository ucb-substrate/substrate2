import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'Substrate Labs',
  tagline: '21st century electronic design automation tools, written in Rust.',
  favicon: 'img/substrate_logo2_blue.png',

  // Set the production url of your site here
  url: 'https://docs.substratelabs.io',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

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

  plugins: ['./src/plugins/substrate-source-assets'],

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/substrate-labs/substrate2/tree/main/docs/site',
        },
        blog: {
          showReadingTime: true,
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/substrate-labs/substrate2/tree/main/docs/site',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],

  themeConfig: {
      // Replace with your project's social card
      image: 'img/substrate_logo2.png',
      navbar: {
        title: 'Substrate Labs',
        logo: {
          alt: 'Substrate Labs Logo',
          src: 'img/substrate_logo2.png',
          srcDark: 'img/substrate_logo2_dark.png',
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'tutorialSidebar',
            position: 'left',
            label: 'Documentation',
          },
          {
            href: 'https://api.substratelabs.io/substrate/',
            label: 'API',
            position: 'left',
          },
          {
            href: 'https://github.com/substrate-labs/substrate2',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              {
                label: 'Tutorial',
                to: '/docs/category/getting-started',
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'Discord',
                href: 'https://discord.gg/XhtHzrEyaG',
              },
            ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'GitHub',
                href: 'https://github.com/substrate-labs/substrate2',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Substrate Labs. Built with Docusaurus.`,
      },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'toml'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
