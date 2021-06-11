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
end
