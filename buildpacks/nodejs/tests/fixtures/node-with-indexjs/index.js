const http = require('http');
const port = process.env["PORT"] || 8080;

const requestListener = function (req, res) {
    res.writeHead(200);
    res.end('Hello from node-with-indexjs!');
}

const server = http.createServer(requestListener);
server.listen(port, function () {
    console.log(`App started on port ${port}`)
});
