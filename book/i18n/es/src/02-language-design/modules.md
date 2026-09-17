# 02.3 — Módulos y límites del programa

> **Status:** Propuesto (PROPOSED)  
> **Summary:** Los módulos deben ofrecer espacios de nombres explícitos y una estructura de programa reproducible sin ocultar la autoridad ni las fronteras de dependencia.

El diseño de módulos está muy ligado al diseño de paquetes, a la resolución de
nombres y a las interfaces públicas. NUDO debería evitar los espacios de nombres
globales implícitos y debería hacer deliberada la superficie de API pública.

El trabajo de diseño abierto incluye los manifiestos de paquete, la visibilidad, la
resolución de dependencias y cómo se exponen a quien llama las capacidades
declaradas por las dependencias.
