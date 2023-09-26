# frozen_string_literal: true

require "rspec/core"
require "rspec/retry"

require "cutlass"

def test_dir
  Pathname(__dir__).join("../..")
end

NODEJS_FUNCTION_BUILDPACK = Cutlass::LocalBuildpack.new(directory: test_dir.join("../meta-buildpacks/nodejs-function"))
Cutlass.config do |config|
  config.default_buildpack_paths = [NODEJS_FUNCTION_BUILDPACK]
  config.default_builder = "heroku/builder:20"
  config.default_repo_dirs = [test_dir.join("fixtures")]
end

RSpec.configure do |config|
  # config.filter_run :focus => true

  config.before(:suite) do
    Cutlass::CleanTestEnv.record
  end

  config.after(:suite) do
    NODEJS_FUNCTION_BUILDPACK.teardown
    Cutlass::CleanTestEnv.check
  end
end

