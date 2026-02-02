# JaLM

JaLM is a new programming language designed to be **AI agent friendly**. The core design goal is to make code generation by LLMs reliable and predictable by providing:

- A small, explicit syntax surface area.
- Deterministic typing rules with minimal inference.
- Explicit effects and structured concurrency.
- A file-based module system with clear visibility rules.

## How an AI Agent Uses This Repo

An LLM/agent can use the specs in this repository as a deterministic contract for generating valid JaLM programs:

- `SPEC_MVP.md`: Defines the MVP scope and success criteria so agents know exactly what features exist.
- `GRAMMAR_V0.md`: Provides the formal grammar so agents can generate syntactically valid code.
- `TYPE_SYSTEM_V0.md`: Specifies types, inference boundaries, and operator rules so code type-checks on first try.
- `EFFECT_SYSTEM_V0.md`: Requires explicit side-effect declarations so agents can reason about purity and capabilities.
- `STRUCTURED_CONCURRENCY_V0.md`: Defines scoped tasks and cancellation rules so agents avoid leaks or detached tasks.
- `MODULE_SYSTEM_V0.md`: Defines layout and import/visibility rules for deterministic project structure.

## Why “AI Agent Friendly” Matters

JaLM is designed so a model can:

- Synthesize code that is valid without relying on implicit conversions.
- Make effectful behavior explicit at function boundaries.
- Use structured concurrency patterns that prevent task leaks.
- Organize modules in a deterministic, file-based layout.

The result is a language where “what the model writes” is more likely to compile, be safe, and be easy to verify.
