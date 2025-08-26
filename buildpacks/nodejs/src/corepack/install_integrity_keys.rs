use crate::{CorepackBuildpack, CorepackBuildpackError};
use heroku_nodejs_utils::vrs::{Requirement, Version};
use indoc::indoc;
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::UncachedLayerDefinition;
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};

pub(super) fn install_integrity_keys(
    context: &BuildContext<CorepackBuildpack>,
    corepack_version: &Version,
) -> Result<(), libcnb::Error<CorepackBuildpackError>> {
    // This is a workaround for Node versions that bundle a version of Corepack that is affected by
    // recent changes to npm's public signing keys:
    // * Corepack versions before 0.27.0 don't verify the integrity signatures from npm
    // * Corepack versions after 0.31.0 have the correct npm keys
    let patchable_corepack_versions =
        Requirement::parse(">= 0.27.0 || < 0.31.0").expect("This should be a valid range");

    if !patchable_corepack_versions.satisfies(corepack_version) {
        return Ok(());
    }

    let corepack_integrity_keys_layer = context.uncached_layer(
        layer_name!("corepack-integrity-keys"),
        UncachedLayerDefinition {
            build: true,
            launch: false,
        },
    )?;

    // these should line up with https://registry.npmjs.org/-/npm/v1/keys except Corepack
    // names the top-level property "npm" instead of "keys"
    // see: https://github.com/nodejs/corepack/pull/614
    let integrity_keys = indoc! {r#"
        {
          "npm": [
            {
              "expires": "2025-01-29T00:00:00.000Z",
              "keyid": "SHA256:jl3bwswu80PjjokCgh0o2w5c2U4LhQAE57gj9cz1kzA",
              "keytype": "ecdsa-sha2-nistp256",
              "scheme": "ecdsa-sha2-nistp256",
              "key": "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE1Olb3zMAFFxXKHiIkQO5cJ3Yhl5i6UPp+IhuteBJbuHcA5UogKo0EWtlWwW6KSaKoTNEYL7JlCQiVnkhBktUgg=="
            },
            {
              "expires": null,
              "keyid": "SHA256:DhQ8wR5APBvFHLF/+Tc+AYvPOdTpcIDqOhxsBHRwC7U",
              "keytype": "ecdsa-sha2-nistp256",
              "scheme": "ecdsa-sha2-nistp256",
              "key": "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEY6Ya7W++7aUPzvMTrezH6Ycx3c+HOKYCcNGybJZSCJq/fd7Qa8uuAKtdIkUQtQiEKERhAmE5lMMJhP8OkDOa2g=="
            }
          ]
        }
    "#};

    corepack_integrity_keys_layer.write_env(LayerEnv::new().chainable_insert(
        Scope::Build,
        ModificationBehavior::Override,
        "COREPACK_INTEGRITY_KEYS",
        integrity_keys,
    ))
}
