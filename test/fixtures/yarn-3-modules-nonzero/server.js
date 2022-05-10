const express = require("express");

const port = process.env['PORT'] || 8080;
const app = express();

app.get("/", (_req, res) => {
    res.send("Hello from yarn-3-modules-nonzero");
});

app.listen(port, () => {
    console.log(`Express running on ${port}.`);
});
