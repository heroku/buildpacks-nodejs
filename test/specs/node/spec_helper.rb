# frozen_string_literal: true

require "rspec/core"
require "rspec/retry"

require "cutlass"

def test_dir
  Pathname(__dir__).join("../..")
end

NODEJS_BUILDPACK = Cutlass::LocalBuildpack.new(directory: test_dir.join("meta-buildpacks/nodejs"))
Cutlass.config do |config|
  config.default_buildpack_paths = [NODEJS_BUILDPACK]
  config.default_builder = "heroku/buildpacks:20"
  config.default_repo_dirs = [test_dir.join("fixtures")]
end

RSpec.configure do |config|
  # config.filter_run :focus => true

  config.before(:suite) do
    Cutlass::CleanTestEnv.record
  end

  config.after(:suite) do
    NODEJS_BUILDPACK.teardown
    Cutlass::CleanTestEnv.check
  end
end

def set_node_version_in_dir(dir, version: )
  package_json = Pathname(dir).join("package.json")
  package_json_hash = JSON.parse(package_json.read)
  package_json_hash["engines"]["node"] = version
  package_json.write(package_json_hash.to_json)
end
