import PugPlugin from 'pug-plugin'
import WasmPackPlugin from '@wasm-tool/wasm-pack-plugin'
import url from 'url'
import path from 'path'

const __dirname = url.fileURLToPath(new URL('.', import.meta.url))

export default {
    mode: 'production',

    output: {
        path: path.join(__dirname, 'build'),
        publicPath: '/markdown-it/',
        filename: '[name].[contenthash:8].js'
    },

    entry: {
        index: './src/index.pug'
    },

    devServer: {
        watchFiles: [ './pkg/*', './src/*' ],
    },

    watchOptions: {
        aggregateTimeout: 1000,
        poll: 1000,
    },

    plugins: [
        new PugPlugin({
            css: {
                filename: '[name].[contenthash:8].css',
            },
        }),

        new WasmPackPlugin({
            crateDirectory: __dirname,
        }),
    ],

    module: {
        rules: [
            {
                test: /\.css$/,
                use: [ 'css-loader' ]
            },
            {
                test: /\.styl$/,
                exclude: /node_modules/,
                use: [ 'css-loader', 'stylus-loader' ]
            },
            {
                test: /\.pug$/,
                loader: PugPlugin.loader,
                options: {
                    method: 'render',
                    self: true,
                }
            },
        ],
    },

    experiments: {
        asyncWebAssembly: true
    },

    performance: {
        hints: false
    },

    stats: {
        logging: 'verbose',
    },
}
