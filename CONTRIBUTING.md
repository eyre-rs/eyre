# Welcome to the eyre contributing guide

Thank you for investing your time in contributing to our project! Eyre is a
community owned and maintained project dedicated to improving the error
handling and error reporting experience of users of the Rust programming
language.

Check out our community's[^1] [Code of
Conduct](https://www.rust-lang.org/policies/code-of-conduct) and feel free to
say hi on [Discord] if you'd like. It's a nice place to chat about eyre
development, ask questions, and get to know the other contributors and users in
a less formal setting.

## The Eyre Organization

The Eyre Organization is the group of people responsible for stewarding the
Eyre project. It handles things like merging pull requests, choosing project
direction, managing bugs / issues / feature requests, controlling access to
secrets, defining and enforcing best practices, etc.

The eyre organization's governance is based on and inspired by
[sociocracy](https://www.sociocracyforall.org/sociocracy/), the Rust Project,
and the Bevy Organization. Many thanks to their great examples and resources.

Note that you *do not* need to be a member of the Eyre Organization to
contribute to Eyre. Community contributors (this means you) can freely open
issues, submit pull requests, and review pull requests.

### New contributor guide

To get an overview of the project, read the [README](README.md). Here are some
resources to help you get started with open source contributions:

- [Finding ways to contribute to open source on GitHub](https://docs.github.com/en/get-started/exploring-projects-on-github/finding-ways-to-contribute-to-open-source-on-github)
- [Set up Git](https://docs.github.com/en/get-started/quickstart/set-up-git)
- [GitHub flow](https://docs.github.com/en/get-started/quickstart/github-flow)
- [Collaborating with pull requests](https://docs.github.com/en/github/collaborating-with-pull-requests)

Your first PR will be merged in no time!

No matter how you're helping: thank you for contributing to Eyre!

### Classifying PRs

We use labels to organize our issues and PRs.

Each [label](https://github.com/eyre-rs/eyre/labels) has a prefix denoting its category:

* A: Area or subcrate (e.g. A-eyre, A-color-eyre, A-color-spantrace)
* C: Category (e.g. C-Breaking-Change, C-Code-Quality, C-Docs)
* P: Priority (e.g. P-Urgent, P-Important)
* S: Status (e.g. S-Blocked, S-Controversial, S-Needs-Design)
* Misc (e.g. "good first issue", "help wanted", "duplicate", "invalid", "wontfix")

## Making changes to Eyre

Most changes don't require much process. If your change is relatively straightforward, just do the following:

1. A community member (that's you!) creates one of the following:
    * [GitHub Discussions]: An informal discussion with the community. This is
      the place to start if you want to propose a feature or specific
      implementation and gathering community wisdom and advice before jumping
      to solutions.
    * [Issue](https://github.com/eyre-rs/eyre/issues): A formal way for us to
      track a bug or feature. Please look for duplicates before opening a new
      issue and consider starting with a Discussion.
    * [Pull Request](https://github.com/eyre-rs/eyre/pulls) (or PR for short):
      A request to merge code changes. This starts our "review process". You
      are welcome to start with a pull request, but consider starting with an
      Issue or Discussion for larger changes (or if you aren't certain about a
      design). We don't want anyone to waste their time on code that didn't
      have a chance to be merged! But conversely, sometimes PRs are the most
      efficient way to propose a change. Just use your own judgement here.
2. Other community members review and comment in an ad-hoc fashion. Active
   subject matter experts may be pulled into a thread using `@mentions`. If
   your PR has been quiet for a while and is ready for review, feel free to
   leave a message to "bump" the thread, or bring it up on [Discord]
3. Once they're content with the pull request (design, code quality,
   documentation, tests), individual reviewers leave "Approved" reviews.
4. After consensus has been reached (typically two approvals from the community
   or one for extremely simple changes) and CI passes, the
   [S-Ready-For-Final-Review](https://github.com/eyre-rs/eyre/issues?q=is%3Aopen+is%3Aissue+label%3AS-Ready-For-Final-Review)
   label is added.
5. When they find time, someone with merge rights performs a final code review
   and queue the PR for merging.

## How you can help

If you've made it to this page, you're probably already convinced that Eyre is
a project you'd like to see thrive. But how can *you* help?

No matter your experience level with Eyre or Rust or your level of commitment,
there are ways to meaningfully contribute. Take a look at the sections that
follow to pick a route (or five) that appeal to you.

If you ever find yourself at a loss for what to do, or in need of mentorship or
advice on how to contribute to Eyre, feel free to ask in [Discord] and one of
our more experienced community members will be happy to help.

### Writing Handlers

You can improve Eyre's ecosystem by building your own
[EyreHandler](https://docs.rs/eyre/0.6.8/eyre/trait.EyreHandler.html) crates
like [color-eyre](https://github.com/eyre-rs/eyre/tree/master/color-eyre/). The customizable
reporting of `eyre` is it's secret sauce, using that customizability in
creative ways and sharing your work is one of the best ways you can inspire
others and help grow our community.

### Fixing bugs

Bugs in Eyre are filed on the issue tracker using the [`C-bug`](https://github.com/eyre-rs/eyre/issues?q=is%3Aissue+is%3Aopen+label%3AC-Bug) label.

If you're looking for an easy place to start, take a look at the [`good first
issue`](https://github.com/eyre-rs/eyre/labels/good%20first%20issue) label, and
feel free to ask questions on that issue's thread in question or on [Discord].
You don't need anyone's permission to try fixing a bug or adding a simple
feature, but stating that you'd like to tackle an issue can be helpful to avoid
duplicated work.

When you make a pull request that fixes an issue, include a line that says
`Fixes #X` (or "Closes"), where `X` is the issue number. This will cause the
issue in question to be closed when your PR is merged.

General improvements to code quality are also welcome!
Eyre can always be safer, better tested, and more idiomatic.

### Writing docs

This is incredibly valuable, easily distributed work, but requires a bit of guidance:

* Inaccurate documentation is worse than no documentation: prioritize fixing
  broken docs.
* Code documentation (doc examples and in the examples folder) is easier to
  maintain because the compiler will tell us when it breaks.
* Inline documentation should be technical and to the point. Link relevant
  examples or other explanations if broader context is useful.

### Reviewing others' work

Reviewing others work with the aim of improving it is one of the most helpful
things you can do. You don't need to be an Elder Rustacean to be useful here:
anyone can catch missing tests, unclear docs, logic errors, and so on. If you
have specific skills (e.g. advanced familiarity with `unsafe` code, rendering
knowledge or web development experience) or personal experience with a problem,
try to prioritize those areas to ensure we can get appropriate expertise where
we need it.

Focus on giving constructive, actionable feedback that results in real
improvements to code quality or end-user experience. If you don't understand
why an approach was taken, please ask!

Provide actual code suggestions when that is helpful. Small changes work well
as comments or in-line suggestions on specific lines of codes. Larger changes
deserve a comment in the main thread, or a pull request to the original
author's branch (but please mention that you've made one).

Once you're happy with the work and feel you're reasonably qualified to assess
quality in this particular area, leave your `Approved` review on the PR. If
you're new to GitHub, check out the [Pull Request Review
documentation](https://docs.github.com/en/github/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/about-pull-request-reviews).
Anyone can leave reviews ... no special permissions are required!

There are three main places you can check for things to review:

1. Pull request which are ready and in need of more reviews on
   [eyre](https://github.com/eyre-rs/eyre/pulls?q=is%3Aopen+is%3Apr+-label%3AS-Ready-For-Final-Review+-draft%3A%3Atrue+-label%3AS-Needs-RFC+-reviewed-by%3A%40me+-author%3A%40me)
2. Pull requests on [eyre](https://github.com/eyre-rs/eyre/pulls) repo.

Not even our Circle Members are exempt from reviews! By giving feedback on this
work (and related supporting work), you can help us make sure our releases are
both high-quality and timely.

Finally, if nothing brings you more satisfaction than seeing every last issue
labeled and all resolved issues closed, feel free to message any Eyre Circle
Member (currently @yaahc) for the triage role to help us keep things tidy. This
role only requires good faith and a basic understanding of our development
process.

[Discord]: https://discord.gg/z94RqmUTKB
[^1]: Okay, I'll admit it, it's really just the Rust Project's CoC :sweat_smile:
