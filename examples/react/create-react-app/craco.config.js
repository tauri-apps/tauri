module.exports = {
    webpack: {
        configure: (config) => {
            config.output.publicPath = ''

            return config
        }
    }
}
