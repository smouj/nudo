# 10.1 — Arquitectura del repositorio

> **Status:** Base implementada (IMPLEMENTED FOUNDATION)  
> **Summary:** El monorepositorio mantiene la evolución de la especificación, el compilador, el runtime y las herramientas en una única historia auditable, preservando a la vez las fronteras conceptuales entre crates.

El workspace separa las fases del compilador, los servicios del runtime, los crates
de fundamento compartido, la CLI, las herramientas y los backends. Eso hace visible
la propiedad del código, pero las fronteras entre crates en fase pre-alpha deberían
seguir siendo revisables.

Un crate de relleno es un marcador de hoja de ruta, no la prueba de que su API pública
esté lista. Debería permitirse que la implementación fusione o divida unidades
internas cuando la experiencia muestre una frontera mejor.
