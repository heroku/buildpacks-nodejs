const express = require("express");

const port = process.env['PORT'] || 8080;
const app = express();

app.get("/", (_req, res) => {
    res.send("Hello from yarn-4-pnp-zero");
});

app.listen(port, () => {
    console.log(`yarn-4-pnp-zero running on ${port}.`);
});
