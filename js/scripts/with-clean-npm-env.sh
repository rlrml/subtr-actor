#!/bin/sh

# Strip legacy npm-injected env vars that newer npm/node warn about when
# nested script runners hand them forward.
unset npm_config_argv
unset npm_config_version_git_tag
unset npm_config_version_commit_hooks
unset npm_config_version_tag_prefix
unset npm_config_version_git_message

exec "$@"
