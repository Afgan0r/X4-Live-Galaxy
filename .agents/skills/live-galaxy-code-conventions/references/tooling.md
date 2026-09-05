# Repository Tools and Check Runners

Apply these rules to retained tools/scripts in every language, including
PowerShell launchers. The parent skill owns common configuration, state,
errors, and its logging reference owns diagnostic policy.

## TOOL-01 — Invocation

Define inputs, environment prerequisites, working directory, relative-path
bases, result format, and failure outcomes. Do not depend on the author's
accidental current directory or hard-coded machine paths. Use configuration
or a defined discovery mechanism for machine-specific locations.

Keep machine results separate from progress/diagnostics. Account for the
PowerShell success pipeline: incidental command output must not become extra
result objects.

## TOOL-02 — External commands

Pass data as arguments through the process API rather than building executable
shell text. When a shell is necessary, quote for that actual shell. Interpret
exit codes/stdout/stderr by the called command's contract: nonzero can describe
an expected condition, and empty stderr does not prove success. Report failed
mandatory steps and propagate bounded waits/cancellation to owned child work.

## TOOL-03 — Files and publication

Verify resolved targets before deletion, movement, or overwrite. Constrain
changes to the intended area and preserve unrelated files. Publish output as
ready only after required writes complete; safely replace output whose previous
valid version must survive failure. Define temporary-work and cleanup
ownership. Respect private-artifact retention and installed-game read-only
boundaries from the repository contract.

## TOOL-04 — Preparation

Install/update dependencies explicitly. Ordinary checks use prepared tools and
must not silently upgrade them. Preserve pinned versions/lockfiles. Missing or
incompatible mandatory tools cause an honest failure, not a successful skip
or an improvised substitute.

## TOOL-05 — Check integrity

Success requires that mandatory steps ran against intended inputs. Accidental
empty discovery, missing runners, swallowed child failures, and permissive
fallbacks are not success. Identify deliberate optional skips; they cannot
satisfy a required gate. Report actual scope, including focused versus full
regression. Use repository formatter/lint commands and permitted exceptions;
do not silently reduce checks, thresholds, or inputs. Rust tools use the same
workspace policy as product Rust.
