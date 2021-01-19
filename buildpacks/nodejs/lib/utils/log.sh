#!/usr/bin/env bash

status() {
  local color="\033[1;35m"
  local no_color="\033[0m"
  echo -e "\n${color}[${1:-""}]${no_color}"
}

info() {
  echo -e "[INFO] ${1:-""}"
}

error() {
  local color="\033[1;31m"
  local no_color="\033[0m"

  echo -e "\n${color}[Error: ${1:-""}]${no_color}\n"
}

warning() {
  local color="\033[1;33m"
  local no_color="\033[0m"

  echo -e "\n${color}[Warning: ${1:-""}]${no_color}\n"
}

notice() {
  local color="\033[1;34m"
  local no_color="\033[0m"

  echo -e "\n${color}[Notice: ${1:-""}]${no_color}\n"
}
