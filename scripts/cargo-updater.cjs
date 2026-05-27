// scripts/cargo-updater.cjs
module.exports = {
  readVersion: function (contents) {
    const match = contents.match(/^version = "([^"]+)"/m);
    return match ? match[1] : null;
  },
  writeVersion: function (contents, version) {
    return contents.replace(/^version = "[^"]*"/m, `version = "${version}"`);
  }
};
