const path = require('path')

module.exports = {
    entry: {
        'public/scripts/jquery': './public/scripts/jquery.min.js',
        'public/scripts/client': ['./public/scripts/client.js', './public/styles/bootstrap-utilities.min.css', './public/styles/style.css', './public/styles/apercu.css', './public/styles/apercu_mono.css'],
        'public/scripts/progress-bar': './public/scripts/external/progress-bar.min.js',
        'public/scripts/ray-core': ['./public/scripts/ray-core.min.js',  './public/styles/ray-core.min.css'],
    },
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: '[name].js'
    }, 
    module: {
        rules: [
            {
                test: /\.css$/,
                use: [
                    'style-loader',
                    'css-loader',
                ]
            }
        ]
    }
}