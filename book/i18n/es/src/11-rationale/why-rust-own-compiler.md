# 11.1 — ¿Por qué Rust y por qué un compilador propio?

> **Status:** Justificación especificada  
> **Summary:** Rust aporta un sustrato de implementación sólido, mientras que un compilador dedicado es necesario si la semántica de NUDO debe existir con independencia de otro lenguaje anfitrión.

## Por qué Rust para la implementación

Rust ofrece propiedad explícita, comprobación estática fuerte, buen rendimiento,
herramientas de análisis y compilación bien desarrolladas y distribución
multiplataforma práctica. Eso no convierte a Rust en parte de la semántica del
lenguaje NUDO; es el lenguaje de implementación.

## ¿Por qué no compilar a Python como definición?

Si la semántica de NUDO fuera simplemente la semántica de Python más una sintaxis,
Python se convertiría en la especificación real. Características como los tipos de
confianza, las comprobaciones de capacidades y los diagnósticos estables del
compilador heredarían las restricciones del lenguaje anfitrión.

Un compilador dedicado permite que la especificación de NUDO siga siendo la
autoridad. Los backends pueden cambiar más adelante sin redefinir qué significa un
programa.
