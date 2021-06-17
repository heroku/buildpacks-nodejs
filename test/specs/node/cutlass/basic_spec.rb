# frozen_string_literal: true

require_relative "../spec_helper"


describe "Heroku's Nodejs CNB" do
  it "handles a downgrade of node engine" do
    Cutlass::App.new("simple-function").transaction do |app|
      set_node_version_in_dir(app.tmpdir, version: "^14.0")

      app.pack_build do |pack_result|
        expect(pack_result.stdout).to include("Installing Node")
        expect(pack_result.stdout).to include("Downloading and extracting Node v14.")
      end

      set_node_version_in_dir(app.tmpdir, version: "^16.0")

      app.pack_build do |pack_result|
        expect(pack_result.stdout).to include("Installing Node")
        expect(pack_result.stdout).to include("Downloading and extracting Node v16.")
      end
    end
  end

  it "installs dependencies using Yarn if a yarn.lock exists" do
    # The Yarn buildpack currently doesn't write to the build plan, so has to be used alongside
    # the NPM buildpack rather than instead of it, which means we have to specify a custom
    # buildpacks list. See W-9482533.
    buildpacks = [test_dir.join("../meta-buildpacks/nodejs"), test_dir.join("../buildpacks/yarn")]
    Cutlass::App.new("yarn-project", buildpacks: buildpacks).transaction do |app|
      app.pack_build do |pack_result|
        expect(pack_result.stdout).to include("Installing Node")
        expect(pack_result.stdout).to include("Installing yarn")
      end
    end
  end
end
