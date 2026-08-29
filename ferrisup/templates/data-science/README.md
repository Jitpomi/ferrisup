# FerrisUp data-science templates

FerrisUp currently includes two data-science paths:

- `polars-cli` for command-line data processing with Polars, including CSV, JSON, and Parquet variants;
- Linfa examples for classification and other traditional machine-learning workflows.

Create a project through the component interface:

```bash
ferrisup new analysis --component-type data-science --framework polars
ferrisup new classifier --component-type data-science --framework linfa
```

If a required choice is omitted, FerrisUp prompts for it. Generated projects use the stable Rust toolchain, but native dependencies, large datasets, and optional visualization or linear-algebra backends can add platform-specific requirements. Review the generated manifest and README before building.

The templates are working foundations rather than complete analytical products. Validate numerical assumptions, dataset licensing, reproducibility, and model quality for the intended use case.
