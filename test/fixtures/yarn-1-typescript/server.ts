import express from 'express';

const app = express();
const port = process.env['PORT'] || 8000;

app.get("/", (_req, res) => {
  res.send("Hello from yarn-1-typescript");
});

app.listen(port, () => {
  console.log(`Express running on ${port}.`);
});
