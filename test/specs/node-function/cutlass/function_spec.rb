# frozen_string_literal: true

require_relative "../spec_helper"

describe "Heroku's Nodejs CNB" do
  it "generates a callable salesforce function" do
    Cutlass::App.new("simple-function").transaction do |app|
      app.pack_build do |pack_result|
        expect(pack_result.stdout).to include("Installing Node.js function runtime")
      end

      app.start_container(expose_ports: 8080) do |container|
        body = { }
        query = Cutlass::FunctionQuery.new(
          port: container.get_host_port(8080),
          body: body
        ).call

        expect(query.as_json).to eq("hello world")
        expect(query.success?).to be_truthy
      end
    end
  end

  it "generates a callable salesforce function from typescript" do
    Cutlass::App.new("simple-typescript-function").transaction do |app|
      app.pack_build do |pack_result|
        expect(pack_result.stdout).to include("Installing Node.js function runtime")
      end

      app.start_container(expose_ports: 8080) do |container|
        body = { }
        query = Cutlass::FunctionQuery.new(
          port: container.get_host_port(8080),
          body: body
        ).call

        expect(query.as_json).to eq("hello world from typescript")
        expect(query.success?).to be_truthy
      end
    end
  end
end
