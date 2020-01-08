module.exports = {
  siteMetadata: {
    siteTitle: `Gatsby Themed Site In Tauri`,
    siteDescription: `This is a smoke test to see that a (themed) Gatsbyjs site will work within Tauri.`,
    siteAuthor: `Jacob Bolda`,
    siteContact: "https://twitter.com/jacobbolda",
    siteURL: "https://www.tauri.studio",
    contactLinks: [
      {
        url: "https://twitter.com/jacobbolda",
        text: "@jacobbolda",
        icon: ["fab", "twitter"]
      },
      {
        url: "https://twitter.com/TauriApps",
        text: "@TauriApps",
        icon: ["fab", "twitter"]
      }
    ],
    navLinks: [{ text: "Articles", url: "/articles/" }]
  },
  plugins: [
    {
      resolve: `gatsby-source-filesystem`,
      options: {
        name: `articles`,
        path: `${__dirname}/src/articles/`
      }
    },
    {
      resolve: `gatsby-source-filesystem`,
      options: {
        name: `homepage`,
        path: `${__dirname}/src/homepage/`
      }
    },
    `gatsby-plugin-theme-ui`,
    `gatsby-plugin-sharp`,
    `gatsby-transformer-sharp`,
    `@jbolda/gatsby-theme-homepage`,
    `@jbolda/gatsby-theme-articles`,
    {
      resolve: `gatsby-plugin-mdx`,
      options: {}
    },
    `gatsby-plugin-react-helmet`,
    `gatsby-plugin-netlify`
  ]
};
