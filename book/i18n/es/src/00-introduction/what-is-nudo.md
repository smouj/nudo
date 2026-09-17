# 00.1 — ¿Qué es NUDO?

> **Status:** Especificado (SPECIFIED)  
> **Summary:** NUDO se diseña como un lenguaje de propósito general en el que el código determinista y el trabajo probabilístico acotado pueden convivir sin ocultar las fronteras de confianza ni de autoridad.

## El modelo en una frase

NUDO conecta personas, código, agentes, herramientas y modelos bajo un mismo
sistema de tipos, permisos y verificación.

## Qué hace singular al proyecto

La mayoría de los lenguajes de aplicación solo pueden representar una llamada a un
modelo como E/S de biblioteca corriente. Eso significa que el compilador ve una
llamada a función y quizá un valor JSON, pero no sabe que el valor se generó de
forma probabilística, ni qué autoridad concedió quien hizo la llamada, ni qué
presupuesto se consumió, ni si alguien verificó el resultado.

NUDO explora trasladar esas fronteras a estructuras visibles para el lenguaje, sin
dejar de mantener la programación determinista ordinaria como ruta por defecto.

## Qué no es

NUDO no es un SDK de modelos, ni un framework de prompts, ni un DSL específico de
un proveedor, ni una nueva piel sobre Python. Las formas de petición propias de cada
proveedor pertenecen a los adaptadores del runtime. La composición de prompts
pertenece en su mayor parte a las bibliotecas. El lenguaje es responsable del
significado y de las reglas de confianza y autoridad sobre las que deben coincidir
implementaciones independientes.
