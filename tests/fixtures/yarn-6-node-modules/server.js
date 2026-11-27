const express = require("express");

const port = process.env['PORT'] || 8080;
const app = express();

app.get("/", (_req, res) => {
    res.send("Hello from yarn-6-node-modules");
});

app.listen(port, () => {
    console.log(`yarn-6-node-modules running on ${port}.`);
});
