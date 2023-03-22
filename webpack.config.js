//@ts-check

"use strict";

const path = require("path");

//@ts-check
/** @typedef {import('webpack').Configuration} WebpackConfig **/

/** @type WebpackConfig */
const extensionConfig = {
    target: "node",
    mode: "none",

    entry: "./src/extension.ts",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "extension.js",
        libraryTarget: "commonjs2",
    },
    externals: {
        vscode: "commonjs vscode",
    },
    resolve: {
        extensions: [".ts", ".js"],
        alias: {
            "@crates/cursor-core": path.resolve(
                __dirname,
                "crates/cursor-core/pkg"
            ),
        },
    },
    module: {
        rules: [
            {
                test: /\.ts$/,
                exclude: /node_modules/,
                use: [
                    {
                        loader: "ts-loader",
                    },
                ],
            },
        ],
    },
    devtool: "nosources-source-map",
    infrastructureLogging: {
        level: "log", // enables logging required for problem matchers
    },
};
module.exports = [extensionConfig];
