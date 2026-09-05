# cargo-perf roadmap

Last updated 2026-09-05. Angle: ship first. Items are ordered by time-to-user-value.

## Where we are

We are on 0.6.0, tagged 2026-01-13, with 327 crates.io downloads (94 recent). About 41 substantive commits have landed since, including the whole accuracy-hardening arc (PR #34), and none of it has reached a user. The scorecard holds 1.00 precision and recall across 14 rules over 80 fixtures, and CI is green with no open issues. The biggest lever is not another rule: the documented onboarding path is broken (the Action passes flags clap cannot parse; the `cargo perf` subcommand path has zero tests), so the work already done is unreachable. Fix the front door, then cut 0.7.0.

## Now (next 2 to 4 weeks) - what must be true to cut v0.7.0

**Make the shipped CLI parse the commands our own docs use.** Adoption dies at the first command; today the official Action pipes a parse error into results.sarif and reports zero findings. Acceptance: `format`, `fail_on`, `min_severity`, `rules` marked `#[arg(global = true)]` (or added to `Commands::Check`); action.yml:126,135, examples/github-workflow.yml:53,90 and docs/getting-started.md:96 updated; a tests/cli.rs case runs `check --format sarif --fail-on error` exactly as the Action does. S. [adopt-1]

**Test the `cargo perf` subcommand invocation end to end.** Every doc tells users to run it this way and nothing exercises it. Acceptance: integration test invoking a PATH-installed `cargo-perf perf check <path>` and asserting parity with `cargo-perf check <path>`; if it fails, move the `args.get(1) == Some("perf")` strip ahead of the first `Cli::parse()` in src/main.rs; ci.yml's dogfood step calls `cargo perf check src/`. S. [adopt-3]

**Cut 0.7.0 with an honest changelog.** Two user-visible behavior changes shipped since 0.6.0 and neither is recorded. Acceptance: Cargo.toml at 0.7.0; the existing CHANGELOG `[Unreleased]` section extended with the lock-across-await severity split (b17b95e) and the collect-then-iterate autofix removal with its D18 rationale (01e3f7b); README rule table notes collect-then-iterate as advisory only. S. [ship-5, docs-1, docs-2]

**Make the release pipeline unable to ship a lie.** Version drift in a rigor-branded tool is self-refuting. Acceptance: release.yml gains an early step failing on `${GITHUB_REF_NAME#v}` not matching Cargo.toml's version, and a `cargo publish --locked` job gated on `CARGO_REGISTRY_TOKEN`; CONTRIBUTING.md gains a numbered release checklist. M. [ship-6, ship-3]

**Make both advertised install paths real.** README's first-listed install and its headline Action example both currently fail. Acceptance: `v1` tag pushed at the 0.7.0 release commit (or README pinned to the real tag until then); `cargo binstall cargo-perf` smoke-tested on macOS and Linux from a clean environment after publish, added to the release checklist. S. [ship-2, ship-4]

**Fix the config schema and the explain gaps.** Two of fourteen rules are rejected by our own published schema and give a non-answer in `cargo perf explain`. Acceptance: `hashmap-no-capacity` and `string-no-capacity` added to cargo-perf.schema.json; a test asserts the schema's rule-key set equals `registry::rule_ids()`; match arms added in `print_rule_explanation` (src/main.rs:554) with a test asserting no rule falls through to the generic arm. S. [acc-4, rigor-4, docs-3, adopt-5]

**Make SECURITY.md true.** It is the document that decides whether a team runs us on their source. Acceptance: supported-versions row reads 0.7.x; `#![forbid(unsafe_code)]` in src/lib.rs; the recursion-limit bullet narrowed to the rule-visitor pass with the parser's unbounded-stack caveat stated; the TOCTOU claim narrowed to config.rs, with baseline.rs moved to fd-based metadata; panic containment noted as requiring `panic=unwind`; private vulnerability reporting enabled (`gh api -X PUT .../private-vulnerability-reporting/enable`) or replaced with a real contact. M. [x1-1, x1-2, x1-3, x1-4, x1-5, x1-6]

**Clean the small correctness and doc drift blocking a clean release.** Acceptance: `.build().unwrap()` in src/main.rs:181 returns an error instead of panicking; examples/github-workflow.yml uses `dtolnay/rust-toolchain@stable` and a current version pin; DESIGN.md's suppression example uses the real `cargo-perf-ignore:` syntax; the no-op `[database] orm` key is either wired into `orm_imported()` or removed from config.rs, the schema and DESIGN.md. S. [rigor-6, adopt-2, docs-5, docs-4]

## Next (1 to 3 months) - what compounds after 0.7.0 ships

**Path and test exclusion.** Every adopting team hits false positives in tests, benches and generated code on day one with no way to silence them. Acceptance: `[files] exclude` globs plus `scan_tests` (default false) in config.rs and the schema, `--exclude`/`--include-tests` flags, tests/cli.rs proving a tests-only finding is skipped by default and a `src/generated/**` glob suppresses generated code; the dogfood step then scans `.` rather than `src/`. M. [x2-1, x2-4, x2-2]

**Stop silently skipping code.** A scan that claims completeness must report what it dropped. Acceptance: nested `build`/`out`/`dist` directories are no longer excluded below the analysis root (or emit a verbose skip warning); symlinked sources log a skip under `CARGO_PERF_VERBOSE`; config discovery walks up to the workspace root; unit tests cover all three. M. [x2-5, x2-6, x2-3]

**Widen the accuracy evidence base.** Ten of fourteen rules rest on a single fixture, so recall is quantized and the scorecard proves shape-matching, not generalization. Acceptance: 2 to 3 syntactically distinct positive fixtures per single-fixture rule; the 5 known_gaps fixtures promoted, starting with the two iterator-adapter-as-loop gaps (rule-local changes in memory_rules.rs `CloneInLoopVisitor` and allocation_rules.rs `MutexLockVisitor`, not a shared visitor change); accuracy.rs floors stay at 1.00. M. [acc-1, acc-2]

**Wire the precision oracle into the rules that lack it.** `ImportOracle` exists to stop shadowed-name false positives and three of five large rule files never call it. Acceptance: `MutexLockVisitor` gates `type_mentions_lock`/`expr_is_lock_ctor` on `!imports.is_local_item(ident)`; n-plus-one's `orm_imported()` reads `Item::Use` trees instead of scanning file text; new guard fixtures (user-defined `Mutex`, ORM named only in a comment, ORM via a prelude re-export) land red first. M. [acc-7, acc-8, rigor-2]

**Measure real-world precision and hostile-input robustness in CI.** Today real-crate-scan only proves we do not crash. Acceptance: real-crate-scan.sh records findings per rule per crate against a committed baseline and fails on an unexplained jump; one async-heavy crate added to the repo list so src/rules/async_rules.rs and lock_across_await.rs see real code; a scheduled workflow runs the bounded cargo-fuzz invocation already documented in fuzz/README.md; a CI step builds benchmarks/ so it cannot rot. M. [proof-2, proof-3, proof-4, proof-5]

**Make the baseline trustworthy.** It is the advertised safe adoption path for existing codebases and it currently under-reports and churns. Acceptance: fingerprints carry an occurrence ordinal so duplicate findings each get a slot; the context hash stops covering adjacent lines; `baseline --update` prunes and reports stale entries; the schema `version` field is checked on load. Each lands with a red test in baseline.rs plus a tests/cli.rs case. M. [x4-1, x4-2, x4-3, x4-4]

**Run the LSP tests and collapse the duplicated visitor boilerplate.** Acceptance: ci.yml's test job runs `--all-features` so the three lsp tests execute, plus new tests driving `did_open`/`did_change`/`code_action`; the hand-copied loop-tracking blocks replaced by `impl_loop_tracking_visitor!(@methods);` at 9 of 10 sites, leaving `CloneInLoopVisitor::visit_expr_for_loop` hand-written for its copy_vars logic. M. [rigor-5, rigor-1]

## Later (3+ months)

**Ground severity in measurement.** All three Error-tier rules have no published cost evidence, and a 737x rule shares a tier with one our own benchmark shows at ~1x. Acceptance: benchmarks for async-block-in-async and lock-across-await published in both README tables; string-concat-loop's bench redesigned at realistic sizes or the rule downgraded to Info; a written caveat rather than a fake micro-bench for n-plus-one-query; a severity-methodology section in docs/accuracy.md requiring a number before any rule ships above Info. M. [x3-1, x3-2, x3-3, x3-4]

**Guard our own throughput.** Adoption depends on being fast enough to run unattended in every consumer's CI, and nothing measures that. Acceptance: a criterion `[[bench]]` running `analyze()` over a fixed corpus, with an informational CI step diffing against a committed baseline. M. [proof-6]

**Extend autofix and unify the library error type.** Acceptance: the 11 fix-less rules classified safe-mechanical versus unsafe, one more autofix shipped (regex-in-loop hoisting) and the autofixable set documented; `Config::load_or_default` and the json/sarif reporters return `crate::Result`, making `Error::Config` reachable. L. [acc-5, rigor-3]

**Close the integration surface.** Acceptance: SARIF rules carry `helpUri` and `fullDescription` so Code Scanning alerts link to rule docs; editors/vscode either publishes to the Marketplace or its README documents an install path that works (`npx vsce package` before `--install-extension`). M. [adopt-4, adopt-6]

**Rewrite DESIGN.md and CONTRIBUTING.md against the real tree.** Acceptance: shipped v0.1 to v0.4 items checked off and a v0.5 to v0.7 section added; the 6 unimplemented rule ids moved to a labeled backlog; crate structure and the `Diagnostic` type regenerated from src/; CONTRIBUTING.md adds database_rules.rs and resolve.rs and corrects `fix.rs` to `fix/mod.rs`. M. [acc-3, docs-6, docs-7, docs-8]

## Not doing / deferred

- **collect-then-iterate versus clippy::needless_collect disclosure** [acc-6]: `needless_collect` is a nursery lint with open false-positive issues, not a default-on lint, so DESIGN.md's "Unique" label is defensible as written.
- **A committed adversarial-hunt harness** [proof-1]: docs/accuracy.md already describes the hunt as a discovery process rather than a rerunnable script; revisit only as contributor onboarding, not as a proof gap.
- **awesome-rust listing, docs site, homebrew tap** [adopt-7]: pointless until the onboarding bugs above are fixed and verified. Backlog note, not a work item.

## How this list is maintained

Items come from the 2026-09 audit and the 2026-07 adversarial FP/FN hunt; nothing enters this file without a finding id. Every item lands via TDD, red before green, one focused commit per fix, and the acceptance criteria above are the test that must fail first. The roadmap is updated in the same PR that closes an item, so the file and the tree never disagree.
