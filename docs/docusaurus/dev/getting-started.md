---
sidebar_position: 1
slug: /
---

# Getting Started

## Environment setup

Follow the instructions below to set up your development environment for Substrate.

### Substrate Core

1. [Install Rust.](https://www.rust-lang.org/tools/install) Make sure you have at least version 1.70.0.
1. Install [just](https://github.com/casey/just) by running `cargo install just`.
1. Enable [git commit signing](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits).
1. Fork the [substrate2](https://github.com/substrate-labs/substrate2) repository.
1. Clone your fork.
1. From the root of your cloned `substrate2` repo, run `just check` and `just test`. Ensure these commands complete with no errors.

### Documentation

1. Install NodeJS. We recommend using a version manager like [nvm](https://github.com/nvm-sh/nvm).
1. Install [yarn](https://classic.yarnpkg.com/lang/en/docs/install/).
1. Change into the `docs/substrate` directory inside your cloned `substrate2` repo.
1. Run `yarn install`.
1. Run `yarn start`. Ensure that the documentation website opens in your browser and that there are no errors logged.

## Contributing guidelines

* Within-repository Rust dependencies should specify **both** a path **and** a version number.
* Commit messages should follow [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/).
  The body of the commit message should describe the changes that were made.
  These messages will be aggregated into `CHANGELOG`s.
* The first line of commit messages should be kept to less than 50 characters where possible.
* Each line in the commit body should be kept to 80 characters or less.
* All contributions should be submitted by opening a PR against this repository.
  If you do not have direct access to this repo, you should create a fork, make changes on your fork,
  and then submit your changes as a PR.
* After merging PRs, delete the branch.
