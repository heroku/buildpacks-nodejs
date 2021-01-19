#!/usr/bin/env bash
set -euo pipefail

released_buildpack_id="${1:?}"
released_buildpack_version="${2:?}"

released_buildpack_next_version=$(
	echo "${released_buildpack_version}" | awk -F. -v OFS=. '{ $NF=sprintf("%d\n", ($NF+1)); printf $0 }'
)

function escape_for_sed() {
	echo "${1:?}" | sed 's/[]\/\[\.]/\\&/g'
}

function is_meta_buildpack_with_dependency() {
	local -r buildpack_toml_path="${1:?}"
	local -r buildpack_id="${2:?}"

	yj -t <"${buildpack_toml_path}" | jq -e "[.order[]?.group[]?.id | select(. == \"${buildpack_id}\")] | length > 0" >/dev/null
}

# This is the heading we're looking for when updating CHANGELOG.md files
unreleased_heading=$(escape_for_sed "## [Unreleased]")

while IFS="" read -r -d "" buildpack_toml_path; do
	buildpack_id="$(yj -t <"${buildpack_toml_path}" | jq -r .buildpack.id)"
	buildpack_changelog_path="$(dirname "${buildpack_toml_path}")/CHANGELOG.md"

	jq_filter="."

	# Update the released buildpack itself
	if [[ "${buildpack_id}" == "${released_buildpack_id}" ]]; then
		if [[ -f "${buildpack_changelog_path}" ]]; then
			new_version_heading=$(escape_for_sed "## [${released_buildpack_version}] $(date +%Y/%m/%d)")
			sed -i "s/${unreleased_heading}/${unreleased_heading}\n\n${new_version_heading}/" "${buildpack_changelog_path}"
		fi

		jq_filter=".buildpack.version = \"${released_buildpack_next_version}\""

	# Update meta-buildpacks that have the released buildpack as a dependency
	elif is_meta_buildpack_with_dependency "${buildpack_toml_path}" "${released_buildpack_id}"; then
		if [[ -f "${buildpack_changelog_path}" ]]; then
			upgrade_entry=$(
				escape_for_sed "* Upgraded \`${released_buildpack_id}\` to \`${released_buildpack_next_version}\`"
			)

			sed -i "s/${unreleased_heading}/${unreleased_heading}\n${upgrade_entry}/" "${buildpack_changelog_path}"
		fi

		jq_filter=$(
			cat <<-EOF
				.order |= map(.group |= map(
					if .id == "${released_buildpack_id}" then
						.version |= "${released_buildpack_next_version}"
					else
						.
					end
				))
			EOF
		)
	fi

	# Write the filtered buildpack.toml to disk...
	updated=$(yj -t <"${buildpack_toml_path}" | jq "${jq_filter}" | yj -jt)
	echo "${updated}" >"${buildpack_toml_path}"

done < <(find . -name buildpack.toml -print0)
