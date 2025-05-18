const webpack = require('webpack');

module.exports = {
  plugins: [
    new webpack.DefinePlugin({
      ENV: {
        apiUrl: process.env.API_URL,
        commitVersion: process.env.COMMIT_SHA,
        buildTimestamp: process.env.BUILD_TIMESTAMP,
      }
    }),
  ],
};
