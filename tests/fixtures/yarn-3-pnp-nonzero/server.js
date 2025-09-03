const express = require("express");

const port = process.env['PORT'] || 8080;
const app = express();

app.get("/", (_req, res) => {
    res.send("Hello from yarn-3-pnp-nonzero");
});

app.listen(port, () => {
    console.log(`yarn-3-pnp-nonzero running on ${port}.`);
});
