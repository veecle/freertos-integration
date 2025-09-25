# Contributing

## Contributing Workflow

We welcome contributions from the community!
Follow this workflow to ensure smooth review and integration.

### Before You Start

1. Browse existing work: Check [issues](https://github.com/veecle/freertos-integration/issues) to find tasks that need help.
2. Discuss large changes: For significant features or architectural changes, open an issue to discuss the approach with maintainers.
3. Claim your work: Comment on issues to indicate you're working on them to avoid duplicate effort.
4. Read our guidelines: Familiarize yourself with the [Code of Conduct](https://github.com/veecle/freertos-integration?tab=coc-ov-file#readme) and this contribution guide.

### Developer certificate of origin (DCO)

To make a good faith effort to ensure licensing criteria are met, this repository requires a DCO process to be followed.
You must sign-off the DCO that you can see at <https://developercertificate.org/> to contribute to the repository by adding a sign-off to your commits.
Use `git commit -s` to sign off your commits.
Refer to [the Developer Certificate of Origin GitHub app](https://probot.github.io/apps/dco/) for details on how this repository enforces sign-off.

### Submitting Changes

1. Make your changes and test them with:
   - `cargo test` and `cargo clippy` at the workspace root will check Rust code.
   - [`taplo fmt`](https://github.com/tamasfe/taplo) will format `.toml` files.
2. Commit with [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format.
   We use squash-merge with the PR title and description as commit title and message.
   Please format your PR title in accordance with Conventional Commits.
3. Push your branch to your fork and open a Pull Request.
4. Address feedback from maintainers as needed.

### Getting Help

- **Issue comments**: For specific questions on relevant issues.
- **Style guide**: Check the [style guide](https://veecle.github.io/style-guide/).

### Submodules

This repository contains a git submodule (see [`FreeRTOS-Kernel`](FreeRTOS-Kernel)).

Submodules are not automatically updated during `git pull`, which may cause problems if any of the submodules gets outdated.
You might need to run the following command to update them:

```bash
git submodule update --init --recursive
```

Also consider using [the `submodule.recurse` Git configuration option to automate submodule updates](https://git-scm.com/docs/git-config#Documentation/git-config.txt-submodulerecurse).
