const webpack = require('webpack');

module.exports = {
  plugins: [
    new webpack.DefinePlugin({
      ENV: {
        apiUrl: JSON.stringify(process.env.API_URL),
        commitVersion: JSON.stringify(process.env.COMMIT_SHA),
        buildTimestamp: JSON.stringify(process.env.BUILD_TIMESTAMP),
      }
    }),
  ],
};
