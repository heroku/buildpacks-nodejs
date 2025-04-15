// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use heroku_nodejs_engine_buildpack::NodeJsEngineBuildpack;
use libcnb::buildpack_main;

buildpack_main!(NodeJsEngineBuildpack);
