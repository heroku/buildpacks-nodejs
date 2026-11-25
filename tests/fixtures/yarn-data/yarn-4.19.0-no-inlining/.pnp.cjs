#!/usr/bin/env node
/* eslint-disable */
// @ts-nocheck
"use strict";

function $$SETUP_STATE(hydrateRuntimeState, basePath) {
  const fs = require('fs');
  const path = require('path');
  const pnpDataFilepath = path.resolve(__dirname, ".pnp.data.json");
  return hydrateRuntimeState(JSON.parse(fs.readFileSync(pnpDataFilepath, 'utf8')), {basePath: basePath || __dirname});
}

// The rest of the Plug'n'Play runtime is omitted from this fixture.
