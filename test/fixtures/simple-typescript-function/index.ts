"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
async function execute(event, context, logger) {
  logger.info(
    `Invoking Myfnts with payload ${JSON.stringify(event.data || {})}`
  );

  return "Hello World from typescript".toLowerCase();
}
exports.default = execute;
