Tests are added into the `~/tests/specs` folder.

They have the format:

```
== description goes here ==
const    u    =     2;

[expect]
const u = 2;
```

See [here](https://github.com/dprint/dprint/tree/main/crates/development#test-specs) for more details.

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
