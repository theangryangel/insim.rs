# Contributing to insim.rs

Thanks for your interest in helping to improve the project.

**No contribution is too small and all contributions are valued.**

## Asking for General Help

If you have reviewed existing documentation and still have questions or are
having problems, you can open an issue asking for help.

In exchange for receiving help, we ask that you contribute back a documentation
PR that helps others avoid the problems that you encountered.

## Submitting a Bug Report

The two most important pieces of information we need in order to properly
evaluate the a bug report is a description of the behavior you are seeing and a simple
test case we can use to recreate the problem on our own. If we cannot recreate
the issue, it becomes harder for to fix.

See [How to create a Minimal, Complete, and Verifiable example][mcve].

[mcve]: https://stackoverflow.com/help/mcve

## Triaging a Bug Report

Once an issue has been opened, it is not uncommon for there to be discussion
around it. Some contributors may have differing opinions about the issue,
including whether the behavior being seen is a bug or a feature. This discussion
is part of the process and should be kept focused, helpful, and professional.

Short, clipped responses—that provide neither additional context nor supporting
detail—are not helpful or professional. To many, such responses are simply
annoying and unfriendly.

## Resolving a Bug Report

In the majority of cases, issues are resolved by opening a Pull Request. The
process for opening and reviewing a Pull Request is similar to that of opening
and triaging issues, but carries with it a necessary review and approval
workflow that ensures that the proposed changes meet the minimal quality.

## Commits, Commit Messages and Linting

- We are not fussy about commit messages, but just ask that you try and keep
  them clean. [How to write a commit message](https://chris.beams.io/posts/git-commit/) may be helpful.
- We use [pre-commit](https://pre-commit.com/) to help run all of our linting.
  Before you raise a Pull Request, ensure that you've installed pre-commit, and
  run pre-commit. This will execute rustfmt, clippy, etc.
  ```bash
  pip install pre-commit
  pre-commit run --all
  ```

## Behaviour and Code of Conduct

We have no formal code of conduct at this time. But we do subscribe to
[Wheaton's law](https://knowyourmeme.com/memes/wheatons-law).

- Be aware of the person behind the code
- Be aware of how you communicate requests and give feedback

## Releasing

Ideally this should be fully automated. At present it is not.

- Create PR to change version in all crates to match (probably only just insim_core and insim realistically)
  - Locally run `cargo publish --dry-run` to ensure everything is OK
  - Wait for CI to go green
  - Merge
- Create GitHub release (and tag)
- `cargo publish` for each of insim_core, optionally insim_pth, optionally insim_smx and finally insim
- Panic
