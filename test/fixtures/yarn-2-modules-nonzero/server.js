const express = require('express')
const app = express()
const port = process.env["PORT"] || 8080;

app.get('/', (req, res) => {
  res.send('yarn-2-modules-nonzero')
})

app.listen(port, () => {
  console.log(`yarn-2-modules-nonzero listening on port ${port}`)
})
