module.exports = {
  env: {
    node: true,
    jest: true
  },

  extends: ["standard"],

  plugins: [],

  globals: {
    __statics: true,
    process: true
  },

  // add your custom rules here
  rules: {
    // allow console.log during development only
    "no-console": process.env.NODE_ENV === "production" ? "error" : "off",
    // allow debugger during development only
    "no-debugger": process.env.NODE_ENV === "production" ? "error" : "off"
  }
}
