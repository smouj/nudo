# 09.1 — MCP, A2A und WebAssembly

> **Status:** Geplant  
> **Summary:** Interoperabilität soll NUDO an bestehenden Tool- und Agenten-Ökosystemen teilnehmen lassen, ohne dass Adapter Autorität erweitern dürfen.

MCP-Interoperabilität kann NUDO Tools nutzen oder bereitstellen lassen. Delegation
im A2A-Stil kann unabhängige Agentenimplementierungen verbinden. WASI kann ein
portables Ausführungsziel mit expliziten Host-Capabilities bieten.

Die Sicherheitsregel ist stärker als die Protokollkompatibilität: Adaptern darf
nicht vertraut werden, sich selbst Capabilities zu gewähren, die der NUDO-Aufrufer
nicht besaß.
