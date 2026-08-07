Tests are added into the `~/tests/specs` folder.

They have the format:

```
== description goes here ==
const    u    =     2;

[expect]
const u = 2;
```

See [here](https://github.com/dprint/dprint/tree/main/crates/development#test-specs) for more details.

## zts fork: the zts spec suite

`tests/specs/zts/` covers the constructs the zts language adds. Same format and
same harness as everything else; the files carry a `-- file.zts --` header so
they exercise the `.zts` extension path too.

  enum/          declarations, payload fields, the `mut` opt-out
  match/         the expression, its arms, and every pattern form
  expressions/   expression `if`, expression blocks, `not`, postfix `?`
  declarations/  newtype, union, impl (incl. traits v2), constrict
  types/         `T[+]`
  comments/      comment attachment around and inside every construct
  fixtures/      whole files, seeded from the zts compiler's own .zts fixtures

The `fixtures/` group is the interesting one: those inputs are real programs
someone wrote by hand, so they catch the layout decisions that only show up when
constructs nest — a match inside an impl method inside an enum's trait, an `if`
expression as a match arm body, a `?` inside a destructuring initializer.

### Idempotence

`tests/zts_idempotence.rs` asserts `fmt(fmt(x)) == fmt(x)` over the whole zts
spec corpus directly.

The harness already formats twice, but it checks the second pass against the
spec's own `[expect]` block, which makes idempotence a consequence of the
expectations being right rather than a property in its own right. The dedicated
test starts from both the input and the expectation of every spec, at two indent
widths, at a narrow line width, and with semicolons off — the last of which is
what caught both round-trip bugs the print rules originally had (see
`zts_requires_semi_colon` in generate.rs: a `?` and an expression-block tail both
make a semicolon load-bearing rather than stylistic).

## zts fork: specs marked `(skip)`

### `tests/specs/declarations/enum/**` — TypeScript `enum` (17 specs)

Every spec in that directory is marked `(skip)`. They are kept, not deleted:
they are the reference for what a `enum` print rule has to produce, and if zts
ever grows a TS-enum compatibility mode they are the suite to un-skip.

Why they cannot run: in zts the `enum` keyword is *taken*. zts `enum` declares a
variant-with-payload type (`enum Shape { Circle { r: number } }`) that lowers to
a tagged union plus factory functions — it is the one place zts is deliberately
not a superset of TypeScript, and TypeScript member syntax is a hard parse error
with a dedicated diagnostic:

```
zts enum variants take the form `Variant { field: Type, ... }` — TypeScript enum
members are not supported; zts `enum` lowers to a tagged union + factory functions
```

So these specs cannot be re-baselined, only skipped: there is no output for them
to have. This is a language decision, not a formatter limitation.

Note also that leaving them unskipped does not merely fail the suite — it hangs
it. `run_spec` turns a format error into a `panic!`, and the parallel file runner
deadlocks rather than reporting when a worker panics, so the whole run wedges
with no output.

### Five JSX specs — whitespace inside a closing tag

- `jsx/JsxClosingElement/JsxClosingElement_All.txt` — `<Test>Test<  / Test   >`
- `jsx/JsxFragment/JsxClosingFragment_All.txt` — `<><   / >`
- `jsx/JsxOpeningElement/JsxOpeningElement_All.txt`
- `jsx/JsxOpeningElement/JsxOpeningElement_PreferHanging_True.txt`
- `jsx/JsxOpeningElement/JsxOpeningElement_PreferSingleLine_True.txt`

The last three are the same input, `<Test attrib={5} other="test"> < /  Test  >`,
under three configs.

All five stopped parsing ("Expression expected") somewhere between the
swc_ecma_parser this plugin was published against (27.0.7) and the one the swc
fork is based on (43.0.0): whitespace between `<` and `/` in a closing tag, and
around the name, is no longer accepted.

This is **not** a zts interaction. Verified by turning the zts syntax flag off in
the deno_ast fork and re-running: all five still fail, identically. It is upstream
swc drift on an input that is arguably invalid JSX anyway (TypeScript itself
rejects `<  / Test   >`), so there is nothing here for zts-fmt to fix and no
reason to declare `.zts` non-JSX over it. If a later swc restores the behaviour,
un-skip them.

### Sweeping the specs without the harness

`cargo run --release --example zts_spec_sweep` formats every default-config spec
input and reports ALL failures instead of panicking on the first one. Add
`--emit <dir>` to write each formatted result to a file, which is how output
parity with stock dprint-plugin-typescript is checked (run the same example in a
stock checkout at the same base commit and diff the two directories).
