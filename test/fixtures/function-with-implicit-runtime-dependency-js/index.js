module.exports = async function (event, context, logger) {
    logger.info("logging info is a fun 1")
    return "Hello World".toLowerCase();
}
