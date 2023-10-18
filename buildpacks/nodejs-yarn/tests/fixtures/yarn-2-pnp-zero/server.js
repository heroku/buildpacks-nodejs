const express = require('express')
const app = express()
const port = process.env["PORT"] || 8080;

app.get('/', (req, res) => {
  res.send('yarn-2-pnp-zero')
})

app.listen(port, () => {
  console.log(`yarn-2-pnp-zero listening on port ${port}`)
})
