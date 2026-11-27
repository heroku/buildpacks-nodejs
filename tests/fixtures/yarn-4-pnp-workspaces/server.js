const express = require("express");
const { greet } = require("greeting");

const port = process.env['PORT'] || 8080;
const app = express();

app.get("/", (_req, res) => {
    res.send(greet("yarn-4-pnp-workspaces"));
});

app.listen(port, () => {
    console.log(`yarn-4-pnp-workspaces running on ${port}.`);
});
