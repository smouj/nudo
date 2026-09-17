# 09.1 — MCP, A2A y WebAssembly

> **Status:** Planificado  
> **Summary:** La interoperabilidad debería permitir que NUDO participe en los ecosistemas existentes de herramientas y agentes sin permitir que los adaptadores amplíen la autoridad.

La interoperabilidad con MCP puede permitir que NUDO consuma o exponga herramientas.
La delegación al estilo A2A puede conectar implementaciones de agentes
independientes. WASI puede ofrecer un objetivo de ejecución portátil con capacidades
explícitas del host.

La regla de seguridad es más fuerte que la compatibilidad de protocolo: no debe
confiarse en los adaptadores para que se concedan a sí mismos capacidades que quien
hace la llamada desde NUDO no poseía.
