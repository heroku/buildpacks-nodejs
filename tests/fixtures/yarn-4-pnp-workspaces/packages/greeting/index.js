const he = require("he");

exports.greet = (name) => `Hello from ${he.encode(name)}`;
