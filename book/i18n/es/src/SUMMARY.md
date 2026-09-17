# Sumario

[Introducción](index.md)

# 00 — Introducción
- [¿Qué es NUDO?](00-introduction/what-is-nudo.md)
- [Por qué existe NUDO](00-introduction/why-exists.md)
- [Estado actual del proyecto](00-introduction/status.md)

# 01 — Filosofía
- [Primero el software determinista](01-philosophy/deterministic-first.md)
- [Confianza explícita](01-philosophy/explicit-trust.md)
- [Local-first y agnóstico del proveedor](01-philosophy/local-first.md)

# 02 — Diseño del lenguaje
- [Disciplina de gramática y sintaxis](02-language-design/grammar.md)
- [Expresiones y precedencia](02-language-design/expressions.md)
- [Módulos y límites del programa](02-language-design/modules.md)

# 03 — Compilador
- [El pipeline del compilador](03-compiler/pipeline.md)
- [Modelo de código fuente y lexer](03-compiler/lexer.md)
- [Parser, sintaxis sin pérdidas y AST](03-compiler/parser.md)
- [Los diagnósticos como diseño de producto](03-compiler/diagnostics.md)

# 04 — Sistema de tipos
- [Fundamentos del sistema de tipos](04-type-system/fundamentals.md)
- [`Generated<T>` y `Verified<T>`](04-type-system/generated-verified.md)

# 05 — Agentes e IA
- [Los agentes como ejecutores acotados](05-agents-and-ai/agents.md)
- [Tareas, herramientas y modelos](05-agents-and-ai/tasks-tools-models.md)

# 06 — Seguridad
- [Capacidades, efectos y autoridad](06-security/capabilities-effects.md)
- [Modelo de amenazas](06-security/threat-model.md)

# 07 — Runtime
- [Trazas y procedencia](07-runtime/trace-provenance.md)
- [Presupuestos y aprobaciones](07-runtime/budgets-approvals.md)

# 08 — Herramientas
- [Cadena de herramientas y CLI](08-tooling/cli.md)

# 09 — Interoperabilidad
- [MCP, A2A y WebAssembly](09-interoperability/mcp-a2a-wasm.md)

# 10 — Internos
- [Arquitectura del repositorio](10-internals/repository-architecture.md)

# 11 — Justificación del diseño
- [¿Por qué Rust y por qué un compilador propio?](11-rationale/why-rust-own-compiler.md)
- [Por qué la confianza forma parte del sistema de tipos](11-rationale/why-trust-types.md)

# 12 — Alternativas descartadas
- [Alternativas descartadas deliberadamente](12-alternatives/rejected.md)

# 13 — Ejemplos
- [Programa determinista ordinario](13-examples/ordinary-program.md)
- [Salida del modelo y flujo de verificación](13-examples/trust-flow.md)

# 14 — Contribuir
- [Especificación, NEPs y ADRs](14-contributing/spec-nep-adr.md)

# 15 — Historia
- [Hitos y evidencia](15-history/milestones.md)
