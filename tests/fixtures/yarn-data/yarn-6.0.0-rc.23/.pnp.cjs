#!/usr/bin/env node
/* eslint-disable */
// @ts-nocheck
"use strict";

const RAW_RUNTIME_STATE =
'{\
  "enableTopLevelFallback": true,\
  "fallbackPool": [\
    [\
      "@parcel/watcher-win32-x64",\
      "npm:2.6.0"\
    ],\
    [\
      "debug",\
      "virtual:94c925cbd515c10bb754d6269936e412a410ab416ff0607304ecd6e14c88582df4d0f5714a4a249e7723d251607a23195370bf3abcb9182f456403f18a0c9f3b#npm:4.3.4"\
    ],\
    [\
      "depd",\
      "npm:2.0.0"\
    ],\
    [\
      "function-bind",\
      "npm:1.1.2"\
    ],\
    [\
      "hasown",\
      "npm:2.0.4"\
    ],\
    [\
      "is-core-module",\
      "npm:2.18.0"\
    ],\
    [\
      "local-pkg",\
      "file:./vendor/local-pkg#253de5d5b1e908c64d72af62517817e3bfbf7a4c3d310d94853053a5592e0fea358257a82f55421028a25612c90b737ca60283980a430639ce8812a7ce40e9e8::parent=pnp-project@workspace:pnp-project"\
    ],\
    [\
      "ms",\
      "npm:2.1.2"\
    ],\
    [\
      "my-ms",\
      "npm:ms@2.1.3"\
    ],\
    [\
      "path-parse",\
      "npm:1.0.7"\
    ],\
    [\
      "portal-pkg",\
      "portal:./vendor/portal-pkg#1c52f2586bf2eafa9cca427739219caab6f8562c41db7bd8ab6c8e965aea660837b729af032fe0dcc5daf31ab285ab24a56a93b017518fbbe9271685562b6344::parent=pnp-project@workspace:pnp-project"\
    ],\
    [\
      "resolve",\
      "patch:resolve%40npm%3A1.22.8#<builtin>&checksum=85eaa72caadee6a5622c928b1473f16d3507770cd417f35e56c48bcc9b50a1d71dfd49ad5a227767d79fdf331a578e26ef8045e83e3f7356f72a1412ae2be199"\
    ],\
    [\
      "supports-preserve-symlinks-flag",\
      "npm:1.0.0"\
    ]\
  ],\
  "fallbackExclusionList": [\
    [\
      "lib",\
      [\
        "workspace:lib"\
      ]\
    ],\
    [\
      "pnp-project",\
      [\
        "workspace:pnp-project"\
      ]\
    ]\
  ],\
  "ignorePatternData": null,\
  "packageRegistryData": [\
    [\
      null,\
      [\
        [\
          null,\
          {\
            "packageLocation": "./",\
            "packageDependencies": [\
              [\
                "@parcel/watcher-win32-x64",\
                "npm:2.6.0"\
              ],\
              [\
                "debug",\
                "virtual:94c925cbd515c10bb754d6269936e412a410ab416ff0607304ecd6e14c88582df4d0f5714a4a249e7723d251607a23195370bf3abcb9182f456403f18a0c9f3b#npm:4.3.4"\
              ],\
              [\
                "depd",\
                "npm:2.0.0"\
              ],\
              [\
                "lib",\
                "workspace:lib"\
              ],\
              [\
                "local-pkg",\
                "file:./vendor/local-pkg#253de5d5b1e908c64d72af62517817e3bfbf7a4c3d310d94853053a5592e0fea358257a82f55421028a25612c90b737ca60283980a430639ce8812a7ce40e9e8::parent=pnp-project@workspace:pnp-project"\
              ],\
              [\
                "my-ms",\
                "npm:ms@2.1.3"\
              ],\
              [\
                "pnp-project",\
                "workspace:pnp-project"\
              ],\
              [\
                "portal-pkg",\
                "portal:./vendor/portal-pkg#1c52f2586bf2eafa9cca427739219caab6f8562c41db7bd8ab6c8e965aea660837b729af032fe0dcc5daf31ab285ab24a56a93b017518fbbe9271685562b6344::parent=pnp-project@workspace:pnp-project"\
              ],\
              [\
                "resolve",\
                "patch:resolve%40npm%3A1.22.8#<builtin>&checksum=85eaa72caadee6a5622c928b1473f16d3507770cd417f35e56c48bcc9b50a1d71dfd49ad5a227767d79fdf331a578e26ef8045e83e3f7356f72a1412ae2be199"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "SOFT",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "@parcel/watcher-win32-x64",\
      [\
        [\
          "npm:2.6.0",\
          {\
            "packageLocation": "./.yarn/unplugged/@parcel-watcher-win32-x64-npm-2.6.0-603748a51f/node_modules/@parcel/watcher-win32-x64/",\
            "packageDependencies": [\
              [\
                "@parcel/watcher-win32-x64",\
                "npm:2.6.0"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "HARD",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "debug",\
      [\
        [\
          "virtual:94c925cbd515c10bb754d6269936e412a410ab416ff0607304ecd6e14c88582df4d0f5714a4a249e7723d251607a23195370bf3abcb9182f456403f18a0c9f3b#npm:4.3.4",\
          {\
            "packageLocation": "./.yarn/__virtual__/hash-94c925cbd515c10b/2/yarn_cache/debug-npm-4.3.4-41fcf71dbe027d1887a6678edea971b8-2.zip/node_modules/debug/",\
            "packageDependencies": [\
              [\
                "@types/supports-color",\
                null\
              ],\
              [\
                "debug",\
                "virtual:94c925cbd515c10bb754d6269936e412a410ab416ff0607304ecd6e14c88582df4d0f5714a4a249e7723d251607a23195370bf3abcb9182f456403f18a0c9f3b#npm:4.3.4"\
              ],\
              [\
                "ms",\
                "npm:2.1.2"\
              ],\
              [\
                "supports-color",\
                null\
              ]\
            ],\
            "packagePeers": [\
              "@types/supports-color",\
              "supports-color"\
            ],\
            "linkType": "HARD",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "depd",\
      [\
        [\
          "npm:2.0.0",\
          {\
            "packageLocation": "./.yarn/unplugged/depd-npm-2.0.0-b6c51a4b43/node_modules/depd/",\
            "packageDependencies": [\
              [\
                "depd",\
                "npm:2.0.0"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "HARD",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "function-bind",\
      [\
        [\
          "npm:1.1.2",\
          {\
            "packageLocation": "../yarn_cache/function-bind-npm-1.1.2-ef8a45d8ef2a0e6ccb3d5e4e8254e788-2.zip/node_modules/function-bind/",\
            "packageDependencies": [\
              [\
                "function-bind",\
                "npm:1.1.2"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "HARD",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "hasown",\
      [\
        [\
          "npm:2.0.4",\
          {\
            "packageLocation": "../yarn_cache/hasown-npm-2.0.4-2f63053f7b01ee57ed1ae03b1a20b9ba-2.zip/node_modules/hasown/",\
            "packageDependencies": [\
              [\
                "function-bind",\
                "npm:1.1.2"\
              ],\
              [\
                "hasown",\
                "npm:2.0.4"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "HARD",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "is-core-module",\
      [\
        [\
          "npm:2.18.0",\
          {\
            "packageLocation": "../yarn_cache/is-core-module-npm-2.18.0-dd4e62a605e2503938e65a027f35a19f-2.zip/node_modules/is-core-module/",\
            "packageDependencies": [\
              [\
                "hasown",\
                "npm:2.0.4"\
              ],\
              [\
                "is-core-module",\
                "npm:2.18.0"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "HARD",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "lib",\
      [\
        [\
          "workspace:lib",\
          {\
            "packageLocation": "./packages/lib/",\
            "packageDependencies": [\
              [\
                "lib",\
                "workspace:lib"\
              ],\
              [\
                "ms",\
                "npm:2.1.2"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "SOFT",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "local-pkg",\
      [\
        [\
          "file:./vendor/local-pkg#253de5d5b1e908c64d72af62517817e3bfbf7a4c3d310d94853053a5592e0fea358257a82f55421028a25612c90b737ca60283980a430639ce8812a7ce40e9e8::parent=pnp-project@workspace:pnp-project",\
          {\
            "packageLocation": "./.yarn/unplugged/local-pkg-file-89849978cf/node_modules/local-pkg/",\
            "packageDependencies": [\
              [\
                "local-pkg",\
                "file:./vendor/local-pkg#253de5d5b1e908c64d72af62517817e3bfbf7a4c3d310d94853053a5592e0fea358257a82f55421028a25612c90b737ca60283980a430639ce8812a7ce40e9e8::parent=pnp-project@workspace:pnp-project"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "HARD",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "ms",\
      [\
        [\
          "npm:2.1.2",\
          {\
            "packageLocation": "../yarn_cache/ms-npm-2.1.2-acef7ddf8df80b780eebf5ec41c7ebb0-2.zip/node_modules/ms/",\
            "packageDependencies": [\
              [\
                "ms",\
                "npm:2.1.2"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "HARD",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "my-ms",\
      [\
        [\
          "npm:ms@2.1.3",\
          {\
            "packageLocation": "../yarn_cache/my-ms-npm-2.1.3-144affa8c9ddb7887b8fd13436fb0276-2.zip/node_modules/ms/",\
            "packageDependencies": [\
              [\
                "my-ms",\
                "npm:ms@2.1.3"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "HARD",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "path-parse",\
      [\
        [\
          "npm:1.0.7",\
          {\
            "packageLocation": "../yarn_cache/path-parse-npm-1.0.7-72757e7bb6616fa5a1042133b13f2057-2.zip/node_modules/path-parse/",\
            "packageDependencies": [\
              [\
                "path-parse",\
                "npm:1.0.7"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "HARD",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "pnp-project",\
      [\
        [\
          "workspace:pnp-project",\
          {\
            "packageLocation": "./",\
            "packageDependencies": [\
              [\
                "@parcel/watcher-win32-x64",\
                "npm:2.6.0"\
              ],\
              [\
                "debug",\
                "virtual:94c925cbd515c10bb754d6269936e412a410ab416ff0607304ecd6e14c88582df4d0f5714a4a249e7723d251607a23195370bf3abcb9182f456403f18a0c9f3b#npm:4.3.4"\
              ],\
              [\
                "depd",\
                "npm:2.0.0"\
              ],\
              [\
                "lib",\
                "workspace:lib"\
              ],\
              [\
                "local-pkg",\
                "file:./vendor/local-pkg#253de5d5b1e908c64d72af62517817e3bfbf7a4c3d310d94853053a5592e0fea358257a82f55421028a25612c90b737ca60283980a430639ce8812a7ce40e9e8::parent=pnp-project@workspace:pnp-project"\
              ],\
              [\
                "my-ms",\
                "npm:ms@2.1.3"\
              ],\
              [\
                "pnp-project",\
                "workspace:pnp-project"\
              ],\
              [\
                "portal-pkg",\
                "portal:./vendor/portal-pkg#1c52f2586bf2eafa9cca427739219caab6f8562c41db7bd8ab6c8e965aea660837b729af032fe0dcc5daf31ab285ab24a56a93b017518fbbe9271685562b6344::parent=pnp-project@workspace:pnp-project"\
              ],\
              [\
                "resolve",\
                "patch:resolve%40npm%3A1.22.8#<builtin>&checksum=85eaa72caadee6a5622c928b1473f16d3507770cd417f35e56c48bcc9b50a1d71dfd49ad5a227767d79fdf331a578e26ef8045e83e3f7356f72a1412ae2be199"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "SOFT",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "portal-pkg",\
      [\
        [\
          "portal:./vendor/portal-pkg#1c52f2586bf2eafa9cca427739219caab6f8562c41db7bd8ab6c8e965aea660837b729af032fe0dcc5daf31ab285ab24a56a93b017518fbbe9271685562b6344::parent=pnp-project@workspace:pnp-project",\
          {\
            "packageLocation": "./vendor/portal-pkg/",\
            "packageDependencies": [\
              [\
                "portal-pkg",\
                "portal:./vendor/portal-pkg#1c52f2586bf2eafa9cca427739219caab6f8562c41db7bd8ab6c8e965aea660837b729af032fe0dcc5daf31ab285ab24a56a93b017518fbbe9271685562b6344::parent=pnp-project@workspace:pnp-project"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "SOFT",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "resolve",\
      [\
        [\
          "patch:resolve%40npm%3A1.22.8#<builtin>&checksum=85eaa72caadee6a5622c928b1473f16d3507770cd417f35e56c48bcc9b50a1d71dfd49ad5a227767d79fdf331a578e26ef8045e83e3f7356f72a1412ae2be199",\
          {\
            "packageLocation": "../yarn_cache/resolve-patch-9d8fc104b1662cb52f56a9f7d765761c-2.zip/node_modules/resolve/",\
            "packageDependencies": [\
              [\
                "is-core-module",\
                "npm:2.18.0"\
              ],\
              [\
                "path-parse",\
                "npm:1.0.7"\
              ],\
              [\
                "resolve",\
                "patch:resolve%40npm%3A1.22.8#<builtin>&checksum=85eaa72caadee6a5622c928b1473f16d3507770cd417f35e56c48bcc9b50a1d71dfd49ad5a227767d79fdf331a578e26ef8045e83e3f7356f72a1412ae2be199"\
              ],\
              [\
                "supports-preserve-symlinks-flag",\
                "npm:1.0.0"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "HARD",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ],\
    [\
      "supports-preserve-symlinks-flag",\
      [\
        [\
          "npm:1.0.0",\
          {\
            "packageLocation": "../yarn_cache/supports-preserve-symlinks-flag-npm-1.0.0-fa6172b5af0b6395f8f9d10e3836906b-2.zip/node_modules/supports-preserve-symlinks-flag/",\
            "packageDependencies": [\
              [\
                "supports-preserve-symlinks-flag",\
                "npm:1.0.0"\
              ]\
            ],\
            "packagePeers": [],\
            "linkType": "HARD",\
            "discardFromLookup": false\
          }\
        ]\
      ]\
    ]\
  ],\
  "dependencyTreeRoots": [\
    {\
      "name": "lib",\
      "reference": "workspace:lib"\
    },\
    {\
      "name": "pnp-project",\
      "reference": "workspace:pnp-project"\
    }\
  ]\
}';

function $$SETUP_STATE(hydrateRuntimeState, basePath) {
  return hydrateRuntimeState(JSON.parse(RAW_RUNTIME_STATE), {basePath: basePath || __dirname});
}
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const os = require('os');
const events = require('events');
const nodeUtils = require('util');
const stream = require('stream');
const zlib = require('zlib');
const require$$0 = require('module');
const StringDecoder = require('string_decoder');
const url = require('url');
const buffer = require('buffer');
const readline = require('readline');
const assert = require('assert');

// The rest of the Plug'n'Play runtime is omitted from this fixture.
