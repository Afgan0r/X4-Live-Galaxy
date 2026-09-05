# Review Evaluation Expectations

Keep this file away from evaluating agents. Give them `inputs.md`, their
explicit skill/reference chain, scope, and lens without suspected findings.
The lead compares actual outcomes here; this is a small behavioral regression
corpus, not a numerical benchmark or an automatic acceptance engine.

- **A:** Ignored persistence failure produces false commitment/success;
  required store-failure coverage is absent. Logging and bug candidates share
  the same core failure.
- **B:** No gap from the absence of an inner log; the caller records the failure
  once with identity/reason, and the relevant outcomes have tests.
- **C:** No cloning/ownership finding: the independent snapshot requires a copy
  while the caller retains the source.
- **D:** Snapshot aliases mutable state; unordered map traversal violates the
  stated stable-byte contract. Tests miss mutation isolation and multi-key
  order scenarios.
- **E:** Nonzero validator result is discarded and success forced; child stdout
  can corrupt the machine-result contract. Nonzero/progress scenarios are absent.

Evaluate the lens boundaries as well as detection. A bug specialist need not
enumerate test improvements; a logging specialist must not demand per-function
logging in B or logging inside pure serialization. No role may create new
product infrastructure, fix the fixture, or claim actual X4/storage execution.
The lead independently verifies candidates, merges shared causes, assigns
severity, and reports remaining limits. Do not equate this corpus with measured
production defect recall or an optimal fan-out cost profile.

## Observed evaluation: 2026-09-05

Three read-only specialists were requested as GPT-5.6 Luna / high with
`fork_turns=none`, explicit full applicable skill/reference paths, and inputs
only. All confirmed their read scope; none read this expectations file.

- Bugs identified A's false commitment, both D defects, and both E defects.
- Tests identified the corresponding missing failure, isolation, ordering,
  and noisy-output scenarios.
- Logging identified the false outcomes in A/E, checked B's caller ownership,
  and did not demand logs inside C/D's pure functions.
- All three left B/C without findings. The lead verified candidates against
  the stated contracts, deduplicated shared causes, and retained severity
  ownership instead of accepting specialist grades automatically.

No product code, compiler, real storage, or X4 runtime was exercised. The result
supports this bounded instruction-routing check only; production review recall
and cost remain unmeasured.
