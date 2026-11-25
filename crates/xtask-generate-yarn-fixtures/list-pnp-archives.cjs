// Prints the package archives the Plug'n'Play runtime loads, one per line, using Yarn's own
// Plug'n'Play API. The output is written to each fixture's `expected-archives.txt` so the
// buildpack's parsing of the Plug'n'Play data is checked against Yarn itself.
const path = require("path");
const pnp = require("pnpapi");

function getLocators() {
  // Plug'n'Play API versions before 3 (Yarn 2) have no `getAllLocators`, so walk the dependency
  // tree from its roots instead.
  if (pnp.getAllLocators) {
    return pnp.getAllLocators();
  }
  const seen = new Map();
  const queue = [...pnp.getDependencyTreeRoots()];
  while (queue.length > 0) {
    const locator = queue.pop();
    const key = `${locator.name}@${locator.reference}`;
    if (seen.has(key)) continue;
    seen.set(key, locator);
    const information = pnp.getPackageInformation(locator);
    for (const [name, reference] of information.packageDependencies) {
      if (reference === null) continue;
      queue.push(
        Array.isArray(reference)
          ? { name: reference[0], reference: reference[1] }
          : { name, reference },
      );
    }
  }
  return [...seen.values()];
}

const archives = new Set();
for (const locator of getLocators()) {
  const information = pnp.getPackageInformation(locator);
  if (!information) continue;
  // Virtual packages live under `.yarn/__virtual__`; resolve them to their real location.
  const location = pnp.resolveVirtual(information.packageLocation) ?? information.packageLocation;
  const archive = location.split(path.sep).find((segment) => segment.endsWith(".zip"));
  if (archive) archives.add(archive);
}
for (const archive of [...archives].sort()) {
  console.log(archive);
}
